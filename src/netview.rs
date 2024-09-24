use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use petgraph::{Graph, Undirected};

use crate::dist::{euclidean_distance_of_distances, parse_input_matrix, parse_identifiers, skani_distance_matrix};
use crate::mknn::{convert_to_graph, k_mutual_nearest_neighbors, GraphJson};
use crate::error::NetviewError;

pub type NetviewGraph = Graph<NodeLabel, EdgeLabel, Undirected>;

pub trait NetviewLabels {
    fn label_nodes(&self, labels: Vec<Option<String>>) -> Result<(), NetviewError>;
}

impl NetviewLabels for NetviewGraph {
    fn label_nodes(&self, labels: Vec<Option<String>>) -> Result<(), NetviewError> {
        
        log::info!("Labelling nodes on NetviewGraph");
        

        Ok(())
    }
}


pub struct Netview {
    pub graph: NetviewGraph
}

impl Netview {
    pub fn new() -> Self {
        Self {
            graph: NetviewGraph::new_undirected()
        }
    }
    pub fn from_json(path: &Path) -> Result<Self, NetviewError> {
        Ok(Self {
            graph: GraphJson::read(path)?.into_graph()
        })
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

    /// Main version of the graph method that parses the distance and alignment fraction matrices from file paths.
    pub fn graph_from_files(
        &self, 
        dist_matrix: &PathBuf, 
        k: usize, 
        af_matrix: Option<PathBuf>, 
        identifiers: Option<PathBuf>,
        is_csv: bool
    ) -> Result<Graph<NodeLabel, EdgeLabel, Undirected>, NetviewError> {
        
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

    /// Alternative version of the graph method that takes the distance and alignment fraction matrices directly.
    pub fn graph_from_vecs(
        &self, 
        dist_matrix: Vec<Vec<f64>>, 
        k: usize, 
        af_matrix: Option<Vec<Vec<f64>>>,
        ids: Option<Vec<String>>
    ) -> Result<Graph<NodeLabel, EdgeLabel, Undirected>, NetviewError> {
        
        log::info!("Received distance matrix directly.");

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

#[derive(Serialize, Deserialize, Clone, Debug)]
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