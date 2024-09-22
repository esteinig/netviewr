use rayon::prelude::*;
use petgraph::{dot::Dot, Graph, Undirected};
use csv::WriterBuilder;
use serde_json;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::Path};
use petgraph::visit::EdgeRef;
use core::f64::NAN;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

use crate::dist::make_symmetrical;
use crate::error::NetviewError;

/// Helper struct for managing neighbors during sorting.
// struct ReverseNeighbor(f64, usize);

// impl Ord for ReverseNeighbor {
//     fn cmp(&self, other: &Self) -> Ordering {
//         other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
//     }
// }

// impl PartialOrd for ReverseNeighbor {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl PartialEq for ReverseNeighbor {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }

// impl Eq for ReverseNeighbor {}

/// Calculates the k-mutual nearest neighbors from a distance matrix.
///
/// This function takes a symmetrical or lower triangular distance matrix and a value of `k`
/// and returns a list of k mutual nearest neighbors for each element.
///
/// # Arguments
///
/// * `distance_matrix` - A slice of Vec<Vec<f64>> representing the distance matrix.
/// * `k`               - The number of nearest neighbors to find for each element.
///
/// # Returns
///
/// Returns a `Result` containing either the calculation result as a Vec<Vec<usize>>,
/// indicating the indices of the k mutual nearest neighbors for each element,
/// or an error of type `NetviewError`.
///
/// # Examples
///
/// ```
/// use netview::k_mutual_nearest_neighbors;
///
/// let distance_matrix = vec![
///     vec![0.0, 1.0, 2.0],
///     vec![1.0, 0.0, 3.0],
///     vec![2.0, 3.0, 0.0],
/// ];
/// let k = 1;
/// let mnn = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
/// assert_eq!(mnn, vec![vec![1], vec![0], vec![]]);
/// ```
pub fn k_mutual_nearest_neighbors(distance_matrix: &Vec<Vec<f64>>, k: usize) -> Result<Vec<Vec<usize>>, NetviewError> {
    let n = distance_matrix.len();
    
    // Validate the matrix is non-empty and either square or lower triangular
    if n == 0 || distance_matrix.iter().any(|row| row.len() > n) {
        return Err(NetviewError::InvalidMatrix);
    }
    if k == 0 || k >= n {
        return Err(NetviewError::InvalidK);
    }

    // Transform lower triangular matrix to a symmetrical matrix if needed
    let matrix = make_symmetrical(distance_matrix)?;

    // Compute nearest neighbors in parallel
    let nearest_neighbors: Vec<Vec<usize>> = (0..n).into_par_iter().map(|i| {
        let mut neighbors = vec![];
        for j in 0..n {
            if i != j {
                neighbors.push((j, matrix[i][j]));
            }
        }

        // Sort by distance and select the k nearest
        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        neighbors.into_iter().map(|(index, _)| index).take(k).collect::<Vec<usize>>()
    }).collect();

    // Identify mutual nearest neighbors
    let mutual_nearest_neighbors: Vec<Vec<usize>> = nearest_neighbors.iter().enumerate().map(|(i, neighbors)| {
        neighbors.iter().filter(|&&j| nearest_neighbors[j].contains(&i)).cloned().collect()
    }).collect();

    Ok(mutual_nearest_neighbors)
}


