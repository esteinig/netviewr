// Graph layouts and plotting, hacked-up for now

use petgraph::graph::{Graph, NodeIndex};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use petgraph::visit::{Dfs, EdgeRef};
use petgraph::Undirected;
use plotters::prelude::*;
use rand::Rng;

use crate::error::NetviewError;
use crate::netview::{Netview, NetviewGraph};

pub enum Layout {
    ForceDirected,
    FruchtermannReingold
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum PlotFormat {
    Png
}


pub struct ForceDirectedConfig {
    pub repulsion_constant: f64,
    pub attraction_constant: f64,
    pub max_iterations: usize,
}
impl Default for ForceDirectedConfig {
    fn default() -> Self {
        Self {
            repulsion_constant: 1000.0,
            attraction_constant: 0.1,
            max_iterations: 100
        }
    }
}

pub struct FruchtermanReingoldConfig {
    pub max_iterations: usize,
    pub start_temp: f64,
    pub min_disp: f64,
}
impl Default for FruchtermanReingoldConfig {
    fn default() -> Self {
        Self {
            max_iterations: 500,
            start_temp: 20.0,
            min_disp: 1e-09,
        }
    }
}

pub struct PlotConfig {
    width: u32,
    height: u32
}
impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

pub fn get_random_graph() -> Graph<(), (), Undirected> {

    // Create a simple undirected graph for testing layout algorithms
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();

    // Add nodes and edges
    let n1 = graph.add_node(());
    let n2 = graph.add_node(());
    let n3 = graph.add_node(());
    let n4 = graph.add_node(());
    
    graph.add_edge(n1, n2, ());
    graph.add_edge(n2, n3, ());
    graph.add_edge(n3, n4, ());
    graph.add_edge(n4, n1, ());

    return graph
}


pub fn init_random_node_positions(graph: &NetviewGraph, config: &PlotConfig) -> HashMap<NodeIndex, Node> {

    // Initialize random positions for the nodes
    let mut positions: HashMap<NodeIndex, Node> = HashMap::new();
    let mut rng = rand::thread_rng();
    
    for node in graph.node_indices() {
        let x = rng.gen_range(0.0..config.width as f64);
        let y = rng.gen_range(0.0..config.height as f64);
        positions.insert(node, Node { x, y, vx: 0.0, vy: 0.0 });
    }
    positions
}

// Simple test function of the force-directed layout - only usable for very small graphs without disconnected components
pub fn force_directed_layout(graph: &NetviewGraph, mut positions: HashMap<NodeIndex, Node>, config: &ForceDirectedConfig) -> HashMap<NodeIndex, Node> {

    // Run the force-directed layout algorithm (simple version)
    for _ in 0..config.max_iterations {
        // Apply repulsive force between all nodes
        for (i, pos_i) in positions.clone().iter() {
            for (j, pos_j) in positions.clone().iter() {
                if i != j {
                    let dx = pos_i.x - pos_j.x;
                    let dy = pos_i.y - pos_j.y;
                    let distance = (dx * dx + dy * dy).sqrt().max(1.0);  // avoid division by zero
                    let force = config.repulsion_constant / distance.powi(2);
                    let fx = force * dx / distance;
                    let fy = force * dy / distance;

                    positions.get_mut(i).unwrap().vx += fx;
                    positions.get_mut(i).unwrap().vy += fy;
                }
            }
        }

        // Apply attractive force between connected nodes (spring-like)
        for edge in graph.edge_references() {
            let (i, j) = (edge.source(), edge.target());
            let pos_i = positions.get(&i).unwrap().clone();
            let pos_j = positions.get(&j).unwrap().clone();

            let dx = pos_i.x - pos_j.x;
            let dy = pos_i.y - pos_j.y;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);
            let force = config.attraction_constant * (distance - 50.0); // Target distance of 50 units
            let fx = force * dx / distance;
            let fy = force * dy / distance;

            positions.get_mut(&i).unwrap().vx -= fx;
            positions.get_mut(&i).unwrap().vy -= fy;
            positions.get_mut(&j).unwrap().vx += fx;
            positions.get_mut(&j).unwrap().vy += fy;
        }

        // Update positions based on velocity
        for pos in positions.values_mut() {
            pos.x += pos.vx;
            pos.y += pos.vy;

            // Apply some friction to avoid oscillations
            pos.vx *= 0.85;
            pos.vy *= 0.85;
        }
    }

