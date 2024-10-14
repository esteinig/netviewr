use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

use crate::{centrality::NodeCentrality, mknn::GraphFormat};

#[cfg(feature = "plot")]
use crate::plot::PlotFormat;

/// Netview
#[derive(Debug, Parser)]
#[command(author, version, about)]
#[command(styles=get_styles())]
#[command(arg_required_else_help(true))]
#[clap(name = "netview", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Predict target labels using sparse k-mer chaining and population graphs
    Predict(PredictArgs),
    /// Pairwise distance matrix computation using 'skani'
    Dist(DistArgs),
    /// Mutual nearest neighbor population graph computation 
    Graph(GraphArgs),
    /// Label propagation to predict node labels on a graph
    Label(LabelArgs),
    /// Stratified label-based dereplication of input sequences 
    Derep(DerepArgs),
    /// Stratified k-fold cross-validation for prediction
    Xval(CrossValidationArgs),
    #[cfg(feature = "plot")]
    /// Plot a graph using the Netview plotting library
    Plot(PlotArgs)
}


#[derive(Debug, Args)]
pub struct PredictArgs {
    /// Genomes for prediction as single for multiple files (.fasta)
    #[clap(long, short = 'f', num_args(0..), required=true)]
    pub fasta: Vec<PathBuf>,
    /// Reference database sequences (.fasta)
    #[clap(long, short = 'd', required=true)]
    pub db: PathBuf,
    /// Database labels, in order of database genomes (.csv)
    #[clap(long, short = 'l', required = true)]
    pub labels: PathBuf,
    /// Output directory of working data and results
    #[clap(long, short = 'o', default_value="netview")]
    pub outdir: PathBuf,
    /// K parameter for mutual nearest neighbor algorithm
    #[clap(long = "mknn", short = 'k', num_args(0..), default_value="20")]
    pub k: usize,
    /// Propagate all labels across the graph topology
    #[clap(long, short = 'a')]
    pub all: bool,
    /// Basename of output files in output folder
    #[clap(long, short = 'n', default_value="netview")]
    pub basename: String,
    /// Threads for distance matrix computations
    #[clap(long, short = 't')]
    pub threads: Option<usize>,
    /// Chunk size for distance abstraction computation
    /// 
    /// Requires --threads for parallel chunk-wise computation of
    /// distance abstractions.
    #[clap(long, short = 'c')]
    pub chunk_size: Option<usize>,
    
