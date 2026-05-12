use std::{fs::{File, create_dir_all}, io::{Write, stdout}};
use csv::Writer;

use rand::SeedableRng;

use crate::{benchmark::run_algorithm, graph, treewidth::exact::ExactAlgorithm};

pub fn run_random_graphs_benchmark(algorithm: &ExactAlgorithm, seed: u64, num_iterations: usize, num_vertices: usize, num_edges: usize) -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all("benchmarks/random_graphs")?;
    let file = File::create(format!("benchmarks/random_graphs/{:?}_n{}_m{}.csv", algorithm, num_vertices, num_edges))?;
    let mut writer = Writer::from_writer(file);

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut mean_runtime = 0.0;

    for i in 0..num_iterations {
        let graph = graph::Graph::AdjList(graph::adjlist::Graph::generate_random_with_rng(num_vertices, num_edges, &mut rng));
        let runtime = run_algorithm(algorithm, &graph, &mut writer, None);

        mean_runtime = ((mean_runtime * i as f64) + runtime) / ((i + 1) as f64);
        let eta = mean_runtime * (num_iterations - i - 1) as f64;

        print!("Iteration {}/{}: Runtime = {:.2} seconds, Mean Runtime = {:.2} seconds, ETA = {:.2} seconds\r", i + 1, num_iterations, runtime, mean_runtime, eta);
        stdout().flush()?;
    }
    println!();
    Ok(())
}
