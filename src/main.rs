#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use netview::dist::{skani_distance_matrix, write_matrix_to_file};
use netview::mknn::write_graph_to_file;
use netview::netview::Netview;
use netview::terminal::{App, Commands};
use netview::error::NetviewError;
use netview::log::init_logger;

use clap::Parser;

pub fn main() -> Result<(), NetviewError> {
    
    init_logger();

    let cli = App::parse();

    match &cli.command {
        Commands::Graph(args) => {
            
            let netview = Netview::new();

            let graph = netview.graph_from_files(
                &args.dist, 
                args.k, 
                args.afrac.clone(),
                false
            )?;

            write_graph_to_file(
                &graph, 
                &args.output, 
                &args.format, 
                args.weights
            )?;
        },
        Commands::Dist(args) => {

            let (dist, af) = skani_distance_matrix(
                &args.fasta, 
                args.marker_compression_factor, 
                args.compression_factor, 
                args.threads, 
                args.min_percent_identity,
                args.min_alignment_fraction,
                args.small_genomes
            )?;

            write_matrix_to_file(dist, &args.dist)?;

            if let Some(afrac) = &args.afrac {
                write_matrix_to_file(af, &afrac)?;
            }
        }
    }
    Ok(())
}