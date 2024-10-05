#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use netview::centrality::NodeCentrality;

use netview::config::NetviewConfig;
use netview::derep::Dereplicator;
#[cfg(feature = "plot")]
use netview::plot::plot_test;

use netview::dist::{skani_distance_matrix, write_ids, write_matrix_to_file};
use netview::label::{read_labels_from_file, VoteWeights};
use netview::mknn::write_graph_to_file;
use netview::log::init_logger;

use netview::terminal::{App, Commands};
use netview::error::NetviewError;
use netview::netview::Netview;

use netview::validation::CrossFoldValidation;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use clap::Parser;


pub fn main() -> Result<(), NetviewError> {
    
    init_logger();

    let cli = App::parse();

    match &cli.command {
        Commands::Graph(args) => {
            
            let netview = Netview::new(NetviewConfig::default());

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
            write_matrix_to_file(&dist, &args.dist)?;

            if let Some(path) = &args.afrac {

                log::info!("Writing alignment fraction matrix to: {}", path.display());
                write_matrix_to_file(&af, &path)?;
            }
            if let Some(path) = &args.ids {
                log::info!("Writing sequence identifiers to: {}", path.display());
                write_ids(&ids, &path)?;
            }
        },
        Commands::Label(args) => {

            let netview = Netview::new(NetviewConfig::default());

            let mut graph = netview.read_json_graph(&args.graph)?;

            log::info!("Reading labels from file...");
            let labels: Vec<Option<String>> = read_labels_from_file(&args.labels, false)?
                .into_iter()
                .map(|g| g.label)
                .collect();

            log::info!("Decorating nodes with labels...");
            netview.label_nodes(&mut graph, labels)?;

            netview.label_propagation(
                &mut graph,
                NodeCentrality::Degree, 
                args.max_iterations, 
                VoteWeights::default(),
                args.neighbor_centrality, 
                true, 
                args.query.clone(), 
                args.unlabelled
            );

            netview.write_labels(&graph, &args.output_labels)?;
            
        },
        Commands::Derep(args) => {

            let drp = Dereplicator::new(
                &args.fasta, 
                &args.labels, 
                args.max_per_label
            );

            drp.dereplicate(&args.output_fasta, &args.output_labels, &args.exclude, args.min_length)?;
            
        },
        Commands::Xval(args) => {

            let cv = CrossFoldValidation::new(
                &args.labels, 
                &args.fasta, 
                args.k_folds, 
                args.max_per_label.clone(),
                &args.outdir,
            )?;

            cv.generate_k_folds()?;
            
        },
        Commands::Predict(args) => {

            let config = match (&args.json, &args.toml) {
                (Some(path), _)    => NetviewConfig::read_json(path)?,
                (None, Some(path)) => NetviewConfig::read_toml(path)?,
                _ => NetviewConfig::default()
            };

            let netview = Netview::new(config);

            netview.predict(
                &args.fasta, 
                &args.db,
                &args.labels, 
                args.k, 
                &args.outdir,
                args.all,
                args.basename.clone(),
                args.threads
            )?;
            
        },

        #[cfg(feature = "plot")]
        Commands::Plot(args) => {
            plot_test(&args.graph)?;
        }
    }
    Ok(())
}