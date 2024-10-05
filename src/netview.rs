
use serde::{Deserialize, Serialize};
use petgraph::{Graph, Undirected};
use std::path::{Path, PathBuf};
use std::ops::{Add, Sub};
use std::cmp::Ordering;

use crate::centrality::NodeCentrality;
use crate::config::NetviewConfig;
use crate::dist::{euclidean_distance_of_distances, parse_identifiers, parse_input_matrix, skani_distance_matrix, write_ids, write_matrix_to_file};
use crate::mknn::{convert_to_graph, k_mutual_nearest_neighbors, write_graph_to_file, GraphFormat, GraphJson};
use crate::label::{label_nodes, label_propagation, read_labels_from_file, write_graph_labels_to_file, VoteWeights};
use crate::error::NetviewError;
use crate::utils::{concatenate_fasta_files, get_ids_from_fasta_files};

pub type NetviewGraph = Graph<NodeLabel, EdgeLabel, Undirected>;

pub struct Netview {
    config: NetviewConfig
}

pub struct NetviewPredictFiles {
    data: PathBuf,
    label: PathBuf,
    dist: PathBuf,
    af: PathBuf,
    id: PathBuf,
    graph_json: PathBuf,
    graph_edges: PathBuf,
    graph_predict: PathBuf,
    label_predict: PathBuf,
}
impl NetviewPredictFiles {
    fn from(outdir: &PathBuf, name: String) -> Self {
        Self {
            data: outdir.join(format!("{name}.fasta")),
            label: outdir.join(format!("{name}.csv")),
            dist: outdir.join(format!("{name}.dist")),
            af: outdir.join(format!("{name}.af")),
            id: outdir.join(format!("{name}.id")),
            graph_json: outdir.join(format!("{name}.json")),
            graph_edges: outdir.join(format!("{name}.edges")),
            graph_predict: outdir.join(format!("{name}.predict.json")),
            label_predict: outdir.join(format!("{name}.predict.csv")),
        }
    }
}

impl Netview {
    pub fn new(config: NetviewConfig) -> Self {
        Self { config }
    }
    pub fn read_json_graph(&self, path: &Path) -> Result<NetviewGraph, NetviewError> {
        Ok(GraphJson::read(path)?.into_graph())
    }
    pub fn predict(
        &self, 
        fasta: &Vec<PathBuf>, 
        db: &PathBuf, 
        labels: &PathBuf, 
        k: usize, 
        outdir: &PathBuf,
        propagate_all: bool,
        basename: String,
        threads: usize
    ) -> Result<(), NetviewError> {
        
        if !outdir.exists() {
            std::fs::create_dir_all(&outdir)?;
        }

        let files = NetviewPredictFiles::from(outdir, basename);
        let fasta_ids = get_ids_from_fasta_files(&fasta)?; // seq ids for prediction
       
        concatenate_fasta_files(db, fasta, &files.data)?;

        let (dist, af, ids) = self.skani_distance(
            &files.data,
            self.config.skani.marker_compression_factor,
            self.config.skani.compression_factor,
            threads,
            self.config.skani.min_percent_identity,
            self.config.skani.min_alignment_fraction,
            self.config.skani.small_genomes
        )?;

        write_matrix_to_file(&dist, &files.dist)?;
        write_matrix_to_file(&af, &files.af)?;
        write_ids(&ids, &files.id)?;

        let mut graph = self.graph_from_vecs(
            dist, k, Some(af), Some(ids)
        )?;

        let db_labels: Vec<Option<String>> = read_labels_from_file(&labels, false)?
            .into_iter()
            .map(|g| g.label)
            .collect();

        // Add unknowns to labels for prediction, this is a bit hacky right now...
        let mut labels = db_labels.clone();
        for _ in &fasta_ids { labels.push(None) };

        self.label_nodes(&mut graph, labels)?;
        self.write_labels(&graph, &files.label)?;

        write_graph_to_file(&graph, &files.graph_json, &GraphFormat::Json, true)?;
        write_graph_to_file(&graph, &files.graph_edges, &GraphFormat::Edges, false)?;

        self.label_propagation(
            &mut graph,
            self.config.label.centrality_metric.clone(), 
            self.config.label.max_iterations, 
            self.config.label.vote_weights.clone(),
            self.config.label.neighbor_centrality_vote, 
            true, 
            if propagate_all { None } else { Some(fasta_ids) }, 
            false
        );

        write_graph_to_file(&graph, &files.graph_predict, &GraphFormat::Json, true)?;
        self.write_labels(&graph, &files.label_predict)?;

        Ok(())

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

        log::info!("Computing Euclidean distance abstraction matrix");
        let distance_of_distances = euclidean_distance_of_distances(
            &distance, 
            false, 
            false, 
            None
        )?;
        
        log::info!("Computing mutual nearest neighbor graph (k = {k})");
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
        

        log::info!("Computing Euclidean distance abstraction matrix");
        let distance_of_distances = euclidean_distance_of_distances(
            &dist_matrix, 
            false, 
            false, 
            None
        )?;
        
        log::info!("Computing mutual nearest neighbor graph (k = {k})");
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
    pub fn label_propagation(
        &self,
        graph: &mut NetviewGraph,
        centrality_metric: NodeCentrality,
        max_iterations: usize,
        vote_weights: VoteWeights,
        neighbor_centrality_vote: bool,
        distance_percent: bool,             // If distance weight in percent scale to 0 - 1
        query_nodes: Option<Vec<String>>,   // Optional subset of nodes by identifiers
        propagate_on_unlabeled: bool        // Whether to propagate only on nodes without a label (None)
    ) -> NetviewGraph {

        label_propagation(
            graph, 
            centrality_metric,
            max_iterations, 
            vote_weights,
            neighbor_centrality_vote,
            distance_percent,
            query_nodes,
            propagate_on_unlabeled
        )
    }
    pub fn label_nodes(&self, graph: &mut NetviewGraph, labels: Vec<Option<String>>) -> Result<(), NetviewError> {
        log::info!("Labelling nodes on graph (n = {})", labels.len());
        label_nodes(graph, labels)
    }
    pub fn write_labels(&self, graph: &NetviewGraph, path: &Path) -> Result<(), NetviewError> {
        log::info!("Writing graph labels to: {}", path.display());
        write_graph_labels_to_file(&graph, path, false)
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