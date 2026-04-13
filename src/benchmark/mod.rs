use std::fs::File;

use csv::Writer;
use serde::Serialize;
use tokio::{task, time::timeout};

use crate::{benchmark::{preset_graphs::{run_preset_graphs_benchmark, run_preset_graphs_benchmark_timeout}, random_graphs::run_random_graphs_benchmark}, graph::{self, Graph}, treewidth::{self, Algorithm}};

mod random_graphs;
mod preset_graphs;

pub struct BenchmarkConfig {
    pub algorithm: Algorithm,
    pub benchmark_type: BenchmarkType,
}

pub enum BenchmarkType {
    RandomGraphs {
        seed: u64,
        num_iterations: usize,
        num_vertices: usize,
        num_edges: usize,
    },
    PresetGraphs {
        graph_file: String,
    }
}

#[derive(Serialize)]
struct ExperimentResult {
    graph_g6: String,
    runtime: Option<f64>,
    timeout: bool,
    treewidth: Option<usize>,
    name: Option<String>,
}

fn run_algorithm(algorithm: &Algorithm, graph: &Graph, output: &mut Writer<File>, name: Option<&str>) -> f64 {
    let start_time = std::time::Instant::now();
    let tw = match algorithm {
        Algorithm::DynamicProg => treewidth::dynamic_prog::treewidth(&graph),
        Algorithm::Recursive => treewidth::rec::treewidth(&graph),
        Algorithm::ImprovedRec => treewidth::improved_rec::treewidth(&graph),
        Algorithm::BranchBound => treewidth::branch_bound::treewidth(&graph),
    };
    let duration = start_time.elapsed();
    let runtime = duration.as_secs_f64();

    let experiment_result = ExperimentResult {
        graph_g6: graph.to_g6(),
        runtime: Some(runtime),
        timeout: false,
        treewidth: Some(tw),
        name: name.map(|s| s.to_string()),
    };

    output.serialize(experiment_result).expect("Failed to write result");
    runtime
}

async fn run_algorithm_timeout(algorithm: &Algorithm, graph: &Graph, output: &mut Writer<File>, name: Option<&str>, timeout_dur: std::time::Duration) -> Option<f64> {
    let graph = graph.clone();
    let graph_g6 = graph.to_g6();
    let algorithm = algorithm.clone();

    let (tw, runtime) = match timeout(timeout_dur, task::spawn_blocking(move || {
        let start_time = std::time::Instant::now();
        let tw = match algorithm {
            Algorithm::DynamicProg => treewidth::dynamic_prog::treewidth(&graph),
            Algorithm::Recursive => treewidth::rec::treewidth(&graph),
            Algorithm::ImprovedRec => treewidth::improved_rec::treewidth(&graph),
            Algorithm::BranchBound => treewidth::branch_bound::treewidth(&graph),
        };
        let duration = start_time.elapsed();
        let runtime = duration.as_secs_f64();
        (tw, runtime)
    })).await {
        Ok(join_result) => join_result.map(|(tw, runtime)| (Some(tw), Some(runtime))).unwrap_or((None, None)),
        Err(_) => (None, None),
    };

    let experiment_result = ExperimentResult {
        graph_g6,
        runtime,
        timeout: tw.is_none(),
        treewidth: tw,
        name: name.map(|s| s.to_string()),
    };

    output.serialize(experiment_result).expect("Failed to write result");
    runtime
}

pub fn run_benchmarks(config: BenchmarkConfig) -> Result<(), Box<dyn std::error::Error>> {
    match config.benchmark_type {
        BenchmarkType::RandomGraphs { seed, num_iterations, num_vertices, num_edges } => {
            run_random_graphs_benchmark(&config.algorithm, seed, num_iterations, num_vertices, num_edges)
        },
        BenchmarkType::PresetGraphs { graph_file } => {
            run_preset_graphs_benchmark(&config.algorithm, &graph_file)
        }
    }
}

pub async fn run_benchmarks_timeout(config: BenchmarkConfig, timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
    match config.benchmark_type {
        // BenchmarkType::RandomGraphs { seed, num_iterations, num_vertices, num_edges } => {
        //     run_random_graphs_benchmark_timeout(&config.algorithm, seed, num_iterations, num_vertices, num_edges, timeout)
        // },
        BenchmarkType::RandomGraphs { .. } => unimplemented!(),
        BenchmarkType::PresetGraphs { graph_file } => {
            run_preset_graphs_benchmark_timeout(&config.algorithm, &graph_file, timeout).await
        }
    }
}
