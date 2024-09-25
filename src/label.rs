
use csv::WriterBuilder;
use petgraph::graph::NodeIndex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use std::fs::File; 
use std::path::Path;
use crate::centrality::betweenness_centrality;
use crate::centrality::closeness_centrality;
use crate::centrality::degree_centrality;
use crate::centrality::NodeCentrality;
use crate::error::NetviewError;
use crate::netview::NetviewGraph;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Label {
    pub id: String,
    pub label: Option<String>,
}


pub fn read_labels_from_file<P: AsRef<Path>>(
    file_path: P,
    tsv: bool,
) -> Result<Vec<Label>, NetviewError> {
    let file = File::open(file_path)?;
    let mut rdr = if tsv {
        csv::ReaderBuilder::new().delimiter(b'\t').trim(csv::Trim::All).from_reader(file)
    } else {
        csv::ReaderBuilder::new().trim(csv::Trim::All).from_reader(file)
    };

    rdr.deserialize()
        .collect::<Result<Vec<Label>, csv::Error>>()
        .map_err(NetviewError::CsvError)
}


// Function to write the labels to a file
pub fn write_labels_to_file<P: AsRef<Path>>(
    graph: &NetviewGraph,        
    output_file: P,                
    tsv: bool                      
) -> Result<(), NetviewError> {

    // Open the output file for writing
    let file = File::create(output_file)?;
    
    // Use the csv::WriterBuilder to set the delimiter (tab for TSV, comma for CSV)
    let mut wtr = if tsv {
        WriterBuilder::new().delimiter(b'\t').from_writer(file)
    } else {
        WriterBuilder::new().from_writer(file)
    };

    // Iterate over each node in the graph and extract the id and label
    for node in graph.node_indices() {
        if let Some(node_label) = graph.node_weight(node) {

            // Construct a Label struct
            let label = Label {
                id: node_label.id.clone().unwrap_or_else(|| node.index().to_string()),
                label: node_label.label.clone(),
            };

            // Write the label to the file
            wtr.serialize(label)?;
        }
    }

    // Flush the writer to ensure all data is written to the file
    wtr.flush()?;

    Ok(())
}


pub fn label_nodes(graph: &mut NetviewGraph, labels: Vec<Option<String>>) -> Result<(), NetviewError> {
        
    // Check that the number of labels matches the number of nodes in the graph
    if labels.len() != graph.node_count() {
        return Err(NetviewError::NodeLabelLengthError(graph.node_count()));
    }

    // Iterate through the nodes and assign labels
    for (i, node) in graph.node_indices().enumerate() {

        // Get the corresponding label
        let label = &labels[i];

        // Mutably borrow the node's weight (NodeLabel) and update the label
        if let Some(node_weight) = graph.node_weight_mut(node) {
            node_weight.label = label.clone();
            
            log::debug!(
                "Node {} labeled with '{}'.",
                node.index(),
                label.clone().unwrap_or_else(|| "None".to_string())
            );

        } else {
            return Err(NetviewError::NodeNotFoundError(node.index()));
        }
    }

    Ok(())
}