/// Converts the output of k-mutual-nearest-neighbors and the optional original distance matrix into a `petgraph::Graph`.
///
/// # Arguments
///
/// * `mutual_nearest_neighbors` - The output from `k_mutual_nearest_neighbors`, a Vec<Vec<usize>> indicating the indices 
///                                of the k mutual nearest neighbors for each element.
/// 
/// * `distance_matrix`          - An optional reference to the original distance matrix as a Vec<Vec<f64>>. If provided, 
///                                edges are weighted by the distances between mutual nearest neighbors; otherwise, a 
///                                default weight of 1.0 is used.
///
/// # Returns
///
/// A `Graph<usize, f64>`, where nodes represent the indices in the original dataset, and edges are weighted by the distances 
/// between mutual nearest neighbors if a distance matrix is provided.
///
/// # Examples
///
/// ```
/// use petgraph::Graph;
/// use std::collections::HashMap;
///
/// // Example usage with a distance matrix
/// let distance_matrix = Some(vec![
///     vec![0.0, 1.0, 2.0],
///     vec![1.0, 0.0, 3.0],
///     vec![2.0, 3.0, 0.0],
/// ]);
/// let mutual_nearest_neighbors = vec![vec![1], vec![0], vec![1]]; // Simplified example output
/// let graph_with_distances = convert_to_graph(&mutual_nearest_neighbors, distance_matrix.as_ref());
/// assert_eq!(graph_with_distances.edge_count(), 2);
///
/// // Example usage without a distance matrix
/// let graph_without_distances = convert_to_graph(&mutual_nearest_neighbors, None);
/// assert_eq!(graph_without_distances.edge_count(), 2);
/// ```
pub fn convert_to_graph(mutual_nearest_neighbors: &Vec<Vec<usize>>, distance_matrix: Option<&Vec<Vec<f64>>>) -> Result<Graph<usize, f64, Undirected>, NetviewError> {
    let mut graph = Graph::<usize, f64, Undirected>::new_undirected();

    let mut index_map: HashMap<usize, NodeIndex> = HashMap::new();

    // First add all nodes so that the node indices are consistent:
    for (node, _) in mutual_nearest_neighbors.iter().enumerate(){
        let node_index = graph.add_node(node);
        index_map.insert(node, node_index);
    }

    for (node, neighbors) in mutual_nearest_neighbors.iter().enumerate() {

        let node_index = *index_map.get(&node).ok_or(NetviewError::NodeIndexError)?;

        for &neighbor in neighbors.iter() {

            let distance = match distance_matrix {
                Some(matrix) => matrix.get(node).and_then(|row| row.get(neighbor)).copied().unwrap_or(1.0), // Use 1.0 or another default value as the default distance
                None => 1.0, // Default weight when distance_matrix is not provided
            };

            let neighbor_index = *index_map.get(&neighbor).ok_or(NetviewError::NodeIndexError)?;
            graph.add_edge(node_index, neighbor_index, distance);
        }
    }

    Ok(graph)
}


#[derive(Serialize, Deserialize, Clone, Debug, clap::ValueEnum)]
pub enum GraphFormat {
    Dot,
    Json,
    Adjacency,
}



/// Writes a `petgraph::Graph` to a file in specified formats (DOT or JSON).
///
/// This function supports exporting the graph to various formats for visualization
/// or further processing. Currently supported formats are DOT, for use with Graphviz,
/// and JSON, for generic data interchange.
///
/// # Arguments
/// * `graph`  - Reference to the graph to be written.
/// * `path`   - Path to the output file where the graph should be written.
/// * `format` - Specifies the output format ("dot" or "json").
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(GraphWriteError)` on failure, with detailed error information.
///
/// # Examples
///
/// ```
/// use petgraph::Graph;
/// use std::path::Path;
/// use netview::write_graph_to_file;
///
/// let mut graph = Graph::new();
/// let a = graph.add_node("Node A");
/// let b = graph.add_node("Node B");
/// graph.add_edge(a, b, "connects");
///
/// write_graph_to_file(&graph, Path::new("graph.dot"), "dot").unwrap();
/// write_graph_to_file(&graph, Path::new("graph.json"), "json").unwrap();
/// write_graph_to_file(&graph, Path::new("graph.tsv"), "adjmatrix").unwrap();
/// ```
pub fn write_graph_to_file<N, E>(
    graph: &Graph<N, E, Undirected>,
    path: &Path,
    format: &GraphFormat,
) -> Result<(), NetviewError>
where
    N: Serialize + std::fmt::Debug,
    E: Serialize + Into<f64> + std::clone::Clone + std::fmt::Debug,
{
    let mut file = File::create(path).map_err(|e| NetviewError::GraphFileError(e.to_string()))?;

    match format {
        GraphFormat::Dot => {
            let dot = Dot::with_config(graph, &[petgraph::dot::Config::EdgeNoLabel]);
            write!(file, "{:?}", dot).map_err(|e| NetviewError::GraphFileError(e.to_string()))?;
        },
        GraphFormat::Json => {
            let adj_matrix = graph_to_adjacency_matrix(graph, false)?;
            serde_json::to_writer(&file, &adj_matrix).map_err(|e| NetviewError::GraphSerializationError(e.to_string()))?;
        },
        GraphFormat::Adjacency => {
            let adj_matrix = graph_to_adjacency_matrix(graph, false)?;
            write_adjacency_matrix_to_file(&adj_matrix, path)?;   
        }
    }

    Ok(())
}


