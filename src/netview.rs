
use serde::{Deserialize, Serialize};
use petgraph::{Graph, Undirected};
use std::path::{Path, PathBuf};
use std::ops::{Add, Sub};
use std::cmp::Ordering;

use crate::centrality::NodeCentrality;
use crate::dist::{euclidean_distance_of_distances, parse_input_matrix, parse_identifiers, skani_distance_matrix};
use crate::mknn::{convert_to_graph, k_mutual_nearest_neighbors, GraphJson};
use crate::label::{label_nodes, label_propagation, write_labels_to_file};
use crate::error::NetviewError;

pub type NetviewGraph = Graph<NodeLabel, EdgeLabel, Undirected>;

pub struct Netview {
}

impl Netview {
    pub fn new() -> Self {
        Self {
        }
    }
    pub fn from_json(&self, path: &Path) -> Result<NetviewGraph, NetviewError> {
        Ok(GraphJson::read(path)?.into_graph())
    }
    pub fn skani_distance(
        &self,
        fasta: &PathBuf, 
        marker_compression_factor: usize, 
        compression_factor: usize, 
        threads: usize,
        min_percent_identity: f64,
        min_alignment_fraction: f64,
        small_genomes: bool
    ) -> Result<(Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<String>), NetviewError> {
        skani_distance_matrix(
            fasta,
            marker_compression_factor,
            compression_factor,
            threads,
            min_percent_identity,
            min_alignment_fraction,
            small_genomes
        )
    }
    pub fn graph_from_files(
        &self, 
        dist_matrix: &PathBuf, 
        k: usize, 
        af_matrix: Option<PathBuf>, 
        identifiers: Option<PathBuf>,
        is_csv: bool
    ) -> Result<NetviewGraph, NetviewError> {
        
        log::info!("Reading distance matrix: {}", dist_matrix.display());
        let distance = parse_input_matrix(dist_matrix, is_csv)?;

        let af = if let Some(path) = af_matrix {
            log::info!("Reading alignment fraction matrix: {}", path.display());
            Some(parse_input_matrix(&path, is_csv)?)
        } else {
            None
        };

        let ids = if let Some(path) = identifiers {
            log::info!("Reading identifier file: {}", path.display());
            Some(parse_identifiers(&path)?)
        } else {
            None
        };

        log::info!("Computing Euclidean distance abstraction...");
        let distance_of_distances = euclidean_distance_of_distances(
            &distance, 
            false, 
            false, 
            None
        )?;
        
        log::info!("Computing mutual nearest neighbor graph...");
        let mutual_nearest_neighbors = k_mutual_nearest_neighbors(
            &distance_of_distances, 
            k
        )?;

        let mknn_graph = convert_to_graph(
            &mutual_nearest_neighbors, 
            Some(&distance), 
            af.as_ref(),
            ids
        )?;       

        Ok(mknn_graph)
    }
    pub fn graph_from_vecs(
        &self, 
        dist_matrix: Vec<Vec<f64>>, 
        k: usize, 
        af_matrix: Option<Vec<Vec<f64>>>,
        ids: Option<Vec<String>>
    ) -> Result<NetviewGraph, NetviewError> {
        
        log::info!("Computing Euclidean distance abstraction...");
        let distance_of_distances = euclidean_distance_of_distances(
            &dist_matrix, 
            false, 
            false, 
            None
        )?;

        log::info!("Computing mutual nearest neighbor graph...");
        let mutual_nearest_neighbors = k_mutual_nearest_neighbors(
            &distance_of_distances, 
            k
        )?;

        let mknn_graph = convert_to_graph(
            &mutual_nearest_neighbors, 
            Some(&dist_matrix), 
            af_matrix.as_ref(),
            ids
        )?;

        Ok(mknn_graph)
    }
    pub fn label_nodes(&self, graph: &mut NetviewGraph, labels: Vec<Option<String>>) -> Result<(), NetviewError> {
        log::info!("Labelling nodes on graph (n = {})", labels.len());
        label_nodes(graph, labels)
    }
    pub fn write_labels(&self, graph: &NetviewGraph, path: &Path) -> Result<(), NetviewError> {
        log::info!("Writing graph labels to file: {}", path.display());
        write_labels_to_file(&graph, path, false)
    }
    pub fn label_propagation(
        &self,
        graph: &mut NetviewGraph,
        centrality_metric: NodeCentrality,
        max_iterations: usize,
        weight_ani: f64,
        weight_aai: f64,
        weight_af: f64,
        weight_centrality: f64,
        neighbor_centrality_vote: bool,
        scale_weight: bool,             // If distance weight in percent scale to 0 - 1
        query_nodes: Option<&[usize]>,  // Optional subset of nodes by indices
        propagate_on_unlabeled: bool    // Whether to propagate only on nodes without a label (None)
    ) -> NetviewGraph {

        label_propagation(
            graph, 
            centrality_metric,
            max_iterations, 
            weight_ani, 
            weight_aai, 
            weight_af,
            weight_centrality, 
            neighbor_centrality_vote,
            scale_weight,
            query_nodes,
            propagate_on_unlabeled
        )
    }
}

