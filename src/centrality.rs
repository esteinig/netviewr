
use std::collections::HashMap;

use crate::netview::NetviewGraph;
use petgraph::algo::dijkstra;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, clap::ValueEnum)]
pub enum NodeCentrality {
    Betweenness,
    Degree,
    Closeness
}
impl std::fmt::Display for NodeCentrality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            NodeCentrality::Betweenness => "betweenness centrality",
            NodeCentrality::Degree => "degree centrality",
            NodeCentrality::Closeness => "closeness centrality",
        };
        write!(f, "{}", output)
    }
}

pub fn standardize_centrality(centrality: &mut HashMap<usize, f64>) {
    if centrality.is_empty() {
        return;
    }

    let (min_val, max_val) = centrality.values().fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &val| {
        (min.min(val), max.max(val))
    });

    if max_val > min_val {
        for value in centrality.values_mut() {
            *value = (*value - min_val) / (max_val - min_val);
        }
    }
}

// Function to compute betweenness centrality
pub fn betweenness_centrality(graph: &NetviewGraph, standardized: bool) -> HashMap<usize, f64>
{
    let mut centrality: HashMap<usize, f64> = HashMap::new();

    // Initialize centrality scores to 0
    for node in graph.node_indices() {
        centrality.insert(node.index(), 0.0);
    }

    // Compute the shortest paths between all pairs of nodes
    for source in graph.node_indices() {
        // Perform Dijkstra's algorithm to find shortest paths from the source node
        let shortest_paths = dijkstra(&graph, source, None, |edge| *edge.weight());

        // Iterate over each target node and accumulate centrality scores
        for (target, _) in &shortest_paths {
            if source != *target {
                // Find all nodes that lie on the shortest path between source and target
                let mut predecessors = vec![*target];
                while let Some(&predecessor) = predecessors.last() {
                    if predecessor == source {
                        break;
                    }

                    // Update centrality score for each node on the path
                    predecessors.push(predecessor);
                    *centrality.get_mut(&predecessor.index()).unwrap() += 1.0;
                }
            }
        }
    }
    
    if standardized {
        standardize_centrality(&mut centrality);
    }

    centrality
}

// Function to compute degree centrality
pub fn degree_centrality(graph: &NetviewGraph, standardized: bool) -> HashMap<usize, f64> {
    let mut centrality = HashMap::new();

    // Loop through all nodes in the graph
    for node in graph.node_indices() {
        // Degree centrality is simply the number of neighbors
        let degree = graph.neighbors(node).count() as f64;
        centrality.insert(node.index(), degree);
    }

    if standardized {
        standardize_centrality(&mut centrality);
    }

    centrality
}

pub fn closeness_centrality(graph: &NetviewGraph, standardized: bool) -> HashMap<usize, f64> {
    let mut centrality = HashMap::new();

    // Loop through all nodes in the graph
    for node in graph.node_indices() {
        // Perform Dijkstra's algorithm to find shortest paths from the current node
        let shortest_paths = dijkstra(graph, node, None, |edge| *edge.weight());

        // Calculate the sum of distances to all reachable nodes
        let total_distance: f64 = shortest_paths.values().map(|e| e.weight).sum();

        // Avoid division by zero by checking if total_distance > 0
        if total_distance > 0.0 {
            let closeness = (shortest_paths.len() as f64 - 1.0) / total_distance;
            centrality.insert(node.index(), closeness);
        } else {
            centrality.insert(node.index(), 0.0); // If the node is isolated
        }
    }

    if standardized {
        standardize_centrality(&mut centrality);
    }

    centrality
}

pub fn eigenvector_centrality(graph: &NetviewGraph, iterations: usize, tolerance: f64, standardized: bool) -> HashMap<usize, f64> {
    let mut centrality: HashMap<usize, f64> = HashMap::new();
    
    // Initialize centrality scores to 1
    for node in graph.node_indices() {
        centrality.insert(node.index(), 1.0);
    }

    for _ in 0..iterations {
        let mut new_centrality = HashMap::new();
        let mut max_centrality = 0.0;

        // Update centrality based on the neighbors' centralities
        for node in graph.node_indices() {
            let mut score = 0.0;

            for neighbor in graph.neighbors(node) {
                score += centrality[&neighbor.index()];
            }

            new_centrality.insert(node.index(), score);
            if score > max_centrality {
                max_centrality = score;
            }
        }

        // Normalize the scores
        for (_, score) in new_centrality.iter_mut() {
            *score /= max_centrality;
        }

        // Check for convergence (if the changes are small enough)
        let converged = new_centrality.iter().all(|(node, new_score)| {
            (new_score - centrality[node]).abs() < tolerance
        });

        centrality = new_centrality;

        if converged {
            break;
        }
    }

    if standardized {
        standardize_centrality(&mut centrality);
    }

    centrality
}

pub fn pagerank(graph: &NetviewGraph, iterations: usize, damping_factor: f64, standardized: bool) -> HashMap<usize, f64> {
    let node_count = graph.node_count();
    let mut centrality: HashMap<usize, f64> = HashMap::new();
    let initial_rank = 1.0 / node_count as f64;

    // Initialize all nodes with equal centrality
    for node in graph.node_indices() {
        centrality.insert(node.index(), initial_rank);
    }

    for _ in 0..iterations {
        let mut new_centrality = HashMap::new();

        // Distribute centrality based on neighbors
        for node in graph.node_indices() {
            let mut score = (1.0 - damping_factor) / node_count as f64;

            for neighbor in graph.neighbors(node) {
                let neighbor_degree = graph.neighbors(neighbor).count() as f64;
                score += damping_factor * centrality[&neighbor.index()] / neighbor_degree;
            }

            new_centrality.insert(node.index(), score);
        }

        centrality = new_centrality;
    }

    if standardized {
        standardize_centrality(&mut centrality);
    }

    centrality
}