/// Writes an adjacency matrix to a tab-delimited file.
///
/// # Arguments
/// * `matrix` - The adjacency matrix to be written.
/// * `path` - The file path where the matrix should be written.
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(MatrixFileError)` on failure, detailing the issue encountered.
///
/// # Examples
///
/// ```
/// use netview::write_adjacency_matrix_to_file;
///
/// let matrix = vec![
///    vec![0.0, 1.5, 2.5],
///    vec![1.5, 0.0, 3.5],
///    vec![2.5, 3.5, 0.0],
/// ];
/// write_adjacency_matrix_to_file(&matrix, "path/to/matrix.tsv").unwrap();
/// ```
pub fn write_adjacency_matrix_to_file(matrix: &Vec<Vec<f64>>, path: impl AsRef<Path>) -> Result<(), NetviewError> {
    let file = File::create(path).map_err(|e| NetviewError::WriteError(e.to_string()))?;
    let mut wtr = WriterBuilder::new().delimiter(b'\t').from_writer(file);

    for row in matrix {
        wtr.serialize(row).map_err(NetviewError::CsvError)?;
    }
    wtr.flush().map_err(|err| NetviewError::CsvError(err.into()))
}

/// Writes a petgraph `Graph<N, E>` to a JSON file.
///
/// # Arguments
///
/// * `graph` - The graph to be serialized and written to a file.
/// * `path` - The file path where the graph should be written.
///
/// # Returns
///
/// * `Ok(())` on success.
/// * `Err(NetviewError)` on failure, detailing the issue encountered.
///
/// # Examples
///
/// ```no_run
/// use petgraph::Graph;
/// use std::path::Path;
/// use netview::write_json_graph;
///
/// let mut graph = Graph::<&str, f64>::new();
/// let a = graph.add_node("Node A");
/// let b = graph.add_node("Node B");
/// graph.add_edge(a, b, 1.23);
///
/// write_json_graph(&graph, Path::new("graph.json")).unwrap();
/// ```
pub fn write_json_graph<N, E>(graph: &Graph<N, E>, path: &Path) -> Result<(), NetviewError>
where
    N: Serialize + Clone,
    E: Serialize + Clone,
{
    // Convert the graph into a serializable structure
    #[derive(Serialize)]
    struct EdgeData<N, E> {
        source: N,
        target: N,
        weight: E,
    }

    #[derive(Serialize)]
    struct GraphData<N, E> {
        nodes: Vec<N>,
        edges: Vec<EdgeData<usize, E>>,
    }

    let nodes: Vec<_> = graph.node_indices().map(|n| graph[n].clone()).collect();
    let edges: Vec<_> = graph.edge_references().map(|e| {
        EdgeData {
            source: e.source().index(),
            target: e.target().index(),
            weight: e.weight().clone(),
        }
    }).collect();

    let graph_data = GraphData { nodes, edges };

    // Attempt to open the file for writing
    let file = File::create(path).map_err(|e| NetviewError::GraphFileError(e.to_string()))?;

    // Attempt to serialize the graph data to JSON and write it to the file
    serde_json::to_writer(file, &graph_data).map_err(|e| NetviewError::GraphSerializationError(e.to_string()))?;

    Ok(())
}

