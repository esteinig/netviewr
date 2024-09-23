use rayon::prelude::*;
use petgraph::{dot::Dot, Graph, Undirected};
use csv::WriterBuilder;
use serde_json;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufRead};
use std::{fs::File, io::Write, path::Path};
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use core::f64::NAN;
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};

use crate::dist::make_symmetrical;
use crate::error::NetviewError;
use crate::netview::{EdgeLabel, NodeLabel};


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


// Function to convert mutual nearest neighbors to a graph with NodeLabel and EdgeLabel
pub fn convert_to_graph(
    mutual_nearest_neighbors: &Vec<Vec<usize>>, 
    distance_matrix: Option<&Vec<Vec<f64>>>,  // Distance matrix
    af_matrix: Option<&Vec<Vec<f64>>>,        // Alignment fraction matrix
) -> Result<Graph<NodeLabel, EdgeLabel, Undirected>, NetviewError> {
    
    // Create an undirected graph with NodeLabel and EdgeLabel
    let mut graph = Graph::<NodeLabel, EdgeLabel, Undirected>::new_undirected();

    // Maps to store node indices and avoid duplicate edges
    let mut index_map: HashMap<usize, NodeIndex> = HashMap::new();
    let mut edge_set: HashSet<(usize, usize)> = HashSet::new();  // Set to track added edges
    let mut edge_index = 0;  // Track the edge index

    // Add all nodes to the graph as NodeLabels
    for (node_index, _) in mutual_nearest_neighbors.iter().enumerate() {
        let node_label = NodeLabel::new(node_index);  // Create NodeLabel with index
        let graph_node_index = graph.add_node(node_label);  // Add NodeLabel to the graph
        index_map.insert(node_index, graph_node_index);
    }

    // Add edges between mutual nearest neighbors, ensuring no duplicate edges
    for (node_index, neighbors) in mutual_nearest_neighbors.iter().enumerate() {
        let graph_node_index = *index_map.get(&node_index).ok_or(NetviewError::NodeIndexError)?;

        for &neighbor in neighbors.iter() {
            // Ensure edges are added only once
            let edge = if node_index < neighbor {
                (node_index, neighbor)
            } else {
                (neighbor, node_index)
            };

            if !edge_set.contains(&edge) {
                // Get the distance from the distance matrix, if provided
                let dist = match distance_matrix {
                    Some(matrix) => matrix.get(node_index).and_then(|row| row.get(neighbor)).copied().unwrap_or(1.0),
                    None => 1.0,  // Default weight if no distance matrix is provided
                };

                // Get the alignment fraction from the af_matrix, if provided
                let af = match af_matrix {
                    Some(matrix) => matrix.get(node_index).and_then(|row| row.get(neighbor)).copied(),
                    None => None,  // Default to None if no af_matrix is provided
                };

                // Create the edge label with the index, distance, and af (alignment fraction)
                let edge_label = EdgeLabel::new(edge_index, dist, af);

                let graph_neighbor_index = *index_map.get(&neighbor).ok_or(NetviewError::NodeIndexError)?;
                graph.add_edge(graph_node_index, graph_neighbor_index, edge_label);

                // Mark this edge as added and increment the edge index
                edge_set.insert(edge);
                edge_index += 1;
            }
        }
    }

    Ok(graph)
}



#[derive(Serialize, Deserialize, Clone, Debug, clap::ValueEnum)]
pub enum GraphFormat {
    Dot,
    Json,
    Adjacency,
    Edges,
}