// Function to propagate labels based on weighted voting using the node labels in the graph
pub fn label_propagation(
    graph: &mut NetviewGraph, 
    centrality_metric: NodeCentrality,
    max_iterations: usize,
    weight_ani: f64,
    weight_aai: f64,
    weight_af: f64,
    weight_centrality: f64,
    neighbor_centrality_vote: bool,
    distance_percent: bool,         // If distance weight in percent e.g. from skani, standardize to 0 - 1
    query_nodes: Option<&[usize]>,  // Optional subset of nodes by indices
    propagate_on_unlabeled: bool    // Whether to propagate only on nodes without a label (None)
) -> NetviewGraph {
    // Compute centrality using the previously defined function

    log::info!("Starting label propagation with at most {} iterations or stop if no label has changed.", max_iterations);
    log::info!("Weighting factors - ANI: {}, AAI: {}, AF: {}, Centrality: {}", weight_ani, weight_aai, weight_af, weight_centrality);

    log::info!("Computing centrality measure for all nodes in the graph ({centrality_metric}).");
    let centrality: HashMap<usize, f64> = match centrality_metric {
        NodeCentrality::Betweenness => betweenness_centrality(graph, true),
        NodeCentrality::Degree => degree_centrality(graph, true),
        NodeCentrality::Closeness => closeness_centrality(graph, true),
    };
        
    log::info!("Centrality computation completed.");


    // Generate the subset of nodes based on the input options
    let target_nodes: Vec<NodeIndex> = if propagate_on_unlabeled {
        // Get all nodes that do not have a label
        graph.node_indices()
                .filter(|node| graph.node_weight(*node).unwrap().label.is_none())
                .collect()
    } else if let Some(indices) = query_nodes {
        // Use the provided subset if propagate_on_unlabeled is false
        indices.iter().filter_map(|&idx| graph.node_indices().find(|&n| n.index() == idx)).collect()
    } else {
        // If no subset is provided, use all nodes
        graph.node_indices().collect()
    };

    log::info!("Labelling {} target nodes with label propagation algorithm", target_nodes.len());

    for iter in 0..max_iterations {
        log::info!("Starting iteration {} of label propagation.", iter + 1);
        let mut new_labels = HashMap::new();
        let mut label_changed = false;  // Track if any label changes

        // Loop through target (all, query) nodes in the graph
        for node in &target_nodes {

            let mut label_votes: HashMap<String, f64> = HashMap::new();
            let node_index = node.index();
            let node_centrality = centrality[&node_index];


            log::debug!(
                "Processing node with index {} and centrality score {:.4}.",
                node_index,
                node_centrality
            );

            // Loop through the neighbors of the current node
            for neighbor in graph.neighbors(*node) {

                let neighbor_label = graph.node_weight(neighbor).unwrap(); // Get the neighbor's NodeLabel

                if let Some(ref neighbor_label_value) = neighbor_label.label {

                    // Find the edge between the current node and the neighbor
                    let edge = graph.edge_weight(
                        graph.find_edge(*node, neighbor).expect("Failed to find edge between nodes - it should exist?")
                    ).unwrap();

                    let ani = edge.ani.unwrap_or(0.0) / 100.0;  // percent -> 0 - 1
                    let aai = edge.aai.unwrap_or(0.0) / 100.0;  // percent -> 0 - 1
                    let af = edge.af.unwrap_or(0.0) / 100.0;    // percent -> 0 - 1


                    let weight = if distance_percent {
                        1.0 - (edge.weight / 100.0)  // distance is percent -> similarity
                    } else {
                        1.0 - edge.weight            // distance -> similarity
                    };
                    
                    log::debug!(
                        "Neighbor (index: {}) has label '{}'. Edge weights: WEIGHT = {:.4}, ANI = {:.4}, AAI = {:.4}, AF = {:.4}, CENTR = {:.4}",
                        neighbor.index(),
                        neighbor_label_value,
                        weight,
                        ani,
                        aai,
                        af,
                        node_centrality
                    );

                    // Calculate the vote weight for the neighbor's label
                    let mut vote_weight = weight               // input similarity
                        + (weight_ani * ani)                        // ani distance if provided
                        + (weight_aai * aai)                        // aai distance if provided
                        + (weight_af * af)                          // alignment fraction if provided
                        + (weight_centrality * node_centrality);    // node centrality
                    
                    // Optionally include neighbor centrality in the vote
                    if neighbor_centrality_vote {
                        let neighbor_centrality = centrality[&neighbor.index()]; // Get neighbor's centrality
                        vote_weight += neighbor_centrality;

                        log::debug!(
                            "Including neighbor centrality in vote: neighbor_centrality = {:.4}, updated vote weight = {:.4}.",
                            neighbor_centrality,
                            vote_weight
                        );
                    }

                    *label_votes.entry(neighbor_label_value.clone()).or_insert(0.0) += vote_weight;
                }
            }

            // Select the label with the highest vote
            if let Some((best_label, highest_vote)) = label_votes.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
                log::debug!(
                    "Node {} will adopt label '{}' with vote weight {:.4}.",
                    node_index,
                    best_label,
                    highest_vote
                );
                new_labels.insert(node, best_label);
            }
        }

        // Update the labels for the next iteration directly on the graph
        for (node, new_label) in new_labels {
            log::debug!("Updating node {} with new label '{}'.", node.index(), new_label);
            if let Some(node_weight) = graph.node_weight_mut(*node) {
                node_weight.label = Some(new_label);  // Update the label directly in the graph's NodeLabel
                label_changed = true;                 // Track if a label changes
            }
        }

        if !label_changed {
            log::info!("Label propagation converged at iteration {}.", iter + 1);
            break;  // Early exit if no label changed
        }
    }

    log::info!("Returning graph with updated node labels from label propagation.");
    graph.clone()
}