    /// Distance threshold for mutual nearest neighbor edges
    /// 
    /// Includes only mutual nearest neighbors as edges if their distance
    /// is not equal or greater than this value - used to exclude neighbors
    /// in sparse distance matrices where there is no similarity at all (d >= 100.0)
    #[clap(long, short='e')]
    pub edge_threshold: Option<f64>,
    /// Netview configuration as JSON file (.json)
    #[clap(long)]
    pub json: Option<PathBuf>,
    /// Netview configuration as TOML file (.toml)
    #[clap(long)]
    pub toml: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct GraphArgs {
    /// Distance matrix for graph computation (square)
    #[clap(long, short = 'd', required = true)]
    pub dist: PathBuf,
    /// K parameter for mutual nearest neighbor algorithm
    #[clap(long = "mknn", short = 'k', num_args(0..), default_value="Vec::from([20])")]
    pub k: Vec<usize>,
    /// Alignment fraction matrix for populating edge labels from 'skani'
    #[clap(long, short = 'a', required = false)]
    pub afrac: Option<PathBuf>,
    /// Node identifier file e.g. sequence identifiers
    #[clap(long, short = 'i', required = false)]
    pub ids: Option<PathBuf>,
    /// Include distances as edge weights in the graph
    #[clap(long, short = 'w')]
    pub weights: bool,
    /// If output is an adjacency matrix, absence of an edge is 'NaN' instead of '0.0' 
    #[clap(long, short = 'n')]
    pub nan: bool,
    /// Graph output file
    #[clap(long, short = 'o', default_value="graph.json")]
    pub output: PathBuf,
    /// Output format for graph
    #[clap(long, short = 'f', default_value="json")]
    pub format: GraphFormat,
    /// Threads for distance abstraction computation
    #[clap(long, short = 't')]
    pub threads: Option<usize>,
    /// Chunk size for distance abstraction computation
    /// 
    /// Requires --threads for parallel chunk-wise computation of
    /// distance abstractions.
    #[clap(long, short = 'c')]
    pub chunk_size: Option<usize>,
    /// Distance threshold for mutual nearest neighbor edges
    /// 
    /// Includes only mutual nearest neighbors as edges if their distance
    /// is not equal or greater than this value - used to exclude neighbors
    /// in sparse distance matrices where there is no similarity at all (d >= 100.0)
    #[clap(long, short='e', default_value="100")]
    pub edge_threshold: Option<f64>,
}

#[derive(Debug, Args)]
pub struct LabelArgs {
    /// Netview graph in JSON format
    #[clap(long, short = 'g', required = true)]
    pub graph: PathBuf,
    /// Label file in order of node indices
    #[clap(long, short = 'l', required = true)]
    pub labels: PathBuf,
    /// Centrality metric for nodes used in label propagation
    #[clap(long, short = 'c', default_value="betweenness")]
    pub centrality: NodeCentrality,
    /// Maximum iterations or termination when no more labels change
    #[clap(long, short = 'm', default_value="20")]
    pub max_iterations: usize,
    /// Use neighbor centrality in vote weight determination
    #[clap(long, short = 'n')]
    pub neighbor_centrality: bool,
    /// Propagate labels for unlabelled nodes only
    #[clap(long, short = 'u')]
    pub unlabelled: bool,
    /// Propagate labels for query nodes only
    #[clap(long, short = 'q', num_args(0..))]
    pub query: Option<Vec<String>>,
    /// Propagated labels file in order of node indices
    #[clap(long, short = 'o', default_value="label.prop.csv")]
    pub output_labels: PathBuf,
    /// Netview graph with propagated labels in JSON format 
    #[clap(long, short = 'f', default_value="netview.prop.json")]
    pub output_graph: PathBuf,
}


#[derive(Debug, Args)]
pub struct CrossValidationArgs {
    /// Genomes for cross-validation dataset (.fasta)
    #[clap(long, short = 'f')]
    pub fasta: PathBuf,
    /// Label file in order of genomes (.csv)
    #[clap(long, short = 'l', required = true)]
    pub labels: PathBuf,
    /// Number of cross-validation folds
    #[clap(long, short = 'k', default_value="5")]
    pub k_folds: usize,
    /// Minimum sequence length to be included
    #[clap(long, short = 'm', default_value="0")]
    pub min_length: usize,
    /// Limit the number of sampled genomes per label  
    #[clap(long, short = 'n')]
    pub max_per_label: Option<usize>,
    /// Output directory for validation data and operations
    #[clap(long, short = 'o')]
    pub outdir: PathBuf,
}

#[derive(Debug, Args)]
pub struct DerepArgs {
    /// Genomes for replication by label (.fasta)
    #[clap(long, short = 'f', required = true)]
    pub fasta: PathBuf,
    /// Label file in order of genomes (.csv)
    #[clap(long, short = 'l', required = true)]
    pub labels: PathBuf,
    /// Exclude these labels, all unlabelled are excluded by default
    #[clap(long, short = 'e', required = false, num_args(0..), default_value="Vec::new()")]
    pub exclude: Vec<String>,
    /// Minimum sequence length to be included
    #[clap(long, short = 'm', default_value="0")]
    pub min_length: usize,
    /// Limit number of dereplicated genomes per label
    #[clap(long, short = 'n', default_value="20")]
    pub max_per_label: usize,
    /// Output dereplicated sequences
    #[clap(long, short = 'o', required = true)]
    pub output_fasta: PathBuf,
    /// Output dereplicated labels
    #[clap(long, short = 's', required = true)]
    pub output_labels: PathBuf,
}

#[derive(Debug, Args)]
pub struct DistArgs {
    /// Genomes for pairwise distance matrix in single file (.fasta)
    #[clap(long, short = 'f')]
    pub fasta: PathBuf,
    /// Output pairwise distance matrix as tab-delimited text file 
    #[clap(long, short = 'd')]
    pub dist: PathBuf,
    /// Output pairwise alignment fraction matrix as tab-delimited text file 
    #[clap(long, short = 'a')]
    pub afrac: Option<PathBuf>,
    /// Output sequence identifiers in order of matrix rows 
    #[clap(long, short = 'i')]
    pub ids: Option<PathBuf>,
    /// Output sequence identifiers excluded during 'skani' computations 
    #[clap(long, short = 'e')]
    pub excluded: Option<PathBuf>,
    /// Databases for subtyping
    #[clap(long, short = 'c', default_value="30")]
    pub compression_factor: usize,
    /// Output directory
    #[clap(long, short = 'm', default_value="200")]
    pub marker_compression_factor: usize,
    /// Minimum percent identity to include pairs 
    #[clap(long, short = 's', default_value="80.0")]
    pub min_percent_identity: f64,
    /// Minimum alignment fraction to include pair
    #[clap(long, short = 'n', default_value="15.0")]
    pub min_alignment_fraction: f64,
    /// Small genomes preset
    #[clap(long, short = 'g')]
    pub small_genomes: bool,
    /// Threads for distance matrix computation
    #[clap(long, short = 't', default_value = "8")]
    pub threads: usize,
}


#[cfg(feature = "plot")]
#[derive(Debug, Args)]
pub struct PlotArgs {
    /// Netview graph in JSON format
    #[clap(long, short = 'g', required = true)]
    pub graph: PathBuf,
    /// Output plot file
    #[clap(long, short = 'o', default_value="netview.png")]
    pub output: PathBuf,
    /// Output plot format
    #[clap(long, short = 'f', default_value="png")]
    pub format: PlotFormat,

}

pub fn get_styles() -> clap::builder::Styles {
	clap::builder::Styles::styled()
		.header(
			anstyle::Style::new()
				.bold()
				.underline()
				.fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
		)
		.literal(
			anstyle::Style::new()
				.bold()
				.fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
		)
}

fn _validate_file_path(path: &PathBuf) -> Result<(), String> {
    if !path.exists() { return Err(format!("File path does not exist: {}", path.display())) }
    Ok(())
}
