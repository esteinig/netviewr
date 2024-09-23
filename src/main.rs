#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

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

            let graph = netview.create_graph(
                &args.distance_matrix, 
                args.k, 
                args.af_matrix.clone(),
            true
            )?;

            log::info!("Writing graph to output file: {}", args.output.display());
            write_graph_to_file(&graph, &args.output, &args.output_format, args.weights)?;
        }
    }
    Ok(())
}