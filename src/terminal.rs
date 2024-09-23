use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

use crate::mknn::GraphFormat;

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
    /// Mutual nearest neighbor graph computation from a distance matrix
    Graph(GraphArgs),
    /// Pairwise distance matrix computation using 'skani'
    Dist(DistArgs)
}


#[derive(Debug, Args)]
pub struct GraphArgs {
    /// Distance matrix for graph computation (square)
    #[clap(long, short = 'd', required = true)]
    pub dist: PathBuf,
    /// K parameter for mutual nearest neighbor algorithm
    #[clap(long = "mknn", short = 'k', default_value = "20")]
    pub k: usize,
    /// Alignment fraction matrix for populating edge labels from 'skani'
    #[clap(long, short = 'a', required = false)]
    pub afrac: Option<PathBuf>,
    /// Include distances as edge weights in the graph
    #[clap(long, short = 'w')]
    pub weights: bool,
    /// Graph output file
    #[clap(long, short = 'o', default_value="graph.json")]
    pub output: PathBuf,
    /// Output format for graph
    #[clap(long, short = 'f', default_value="json")]
    pub format: GraphFormat,
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