/* Netview graph nodes and edges with associated 
   metadata for downstream applications 
*/

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeLabel {
    pub index: usize,                    // Original index of sample or sequence in the input matrix
    pub id: Option<String>,              // Node identifier e.g. sample or sequence identifier from input matrix
    pub label: Option<String>,           // Label added or inferred downstream
    pub label_confidence: f64,           // Confidence in the label (0.0 to 1.0) computed downstream
}

impl NodeLabel {
    // Builder pattern for NodeLabel
    pub fn builder(index: usize) -> NodeLabelBuilder {
        NodeLabelBuilder {
            index,
            id: None,
            label: None,
            label_confidence: 0.0,
        }
    }
    pub fn new(index: usize, id: Option<String>) -> Self {
        Self {
            index,
            id,
            label: None,
            label_confidence: 0.0
        }
    }
}

pub struct NodeLabelBuilder {
    index: usize,
    id: Option<String>,
    label: Option<String>,
    label_confidence: f64,
}

impl NodeLabelBuilder {
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn label_confidence(mut self, confidence: f64) -> Self {
        self.label_confidence = confidence;
        self
    }

    pub fn build(self) -> NodeLabel {
        NodeLabel {
            id: self.id,
            index: self.index,
            label: self.label,
            label_confidence: self.label_confidence,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct EdgeLabel {
    pub index: usize,              // Original edge index
    pub source: usize,             // Node source index
    pub target: usize,             // Node target index
    pub weight: f64,               // Original distance from the distance matrix as weight
    pub ani: Option<f64>,          // ANI similarity score (optional, not used for now)
    pub aai: Option<f64>,          // AAI similarity score (optional, not used for now)
    pub af: Option<f64>,           // Alignment fraction (AF), will be filled from af_matrix if present
}

impl EdgeLabel {
    // Builder pattern for EdgeLabel
    pub fn builder(index: usize, source: usize, target: usize, weight: f64) -> EdgeLabelBuilder {
        EdgeLabelBuilder {
            index,
            source,
            target,
            weight,
            ani: None,
            aai: None,
            af: None,
        }
    }

    pub fn new(index: usize, source: usize, target: usize, weight: f64, af: Option<f64>) -> Self {
        Self {
            index, 
            source,
            target,
            weight,
            af,
            ani: None,
            aai: None,
        }
    }
}

pub struct EdgeLabelBuilder {
    index: usize,
    source: usize,
    target: usize,
    weight: f64,
    ani: Option<f64>,
    aai: Option<f64>,
    af: Option<f64>,
}

impl EdgeLabelBuilder {
    pub fn ani(mut self, ani: f64) -> Self {
        self.ani = Some(ani);
        self
    }

    pub fn aai(mut self, aai: f64) -> Self {
        self.aai = Some(aai);
        self
    }

    pub fn af(mut self, af: f64) -> Self {
        self.af = Some(af);
        self
    }

    pub fn build(self) -> EdgeLabel {
        EdgeLabel {
            index: self.index,
            source: self.source,
            target: self.target,
            weight: self.weight,
            ani: self.ani,
            aai: self.aai,
            af: self.af,
        }
    }
}


impl Add for EdgeLabel {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        EdgeLabel {
            weight: self.weight + other.weight,
            ..self
        }
    }
}

impl Sub for EdgeLabel {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        EdgeLabel {
            weight: self.weight - other.weight,
            ..self
        }
    }
}

// Implementing the trait Measure for Dijkstra's algorithm
impl Ord for EdgeLabel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for EdgeLabel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl Eq for EdgeLabel {}

impl PartialEq for EdgeLabel {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

// Implement default measure behavior
impl Default for EdgeLabel {
    fn default() -> Self {
        EdgeLabel {
            index: 0,
            source: 0,
            target: 0,
            weight: 0.0,
            ani: None,
            aai: None,
            af: None,
        }
    }
}