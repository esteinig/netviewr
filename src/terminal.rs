use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

/// Vircov: metagenomic diagnostics for viral genomes
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
    /// K mutual nearest neighbor graph computation from a distance matrix
    Graph(GraphArgs),
}


#[derive(Debug, Args)]
pub struct GraphArgs {
    /// Distance matrix for graph computation
    #[clap(long, short = 'd', required = true)]
    pub distance_matrix: PathBuf,
    /// K parameter for mutual nearest neighbor algorithm
    #[clap(long = "mutual-nearest-neighbors", short = 'k', default_value = "20")]
    pub k: usize,
    /// Include distances as edge weights in the graph
    #[clap(long, short = 'w')]
    pub weights: bool,
    /// Adjacency matrix output of the graph
    #[clap(long, short = 'o', default_value="graph.tsv")]
    pub output: PathBuf,
    /// Output format for graph
    #[clap(long, short = 'f', default_value="adjacency", )]
    pub output_format: String,
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
