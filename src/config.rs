use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use crate::{centrality::NodeCentrality, error::NetviewError, label::VoteWeights};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NetviewConfig {
    pub skani: SkaniConfig,
    pub graph: GraphConfig,
    pub label: LabelConfig,
}

impl Default for NetviewConfig {
    fn default() -> Self {
        NetviewConfig {
            skani: SkaniConfig::default(),
            graph: GraphConfig::default(),
            label: LabelConfig::default(),
        }
    }
}
impl NetviewConfig {
    // Default with some parameters configured manually
    pub fn with_default(k: usize) -> Self {
        NetviewConfig {
            skani: SkaniConfig::default(),
            graph: GraphConfig::with_default(k),
            label: LabelConfig::default(),
        }
    }
    // Read JSON file into NetviewConfig
    pub fn read_json(path: &PathBuf) -> Result<Self, NetviewError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    // Write NetviewConfig to JSON file
    pub fn write_json(&self, path: &PathBuf) -> Result<(), NetviewError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    // Read TOML file into NetviewConfig
    pub fn read_toml(path: &PathBuf) -> Result<Self, NetviewError> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    // Write NetviewConfig to TOML file
    pub fn write_toml(&self, path: &PathBuf) -> Result<(), NetviewError> {
        let content = toml::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkaniConfig {
    pub marker_compression_factor: usize,
    pub compression_factor: usize,
    pub threads: usize,
    pub min_percent_identity: f64,
    pub min_alignment_fraction: f64,
    pub small_genomes: bool,
}

impl Default for SkaniConfig {
    fn default() -> Self {
        SkaniConfig {
            marker_compression_factor: 200,
            compression_factor: 30,
            threads: 8,
            min_percent_identity: 0.0,
            min_alignment_fraction: 0.0,
            small_genomes: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphConfig {
    pub k: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        GraphConfig { k: 20 }
    }
}
impl GraphConfig {
    fn with_default(k: usize) -> Self {
        Self { k }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelConfig {
    pub centrality_metric: NodeCentrality,
    pub max_iterations: usize,
    pub vote_weights: VoteWeights,
    pub neighbor_centrality_vote: bool,
    pub distance_percent: bool,
}

impl Default for LabelConfig {
    fn default() -> Self {
        LabelConfig {
            centrality_metric: NodeCentrality::Degree,
            max_iterations: 20,
            vote_weights: VoteWeights::default(),
            neighbor_centrality_vote: false,
            distance_percent: true,
        }
    }
}