/// Writes a `petgraph::Graph` to a file in specified formats (DOT, JSON, Adjacency Matrix, or Edges List).
///
/// This function supports exporting the graph to various formats for visualization
/// or further processing. Currently supported formats are:
/// - **DOT**: For use with Graphviz for visualizing the graph.
/// - **JSON**: For generic data interchange, representing nodes and edges as JSON objects.
/// - **Adjacency Matrix**: Outputs the adjacency matrix representation of the graph in TSV format.
/// - **Edges**: Outputs an edge list with source, target, and optional weights.
///
/// # Arguments
/// * `graph`  - Reference to the graph to be written.
/// * `path`   - Path to the output file where the graph should be written.
/// * `format` - Specifies the output format ("dot", "json", "adjmatrix", or "edges").
/// * `include_weights` - Whether to include edge weights in the output (relevant for some formats).
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(NetviewError)` on failure, with detailed error information.
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
/// write_graph_to_file(&graph, Path::new("graph.dot"), "dot", false).unwrap();
/// write_graph_to_file(&graph, Path::new("graph.json"), "json", true).unwrap();
/// write_graph_to_file(&graph, Path::new("graph.tsv"), "adjmatrix", false).unwrap();
/// write_graph_to_file(&graph, Path::new("graph_edges.txt"), "edges", true).unwrap();
/// ```
pub fn write_graph_to_file(
    graph: &Graph<NodeLabel, EdgeLabel, Undirected>,
    path: &Path,
    format: &GraphFormat,
    include_weights: bool
) -> Result<(), NetviewError>
where
    NodeLabel: Serialize + std::fmt::Debug,
    EdgeLabel: Serialize + std::clone::Clone + std::fmt::Debug,
{
    let mut file = File::create(path).map_err(|e| NetviewError::GraphFileError(e.to_string()))?;

    match format {
        GraphFormat::Dot => {
            let dot = Dot::with_config(graph, &[petgraph::dot::Config::EdgeNoLabel]);
            write!(file, "{:?}", dot).map_err(|e| NetviewError::GraphFileError(e.to_string()))?;
        },
        GraphFormat::Json => {
            write_json_graph(graph, path)?;
        },
        GraphFormat::Adjacency => {
            let adj_matrix = graph_to_adjacency_matrix(graph, false)?;
            write_adjacency_matrix_to_file(&adj_matrix, path)?;   
        },
        GraphFormat::Edges => {
            let edgelist = graph_to_edgelist(graph);
            write_edgelist_to_file(&edgelist, path, include_weights)?;
        }
    }

    Ok(())
}


/// Writes a `petgraph::Graph` with `NodeLabel` and `EdgeLabel` to a JSON file.
///
/// This function uses the `serde`-derived `Serialize` implementation of `NodeLabel` and `EdgeLabel`
/// to directly serialize the graph nodes and edges into JSON format.
///
/// # Arguments
/// * `graph`          - Reference to the graph to be serialized.
/// * `path`           - Path to the output file where the JSON should be written.
/// * `include_weights` - Whether to include weights (distance) in the edge list.
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(NetviewError)` on failure, with detailed error information.
pub fn write_json_graph<NodeLabel, EdgeLabel>(
    graph: &Graph<NodeLabel, EdgeLabel, Undirected>,
    path: &std::path::Path,
) -> Result<(), NetviewError>
where
    NodeLabel: Serialize + std::fmt::Debug,
    EdgeLabel: Serialize + std::fmt::Debug,
{
    let mut file = File::create(path).map_err(|e| NetviewError::GraphFileError(e.to_string()))?;

    // Serialize nodes and edges
    let mut nodes = vec![];
    let mut edges = vec![];

    // Serialize the nodes
    for (_, node_label) in graph.node_references() {
        nodes.push(node_label);
    }

    // Serialize the edges
    for edge in graph.edge_references() {
       edges.push(edge.weight());
    }

    // Create a JSON object with nodes and edges
    let graph_json = serde_json::json!({
        "nodes": nodes,
        "edges": edges
    });

    // Write the serialized JSON to the file
    serde_json::to_writer_pretty(&mut file, &graph_json)
        .map_err(|e| NetviewError::GraphFileError(e.to_string()))?;

    Ok(())
}

/// Reads an edge list from a file and constructs a petgraph Graph.
///
/// # Arguments
///
/// * `filename` - Path to the edge list file.
/// * `has_weights` - If `true`, the third column of each row is treated as the edge weight.
///                   If `false`, edges will have a default weight of 1.0.
///
/// # Returns
///
/// A `Result` containing a `Graph<usize, f64, Undirected>`, where nodes are indexed by their positions in the edge list.
///
/// # Example
///
/// ```rust
/// let graph = read_edgelist("edgelist.tsv", true).unwrap();
/// ```
pub fn read_edgelist(filename: &Path, has_weights: bool) -> Result<Graph<usize, f64, Undirected>, NetviewError> {
    // Create a new undirected graph
    let mut graph = Graph::<usize, f64, Undirected>::new_undirected();
    
    // A map to keep track of nodes and their corresponding NodeIndex in the graph
    let mut node_map: HashMap<usize, NodeIndex> = HashMap::new();

    // Open the edge list file
    let file = File::open(filename).map_err(|_| NetviewError::GraphDeserializationError(filename.to_string_lossy().to_string()))?;
    let reader = BufReader::new(file);

    // Read the file line by line
    for line in reader.lines() {
        let line = line?;

        // Split the line by tab
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue; // Invalid line, skip
        }

        // Parse the node indices (assuming nodes are represented by usize)
        let node1: usize = parts[0].parse().unwrap();
        let node2: usize = parts[1].parse().unwrap();

        // Parse the weight if `has_weights` is true, otherwise assign 1.0 as the default weight
        let weight: f64 = if has_weights && parts.len() == 3 {
            parts[2].parse().unwrap_or(1.0)
        } else {
            1.0
        };

        // Get or create NodeIndex for node1
        let node1_index = *node_map.entry(node1).or_insert_with(|| graph.add_node(node1));
        // Get or create NodeIndex for node2
        let node2_index = *node_map.entry(node2).or_insert_with(|| graph.add_node(node2));

        // Add the edge between node1 and node2 with the weight
        graph.add_edge(node1_index, node2_index, weight);
    }

    Ok(graph)
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


