#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

#[cfg(feature = "plot")]
use netview::plot::plot_test;

use netview::dist::{skani_distance_matrix, write_ids, write_matrix_to_file};
use netview::mknn::write_graph_to_file;
use netview::log::init_logger;

use netview::terminal::{App, Commands};
use netview::error::NetviewError;
use netview::netview::Netview;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use clap::Parser;


pub fn main() -> Result<(), NetviewError> {
    
    init_logger();

    let cli = App::parse();

    match &cli.command {
        Commands::Graph(args) => {
            
            let netview = Netview::new();

            args.k.par_iter().for_each(|k| {

                log::info!("Computing mutual nearest neighbor graph at k = {k}");
                
                let graph = netview.graph_from_files(
                    &args.dist, 
                    *k, 
                    args.afrac.clone(),
                    args.ids.clone(),
                    false,
                ).expect(&format!("Failed to create graph (k = {k})"));
                
                let output = if args.k.len() == 1 {
                    args.output.clone()
                } else {
                    args.output.with_extension(format!("k{k}.{}", args.format))
                };

                write_graph_to_file(
                    &graph, 
                    &output, 
                    &args.format, 
                    args.weights
                ).expect(&format!("Failed to write graph (k = {k})"));
            });

        },
        Commands::Dist(args) => {

            let (dist, af, ids) = skani_distance_matrix(
                &args.fasta, 
                args.marker_compression_factor, 
                args.compression_factor, 
                args.threads, 
                args.min_percent_identity,
                args.min_alignment_fraction,
                args.small_genomes
            )?;

            log::info!("Writing distance matrix to: {}", args.dist.display());
            write_matrix_to_file(dist, &args.dist)?;

            if let Some(path) = &args.afrac {

                log::info!("Writing alignment fraction matrix to: {}", path.display());
                write_matrix_to_file(af, &path)?;
            }
            if let Some(path) = &args.ids {

                log::info!("Writing sequence identifiers to: {}", path.display());
                write_ids(ids, &path)?;
            }
        },

        #[cfg(feature = "plot")]
        Commands::Plot(args) => {
            plot_test(&args.graph)?;
        }
    }
    Ok(())
}