/// Converts a `Graph<N, E, Undirected>` into an adjacency matrix
///
/// # Arguments
///
/// * `graph` - Input `petgraph::Graph` to be converted.
/// * `nan`   - If `true`, non-existent edges (i.e., no direct path between two nodes) are
///             represented by `NaN` in the matrix. If `false`, they are represented by `0.0`.
///
/// # Returns
///
/// A `Result` containing either the adjacency matrix as `Vec<Vec<f64>>` 
/// if successful or a `NetviewError` in case of failure.
///
/// # Errors
///
/// This function can return a `NetviewError` if the conversion of edge weights to `f64` fails.
///
/// # Examples
///
/// ```
/// use petgraph::Graph;
/// use petgraph::graph::NodeIndex;
/// use petgraph::Undirected;
/// use netview::graph_to_adjacency_matrix;
///
/// let mut graph = Graph::<&str, f64, Undirected>::new_undirected();
/// let a = graph.add_node("A");
/// let b = graph.add_node("B");
/// graph.add_edge(a, b, 1.5);
///
/// let matrix_with_nan = graph_to_adjacency_matrix(&graph, true).unwrap();
/// assert_eq!(matrix_with_nan[0][1], 1.5);
/// assert!(matrix_with_nan[0][0].is_nan());
///
/// let matrix_without_nan = graph_to_adjacency_matrix(&graph, false).unwrap();
/// assert_eq!(matrix_without_nan[0][1], 1.5);
/// assert_eq!(matrix_without_nan[0][0], 0.0);
/// ```
///
/// This example demonstrates how to use the function with both representations for non-existent edges, 
/// showing how to convert a graph into an adjacency matrix with either `NaN` or `0.0` for missing edges.
pub fn graph_to_adjacency_matrix<N, E>(graph: &Graph<N, E, Undirected>, nan: bool) -> Result<Vec<Vec<f64>>, NetviewError>
where
    E: Clone + Into<f64>
{
    let node_count = graph.node_count();
    let mut matrix = vec![vec![match nan { true => NAN, false => 0.}; node_count]; node_count];

    for edge_ref in graph.edge_references() {
        let (source, target) = (edge_ref.source().index(), edge_ref.target().index());
        // Safely attempt conversion of E to f64, handling potential conversion issues
        let weight: f64 = edge_ref.weight().clone().into();

        matrix[source][target] = weight;
        // Since the graph is undirected, mirror the weights
        matrix[target][source] = weight;
    }

    Ok(matrix)
}
    


#[cfg(test)]
mod tests {

    use super::*;

    use petgraph::graph::NodeIndex;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_empty_matrix() {
        let distance_matrix = Vec::<Vec<f64>>::new();
        let k = 1;
        assert!(matches!(
            k_mutual_nearest_neighbors(&distance_matrix, k),
            Err(NetviewError::InvalidMatrix)
        ));
    }

    #[test]
    fn test_invalid_k_zero() {
        let distance_matrix = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let k = 0;
        assert!(matches!(
            k_mutual_nearest_neighbors(&distance_matrix, k),
            Err(NetviewError::InvalidK)
        ));
    }

    #[test]
    fn test_invalid_k_large() {
        let distance_matrix = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let k = 3; // Larger than the number of elements
        assert!(matches!(
            k_mutual_nearest_neighbors(&distance_matrix, k),
            Err(NetviewError::InvalidK)
        ));
    }

    #[test]
    fn test_single_element() {
        let distance_matrix = vec![vec![0.0]];
        let k = 1;
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        assert_eq!(result, vec![Vec::<usize>::new()]); // No neighbors for a single-element matrix
    }

    #[test]
    fn test_symmetrical_matrix_simple() {
        let distance_matrix = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let k = 1;
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        assert_eq!(result, vec![vec![1], vec![0]]);
    }

    #[test]
    fn test_lower_triangular_conversion() {
        let distance_matrix = vec![vec![0.0], vec![1.0, 0.0]];
        let k = 1;
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        assert_eq!(result, vec![vec![1], vec![0]]);
    }

    #[test]
    fn test_no_mutual_neighbors() {
        let distance_matrix = vec![vec![0.0, 2.0, 1.0], vec![2.0, 0.0, 3.0], vec![1.0, 3.0, 0.0]];
        let k = 1;
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        // Expecting no mutual neighbors as the nearest neighbor of each is not reciprocal
        assert_eq!(result, vec![Vec::<usize>::new(), Vec::<usize>::new(), Vec::<usize>::new()]);
    }

