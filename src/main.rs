#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use netview::dist::{parse_distance_matrix, euclidean_distance_of_distances};
use netview::mknn::{k_mutual_nearest_neighbors, convert_to_graph, write_graph_to_file};
use netview::terminal::{App, Commands};
use netview::error::NetviewError;
use netview::log::init_logger;

use clap::Parser;

pub fn main() -> Result<(), NetviewError> {
    
    init_logger();

    let cli = App::parse();

    match &cli.command {
        Commands::Graph(args) => {
           
            let distance = parse_distance_matrix(&args.distance_matrix)?;

            let distance_of_distances = euclidean_distance_of_distances(&distance, false, false, None)?;
            
            let mutual_nearest_neighbors = k_mutual_nearest_neighbors(&distance_of_distances, args.k)?;

            let mknn_graph = convert_to_graph(
                &mutual_nearest_neighbors, 
                match args.weights { true => Some(&distance), false => None }
            )?;

            let adj_matrix = netview::mknn::graph_to_adjacency_matrix(&mknn_graph, false)?;

            write_graph_to_file(&mknn_graph, &args.output, &args.output_format)?;
        }
    }
    Ok(())
}