/// Writes the edge list to a file.
///
/// # Arguments
///
/// * `edgelist` - A `Vec<(usize, usize, f64)>` representing the edges (source, target, weight).
/// * `filename` - The file path where the edge list will be written.
/// * `include_weights` - If `true`, includes the weights in the output; if `false`, only writes the source and target.
///
/// # Returns
///
/// A `Result<(), NetviewError>` indicating success or failure.
///
/// # Example
///
/// ```
/// let edgelist = vec![(0, 1, 1.5), (1, 2, 2.5)];
/// write_edgelist_to_file(&edgelist, "edgelist.txt", true).expect("Failed to write file");
/// ```
pub fn write_edgelist_to_file(edgelist: &Vec<(usize, usize, f64)>, filename: &Path, include_weights: bool) -> Result<(), NetviewError> {
    // Step 1: Create or open the file
    let mut file = File::create(filename).map_err(|e| NetviewError::GraphFileError(e.to_string()))?;

    // Step 2: Iterate over the edge list and write each edge to the file
    for (source, target, weight) in edgelist {
        if include_weights {
            // Write the edge as "source target weight\n"
            writeln!(file, "{} {} {}", source, target, weight).map_err(|e| NetviewError::GraphSerializationError(e.to_string()))?;
        } else {
            // Write the edge as "source target\n" (no weight)
            writeln!(file, "{} {}", source, target).map_err(|e| NetviewError::GraphSerializationError(e.to_string()))?;
        }
    }

    // Return Ok(()) if everything succeeded
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
pub fn graph_to_adjacency_matrix(graph: &Graph<NodeLabel, EdgeLabel, Undirected>, nan: bool) -> Result<Vec<Vec<f64>>, NetviewError>
{
    let node_count = graph.node_count();
    let mut matrix = vec![vec![match nan { true => NAN, false => 0.}; node_count]; node_count];

    for edge_ref in graph.edge_references() {
        let (source, target) = (edge_ref.source().index(), edge_ref.target().index());

        let weight = edge_ref.weight().weight;

        matrix[source][target] = weight;
        // Since the graph is undirected, mirror the weights
        matrix[target][source] = weight;
    }

    Ok(matrix)
}
    

/// Converts a `Graph<N, E, Undirected>` into an edge list compatible with igraph.
///
/// # Arguments
///
/// * `graph` - Input `petgraph::Graph` to be converted.
///
/// # Returns
///
/// A `Vec<(usize, usize, f64)>` representing the edges, where each tuple contains the source
/// node index, target node index, and edge weight.
///
/// # Examples
///
/// ```
/// use petgraph::Graph;
/// use petgraph::Undirected;
/// use netview::graph_to_edgelist;
///
/// let mut graph = Graph::<&str, f64, Undirected>::new_undirected();
/// let a = graph.add_node("A");
/// let b = graph.add_node("B");
/// graph.add_edge(a, b, 1.5);
///
/// let edgelist = graph_to_edgelist(&graph);
/// assert_eq!(edgelist, vec![(0, 1, 1.5)]);
/// ```
pub fn graph_to_edgelist(graph: &Graph<NodeLabel, EdgeLabel, Undirected>) -> Vec<(usize, usize, f64)>
{
    let mut edgelist = Vec::new();

    // Iterate through each edge in the graph
    for edge_ref in graph.edge_references() {
        let source = edge_ref.source().index();
        let target = edge_ref.target().index();

        // Safely attempt conversion of E to f64
        let weight: f64 = edge_ref.weight().weight;

        // Add the edge to the edge list (igraph treats undirected edges as a single pair)
        edgelist.push((source, target, weight));
    }

    edgelist
}


#[cfg(test)]
mod tests {

    use super::*;

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

}