    #[test]
    fn test_with_mutual_neighbors() {
        let distance_matrix = vec![
            vec![0.0, 1.0, 2.0, 3.0],
            vec![1.0, 0.0, 3.0, 2.0],
            vec![2.0, 3.0, 0.0, 1.0],
            vec![3.0, 2.0, 1.0, 0.0],
        ];
        let k = 2;
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        // Expecting mutual neighbors as defined by the distance matrix
        assert_eq!(result, vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]]);
    }

    #[test]
    fn test_large_k_with_few_elements() {
        let distance_matrix = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let k = 2; // k is equal to the number of elements, expecting to reduce to valid neighbors
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        // Only one mutual neighbor is possible, despite k being larger
        assert_eq!(result, vec![vec![1], vec![0]]);
    }

    #[test]
    fn test_complex_mutual_neighbors() {
        let distance_matrix = vec![
            vec![0.0, 2.0, 3.0, 4.0],
            vec![2.0, 0.0, 4.0, 5.0],
            vec![3.0, 4.0, 0.0, 1.0],
            vec![4.0, 5.0, 1.0, 0.0],
        ];
        let k = 1;
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        // Only indices 2 and 3 are mutual nearest neighbors in this setup
        assert_eq!(result, vec![Vec::<usize>::new(), Vec::<usize>::new(), vec![3], vec![2]]);
    }

    #[test]
    fn test_identical_distances() {
        let distance_matrix = vec![
            vec![0.0, 1.0, 1.0],
            vec![1.0, 0.0, 1.0],
            vec![1.0, 1.0, 0.0],
        ];
        let k = 2;
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        // Each point has two equidistant nearest neighbors, making them all mutual
        assert_eq!(result, vec![vec![1, 2], vec![0, 2], vec![0, 1]]);
    }

    #[test]
    fn test_non_symmetrical_matrix_error() {
        let distance_matrix = vec![vec![0.0, 2.0], vec![2.0, 0.0, 1.0]]; // This row has an extra element
        let k = 1;
        assert!(matches!(
            k_mutual_nearest_neighbors(&distance_matrix, k),
            Err(NetviewError::InvalidMatrix)
        ));
    }

    #[test]
    fn test_full_matrix_with_no_neighbors() {
        let distance_matrix = vec![
            vec![0.0, 100.0, 100.0, 100.0],
            vec![100.0, 0.0, 100.0, 100.0],
            vec![100.0, 100.0, 0.0, 100.0],
            vec![100.0, 100.0, 100.0, 0.0],
        ];
        let k = 1;
        // No mutual neighbors due to the high distance values
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        assert_eq!(result, vec![Vec::<usize>::new(), Vec::<usize>::new(), Vec::<usize>::new(), Vec::<usize>::new()]);
    }

    #[test]
    fn test_matrix_with_self_loops() {
        let distance_matrix = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 0.0, 2.0],
            vec![2.0, 2.0, 0.0],
        ];
        let k = 2;
        // Including self-loops should not affect the outcome; mutual neighbors are based on lowest non-zero distances
        let result = k_mutual_nearest_neighbors(&distance_matrix, k).unwrap();
        assert_eq!(result, vec![vec![1, 2], vec![0, 2], vec![0, 1]]);
    }


    fn setup_test_graph() -> Graph<&'static str, i32> {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, 7);
        graph
    }

    #[test]
    fn test_write_empty_graph() {
        let graph = Graph::<&str, i32>::new();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty_graph.json");

        write_json_graph(&graph, &file_path).unwrap();
        let metadata = fs::metadata(file_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_write_simple_graph() {
        let graph = setup_test_graph();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("simple_graph.json");
        write_json_graph(&graph, &file_path).unwrap();
        assert!(file_path.exists());
        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("\"nodes\":[\"A\",\"B\"]"));
        assert!(content.contains("\"weight\":7"));
    }


    #[test]
    fn test_file_write_error() {
        // Simulate a file write error by specifying a directory that does not exist
        let graph = setup_test_graph();
        let file_path = PathBuf::from("/non_existent_directory/graph.json");

        let result = write_json_graph(&graph, &file_path);
        assert!(matches!(result, Err(NetviewError::GraphFileError(_))));
    }

    #[test]
    fn test_write_large_graph() {
        let mut graph = Graph::new();
        for i in 0..100 {
            let node = graph.add_node(format!("Node {}", i));
            if i != 0 {
                let prev_node = NodeIndex::new(i as usize - 1);
                graph.add_edge(prev_node, node, i as i32);
            }
        }

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large_graph.json");
        write_json_graph(&graph, &file_path).unwrap();
        
        let metadata = fs::metadata(&file_path).unwrap();
        assert!(metadata.len() > 0, "File should contain serialized large graph data");
    }

    #[test]
    fn test_write_graph_with_multiple_edges() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, 1);
        graph.add_edge(a, b, 2); // Adding a second edge to test multiple edges

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("multi_edge_graph.json");
        write_json_graph(&graph, &file_path).unwrap();

        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("\"weight\":1") && content.contains("\"weight\":2"), "File should contain both edges");
    }

    #[test]
    fn test_nonexistent_path() {
        // Attempt to write to a path that cannot be created
        let graph = setup_test_graph();
        // Using a path with invalid characters for most filesystems
        let file_path = PathBuf::from(format!("{}/invalid_path/graph.json", tempdir().unwrap().path().to_string_lossy()));

        let result = write_json_graph(&graph, &file_path);
        assert!(matches!(result, Err(NetviewError::GraphFileError(_))));
    }


    #[test]
    fn empty_graph() {
        let graph: Graph<&str, f64, Undirected> = Graph::new_undirected();
        let matrix = graph_to_adjacency_matrix(&graph, false).unwrap();
        assert!(matrix.is_empty());
    }



    #[test]
    fn two_node_graph_with_edge() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, 2.5);
        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        assert_eq!(matrix[0][1], 2.5);
        assert!(matrix[1][0].is_nan()); // For directed graph
    }

    #[test]
    fn graph_with_multiple_edges() {
        let mut graph = Graph::new_undirected(); // This creates a directed graph by default
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        graph.add_edge(a, b, 3.0);
        graph.add_edge(a, c, 4.5);
        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        assert_eq!(matrix[0][1], 3.0);
        assert_eq!(matrix[0][2], 4.5);
        assert!(matrix[1][0].is_nan()); // No edge from B to A in a directed graph
    }

    #[test]
    fn non_existent_edge() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_node("C"); // C has no edges
        graph.add_edge(a, b, 1.0);
        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        assert!(matrix[2][0].is_nan() && matrix[2][1].is_nan(), "Edges involving 'C' should be NaN");
    }

    #[test]
    fn graph_with_self_loops() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        graph.add_edge(a, a, 2.0); // Self-loop at A
        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        assert_eq!(matrix[0][0], 2.0, "Self-loop at 'A' should have a weight of 2.0");
    }

    #[test]
    fn large_graph_performance() {
        let mut graph = Graph::new_undirected();
        for i in 0..100 {
            graph.add_node(format!("Node {}", i));
        }
        // Creating a larger graph with edges between sequential nodes
        for i in 0..99 {
            graph.add_edge(NodeIndex::new(i), NodeIndex::new(i + 1), i as f64 + 1.0);
        }
        let now = std::time::Instant::now();
        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        let elapsed = now.elapsed();
        assert_eq!(matrix.len(), 100, "The graph should have 100 nodes.");
        assert!(elapsed.as_secs_f64() < 1.0, "Function should be performant for large graphs.");
    }

    #[test]
    fn test_missing_weights_default_to_nan() {
        let mut graph = Graph::<&str, f64, Undirected>::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_node("C"); // C is isolated
        graph.add_edge(a, b, 2.5);

        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        assert!(matrix[0][2].is_nan());
        assert!(matrix[2][1].is_nan());
        assert!(matrix[2][2].is_nan());
    }

    #[test]
    fn test_negative_weights() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, -1.5);

        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        assert_eq!(matrix[0][1], -1.5);
    }

    #[test]
    fn test_zero_weights() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, 0.0);

        let matrix = graph_to_adjacency_matrix(&graph, true).unwrap();
        assert_eq!(matrix[0][1], 0.0);
    }

    #[test]
    fn test_empty_mnn_output() {
        let mnn_output = vec![];
        let graph = convert_to_graph(&mnn_output, None).unwrap();
        assert_eq!(graph.node_count(), 0, "Graph should have no nodes for empty input.");
        assert_eq!(graph.edge_count(), 0, "Graph should have no edges for empty input.");
    }

    #[test]
    fn test_simple_graph_conversion_with_distances() {
        let mnn_output = vec![vec![1], vec![0]];
        let distance_matrix = Some(vec![vec![0.0, 1.0], vec![1.0, 0.0]]);
        let graph = convert_to_graph(&mnn_output, distance_matrix.as_ref()).unwrap();
        assert_eq!(graph.node_count(), 2, "Graph should have 2 nodes.");
        assert_eq!(graph.edge_count(), 2, "Graph should have 2 edges for mutual neighbors.");
    }

    #[test]
    fn test_simple_graph_conversion_without_distances() {
        let mnn_output = vec![vec![1], vec![0]];
        let graph = convert_to_graph(&mnn_output, None).unwrap();
        assert_eq!(graph.node_count(), 2, "Graph should have 2 nodes.");
        assert_eq!(graph.edge_count(), 2, "Graph should have 2 edges for mutual neighbors, with default weights.");
    }

    #[test]
    fn test_graph_with_self_loops() {
        let mnn_output = vec![vec![0]]; // Node 0 is its own neighbor
        let distance_matrix = Some(vec![vec![0.0]]);
        let graph = convert_to_graph(&mnn_output, distance_matrix.as_ref()).unwrap();
        assert_eq!(graph.node_count(), 1, "Graph should have 1 node.");
        assert_eq!(graph.edge_count(), 0, "Self-loops are not expected to create edges.");
    }

    #[test]
    fn test_non_existent_neighbors() {
        let mnn_output = vec![vec![1], vec![2]]; // References to non-existent neighbors
        let distance_matrix = Some(vec![vec![0.0, 1.0]]); // Only one row in the matrix
        let graph = convert_to_graph(&mnn_output, distance_matrix.as_ref()).unwrap();
        // This test depends on how convert_to_graph handles non-existent rows in the distance matrix.
        assert_eq!(graph.node_count(), 2, "Graph should have 2 nodes despite referencing a non-existent neighbor.");
        assert_eq!(graph.edge_count(), 0, "Graph should have no edges due to non-existent neighbor distances.");
    }

    #[test]
    fn test_graph_conversion_with_missing_distances() {
        let mnn_output = vec![vec![1], vec![0]];
        // Distance between node 0 and 1 is missing
        let distance_matrix = Some(vec![vec![0.0], vec![0.0]]);
        let graph = convert_to_graph(&mnn_output, distance_matrix.as_ref()).unwrap();
        assert_eq!(graph.node_count(), 2, "Graph should have 2 nodes.");
        // This test assumes that missing distances result in no edge being added
        assert_eq!(graph.edge_count(), 0, "Graph should have no edges due to missing distances.");
    }

    #[test]
    fn two_node_graph_with_edge_false_nan() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, 2.5);
        let matrix = graph_to_adjacency_matrix(&graph, false).unwrap();
        assert_eq!(matrix[0][1], 2.5);
        assert_eq!(matrix[1][0], 2.5); // Mirror for undirected graph
        assert_eq!(matrix[0][0], 0.0, "No self-loop should result in 0.0");
        assert_eq!(matrix[1][1], 0.0, "No self-loop should result in 0.0");
    }

    #[test]
    fn graph_with_multiple_edges_false_nan() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        graph.add_edge(a, b, 3.0);
        graph.add_edge(a, c, 4.5);
        let matrix = graph_to_adjacency_matrix(&graph, false).unwrap();
        assert_eq!(matrix[0][1], 3.0);
        assert_eq!(matrix[0][2], 4.5);
        assert_eq!(matrix[1][0], 3.0, "Mirror edge should have the same weight for undirected graph");
        assert_eq!(matrix[2][0], 4.5, "Mirror edge should have the same weight for undirected graph");
    }

    #[test]
    fn non_existent_edge_false_nan() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_node("C"); // C has no edges
        graph.add_edge(a, b, 1.0);
        let matrix = graph_to_adjacency_matrix(&graph, false).unwrap();
        assert_eq!(matrix[2][0], 0.0, "Edges involving 'C' should be 0.0");
        assert_eq!(matrix[2][1], 0.0, "Edges involving 'C' should be 0.0");
    }

    #[test]
    fn graph_with_self_loops_false_nan() {
        let mut graph = Graph::new_undirected();
        let a = graph.add_node("A");
        graph.add_edge(a, a, 2.0); // Self-loop at A
        let matrix = graph_to_adjacency_matrix(&graph, false).unwrap();
        assert_eq!(matrix[0][0], 2.0, "Self-loop at 'A' should have a weight of 2.0");
    }

    #[test]
    fn test_missing_weights_default_to_zero() {
        let mut graph = Graph::<&str, f64, Undirected>::new_undirected();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_node("C"); // C is isolated
        graph.add_edge(a, b, 2.5);

        let matrix = graph_to_adjacency_matrix(&graph, false).unwrap();
        assert_eq!(matrix[0][2], 0.0, "Missing edge weights should default to 0.0");
        assert_eq!(matrix[2][1], 0.0, "Missing edge weights should default to 0.0");
        assert_eq!(matrix[2][2], 0.0, "Missing edge weights should default to 0.0");
    }
}