    positions

}


fn random_bounded(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

// Check if the graph is connected using a simple DFS
fn is_connected(graph: &NetviewGraph) -> bool {
    if graph.node_count() == 0 {
        return false;
    }
    let start_node = graph.node_indices().next().unwrap();
    let mut visited = HashSet::new();
    let mut dfs = petgraph::visit::Dfs::new(&graph, start_node);

    while let Some(nx) = dfs.next(&graph) {
        visited.insert(nx);
    }

    visited.len() == graph.node_count()
}


// Identify the connected components in the graph (required for handling disconnected components)
fn connected_components(graph: &NetviewGraph) -> Vec<Vec<NodeIndex>> {
    let mut visited = HashSet::new();
    let mut components: Vec<Vec<NodeIndex>> = Vec::new();

    for node in graph.node_indices() {
        if !visited.contains(&node) {
            let mut component = Vec::new();
            let mut dfs = Dfs::new(graph, node);
            while let Some(nx) = dfs.next(graph) {
                if visited.insert(nx) {
                    component.push(nx);
                }
            }
            components.push(component);
        }
    }
    
    components
}


/// Rust implementation of the Fruchterman-Reingold algorithm follows the original igraph implementation 
/// with additional handling of repulsive forces for each component in the topology. Temperature decay and
/// movement limiting help stabilize the graph layout as it converges over iterations.We handle disconnected 
/// components and singletons with the `connected_components` function, ensure edge weights affect node 
/// attraction and implement random perturbations to prevent divisions by zero.
fn fruchterman_reingold_modular(graph: &NetviewGraph, layout_config: &FruchtermanReingoldConfig, plot_config: &PlotConfig) -> HashMap<NodeIndex, Node> {
    
    let vcount = graph.node_count();

    let mut positions: HashMap<NodeIndex, Node> = HashMap::new();
    let mut dispx = vec![0.0; vcount];
    let mut dispy = vec![0.0; vcount];
    let temp = layout_config.start_temp;
    let difftemp = layout_config.start_temp / layout_config.max_iterations as f64;
    let components = connected_components(graph);
    
    // C constant to adjust forces in unconnected components
    let c = (vcount as f64) * (vcount as f64).sqrt();

    // Initialize random positions for nodes
    let mut rng = rand::thread_rng();
    for node in graph.node_indices() {
        let x = rng.gen_range(0.0..plot_config.width as f64);
        let y = rng.gen_range(0.0..plot_config.height as f64);
        positions.insert(node, Node { x, y, vx: 0.0, vy: 0.0 });
    }

    let mut current_temp = temp;

    for _ in 0..layout_config.max_iterations {
        // Reset displacements
        dispx.iter_mut().for_each(|x| *x = 0.0);
        dispy.iter_mut().for_each(|y| *y = 0.0);

        // Calculate repulsive forces for each component
        for component in &components {
            for (i, &v) in component.iter().enumerate() {
                for &u in component.iter().skip(i + 1) {
                    let pos_v = positions.get(&v).unwrap();
                    let pos_u = positions.get(&u).unwrap();
                    let mut dx = pos_v.x - pos_u.x;
                    let mut dy = pos_v.y - pos_u.y;
                    let mut dlen = dx * dx + dy * dy;

                    // Apply random perturbation to avoid division by zero
                    while dlen == 0.0 {
                        dx = random_bounded(-layout_config.min_disp, layout_config.min_disp);
                        dy = random_bounded(-layout_config.min_disp, layout_config.min_disp);
                        dlen = dx * dx + dy * dy;
                    }

                    let rdlen = dlen.sqrt();
                    dispx[v.index()] += dx * (c - dlen * rdlen) / (dlen * c);
                    dispy[v.index()] += dy * (c - dlen * rdlen) / (dlen * c);
                    dispx[u.index()] -= dx * (c - dlen * rdlen) / (dlen * c);
                    dispy[u.index()] -= dy * (c - dlen * rdlen) / (dlen * c);
                }
            }
        }

        // Calculate attractive forces (using edge weights)
        for edge in graph.edge_indices() {
            let (v, u) = graph.edge_endpoints(edge).unwrap();
            let pos_v = positions.get(&v).unwrap();
            let pos_u = positions.get(&u).unwrap();
            let weight = graph.edge_weight(edge).unwrap().weight;

            let dx = pos_v.x - pos_u.x;
            let dy = pos_v.y - pos_u.y;
            let dlen = (dx * dx + dy * dy).sqrt() * weight;

            dispx[v.index()] -= dx * dlen;
            dispy[v.index()] -= dy * dlen;
            dispx[u.index()] += dx * dlen;
            dispy[u.index()] += dy * dlen;
        }

        // Limit max displacement and apply temperature-based movement
        for (v, pos) in positions.iter_mut() {
            let dx = dispx[v.index()];
            let dy = dispy[v.index()];
            let displen = (dx * dx + dy * dy).sqrt();

            if displen > current_temp {
                pos.vx = dx * current_temp / displen;
                pos.vy = dy * current_temp / displen;
            } else {
                pos.vx = dx;
                pos.vy = dy;
            }

            pos.x += pos.vx;
            pos.y += pos.vy;
        }

        current_temp -= difftemp;
    }

    positions
}

fn fruchterman_reingold(graph: &NetviewGraph, layout_config: &FruchtermanReingoldConfig, plot_config: &PlotConfig) -> HashMap<NodeIndex, Node> {

    let vcount = graph.node_count();
    let mut positions: HashMap<NodeIndex, Node> = HashMap::new();
    let mut dispx = vec![0.0; vcount];
    let mut dispy = vec![0.0; vcount];
    let temp = layout_config.start_temp;
    let difftemp = layout_config.start_temp / layout_config.max_iterations as f64;
    let connected = is_connected(graph);

    // Initialize constant C if the graph is disconnected
    let c = if connected { 0.0 } else { (vcount as f64) * (vcount as f64).sqrt() };

    // Randomly initialize positions of nodes
    let mut rng = rand::thread_rng();
    for node in graph.node_indices() {
        let x = rng.gen_range(0.0..plot_config.width as f64);
        let y = rng.gen_range(0.0..plot_config.height as f64);
        positions.insert(node, Node { x, y, vx: 0.0, vy: 0.0 });
    }

    let mut current_temp = temp;

    for _ in 0..layout_config.max_iterations {
        // Reset displacement vectors
        dispx.iter_mut().for_each(|x| *x = 0.0);
        dispy.iter_mut().for_each(|y| *y = 0.0);

        // Calculate repulsive forces
        for v in graph.node_indices() {
            for u in graph.node_indices() {
                if v != u {
                    let pos_v = positions.get(&v).unwrap();
                    let pos_u = positions.get(&u).unwrap();
                    let mut dx = pos_v.x - pos_u.x;
                    let mut dy = pos_v.y - pos_u.y;
                    let mut dlen = dx * dx + dy * dy;

                    // Apply random perturbation to avoid division by zero
                    while dlen == 0.0 {
                        dx = random_bounded(-layout_config.min_disp, layout_config.min_disp);
                        dy = random_bounded(-layout_config.min_disp, layout_config.min_disp);
                        dlen = dx * dx + dy * dy;
                    }

                    // Handle connected or unconnected graphs differently
                    if connected {
                        // Repulsive force for connected graphs
                        dispx[v.index()] += dx / dlen;
                        dispy[v.index()] += dy / dlen;
                        dispx[u.index()] -= dx / dlen;
                        dispy[u.index()] -= dy / dlen;
                    } else {
                        // Adjusted repulsive force for disconnected graphs using C
                        let rdlen = dlen.sqrt();
                        dispx[v.index()] += dx * (c - dlen * rdlen) / (dlen * c);
                        dispy[v.index()] += dy * (c - dlen * rdlen) / (dlen * c);
                        dispx[u.index()] -= dx * (c - dlen * rdlen) / (dlen * c);
                        dispy[u.index()] -= dy * (c - dlen * rdlen) / (dlen * c);
                    }
                }
            }
        }

        // Calculate attractive forces using edge weights
        for edge in graph.edge_references() {
            let (v, u) = (edge.source(), edge.target());
            let pos_v = positions.get(&v).unwrap();
            let pos_u = positions.get(&u).unwrap();
            let weight = edge.weight().weight;  // Use edge weight

            let dx = pos_v.x - pos_u.x;
            let dy = pos_v.y - pos_u.y;
            let dlen = (dx * dx + dy * dy).sqrt() * weight;

            dispx[v.index()] -= dx * dlen;
            dispy[v.index()] -= dy * dlen;
            dispx[u.index()] += dx * dlen;
            dispy[u.index()] += dy * dlen;
        }

        // Limit displacement to temperature and move nodes
        for (v, pos) in positions.iter_mut() {
            let dx = dispx[v.index()];
            let dy = dispy[v.index()];
            let displen = (dx * dx + dy * dy).sqrt();

            // Scale by temperature
            if displen > current_temp {
                pos.vx = dx * current_temp / displen;
                pos.vy = dy * current_temp / displen;
            } else {
                pos.vx = dx;
                pos.vy = dy;
            }

            pos.x += pos.vx;
            pos.y += pos.vy;
        }

        current_temp -= difftemp;  // Decrease temperature over time
    }

    positions
}

// Plots

pub fn plot_graph(graph: &NetviewGraph, positions: HashMap<NodeIndex, Node>, config: &PlotConfig, output: &Path) -> Result<(), NetviewError> {

    // Plot the resulting graph layout
    let root = BitMapBackend::new(
        output, 
        (config.width, config.height)
    ).into_drawing_area();
    
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Force Directed Graph Layout", ("sans-serif", 50))
        .build_cartesian_2d(
            0.0..config.width as f64, 
            0.0..config.height as f64
        )?;

    // Draw the edges
    for edge in graph.edge_references() {
        let i = positions.get(&edge.source()).unwrap();
        let j = positions.get(&edge.target()).unwrap();

        chart.draw_series(LineSeries::new(
            vec![(i.x, i.y), (j.x, j.y)],
            &BLACK,
        ))?;
    }

    // Draw the nodes
    for (_, pos) in positions.iter() {
        chart.draw_series(PointSeries::of_element(
            vec![(pos.x, pos.y)],
            5,
            &RED,
            &|coord, size, style| {
                return EmptyElement::at(coord)    // Position of the node
                    + Circle::new((0, 0), size, style.filled());
            },
        ))?;
    }

    root.present()?;

    Ok(())

}

pub fn plot_test(graph_json: &Path) -> Result<(), NetviewError> {

    let plot_config = PlotConfig::default();

    let netview = Netview::from_json(&graph_json)?;

    let fd_config = ForceDirectedConfig::default();
    let random_positions = init_random_node_positions(&netview.graph, &plot_config);
    let fd_positions = force_directed_layout(&netview.graph, random_positions, &fd_config);


    let fr_config = FruchtermanReingoldConfig::default();
    let fr_positions = fruchterman_reingold(&netview.graph, &fr_config, &plot_config);
    let frm_positions = fruchterman_reingold_modular(&netview.graph, &fr_config, &plot_config);

    plot_graph(&netview.graph, fd_positions, &plot_config, Path::new("graph_fd_layout.png"))?;
    plot_graph(&netview.graph, fr_positions, &plot_config, Path::new("graph_fr_layout.png"))?;
    plot_graph(&netview.graph, frm_positions, &plot_config, Path::new("graph_frm_layout.png"))?;

    Ok(())
}