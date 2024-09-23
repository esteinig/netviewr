use std::path::PathBuf;

use petgraph::{Graph, Undirected};
use serde::{Deserialize, Serialize};

use crate::{dist::{euclidean_distance_of_distances, parse_input_matrix}, error::NetviewError, mknn::{convert_to_graph, k_mutual_nearest_neighbors}};

pub struct Netview {
}

impl Netview {
    pub fn new() -> Self {
        Self { }
    }
    pub fn create_graph(&self, distance_matrix: &PathBuf, k: usize, af_matrix: Option<PathBuf>, tsv: bool) -> Result<Graph<NodeLabel, EdgeLabel, Undirected>, NetviewError> {
        
        log::info!("Parsing distance matrix: {}", distance_matrix.display());
        let distance = parse_input_matrix(&distance_matrix, tsv)?;

        let af = if let Some(path) = af_matrix {
            log::info!("Parsing alignment fraction matrix: {}", path.display());
            Some(parse_input_matrix(&path, tsv)?)
        } else {
            None
        };

        log::info!("Computing Euclidean distance abstraction...");
        let distance_of_distances = euclidean_distance_of_distances(&distance, false, false, None)?;
        
        log::info!("Computing mutual nearest neighbor graph...");
        let mutual_nearest_neighbors = k_mutual_nearest_neighbors(&distance_of_distances, k)?;

        let mknn_graph = convert_to_graph(
            &mutual_nearest_neighbors, 
            Some(&distance), 
            af.as_ref()
        )?;       

        Ok(mknn_graph)
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeLabel {
    pub index: usize,                   // Original dataset index
    pub label: Option<String>,           // Label, could be inferred later
    pub label_confidence: f64,           // Confidence in the label (0.0 to 1.0)
}

impl NodeLabel {
    // Builder pattern for NodeLabel
    pub fn builder(index: usize) -> NodeLabelBuilder {
        NodeLabelBuilder {
            index,
            label: None,
            label_confidence: 0.0,
        }
    }
    pub fn new(index: usize) -> Self {
        Self {
            index,
            label: None,
            label_confidence: 0.0
        }
    }
}

pub struct NodeLabelBuilder {
    index: usize,
    label: Option<String>,
    label_confidence: f64,
}

impl NodeLabelBuilder {
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
            index: self.index,
            label: self.label,
            label_confidence: self.label_confidence,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EdgeLabel {
    pub index: usize,              // Original edge index
    pub weight: f64,               // Original distance from the distance matrix as weight
    pub ani: Option<f64>,          // ANI similarity score (optional, not used for now)
    pub aai: Option<f64>,          // AAI similarity score (optional, not used for now)
    pub af: Option<f64>,           // Alignment fraction (AF), will be filled from af_matrix if present
}

impl EdgeLabel {
    // Builder pattern for EdgeLabel
    pub fn builder(index: usize, dist: f64) -> EdgeLabelBuilder {
        EdgeLabelBuilder {
            index,
            dist,
            ani: None,
            aai: None,
            af: None,
        }
    }

    pub fn new(index: usize, dist: f64, af: Option<f64>) -> Self {
        Self {
            index, 
            weight: dist,
            af,
            ani: None,
            aai: None,
        }
    }
}

pub struct EdgeLabelBuilder {
    index: usize,
    dist: f64,
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
            weight: self.dist,
            ani: self.ani,
            aai: self.aai,
            af: self.af,
        }
    }
}