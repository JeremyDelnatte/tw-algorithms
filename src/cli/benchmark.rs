use std::{fs::{File, create_dir_all}, io::BufRead, time::Duration};

use clap::{ArgGroup, Parser, Subcommand};
use csv::Writer;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use tw_algorithms::{graph::adjlist::Graph, treewidth};

use crate::{cli::{approximate_treewidth::ApproxAlgorithmArg, compute_treewidth::ExactAlgorithmArg, progress_bar}, timeout};

#[derive(Parser)]
#[command(
    group(
        ArgGroup::new("alg")
            .required(true)
            .args(["exact_algorithm", "approximate_algorithm"])
    )
)] // Ensure that exactly one of --exact-algorithm or --approximate-algorithm is provided
pub struct BenchmarkArgs {
    #[arg(long, value_enum)]
    exact_algorithm: Option<ExactAlgorithmArg>,

    #[arg(long, value_enum)]
    approximate_algorithm: Option<ApproxAlgorithmArg>,

    #[arg(short = 'p', long = "progress-bar", help = "Show progress bars during benchmarking")]
    show_progress_bar: bool,

    #[arg(long, help = "Force re-run benchmarks even if results already exist and are valid")]
    force: bool,

    #[command(subcommand)]
    scenario: BenchmarkScenario,
}

#[derive(Subcommand)]
enum BenchmarkScenario {
    #[command(visible_aliases = ["random", "rand", "rg"])]
    RandomGraphs(RandomGraphsArgs),

    #[command(visible_aliases = ["preset", "pg"])]
    PresetGraphs(PresetGraphsArgs),
}

#[derive(Parser)]
struct RandomGraphsArgs {
    #[arg(long, help = "Seed for random graph generation", default_value_t = 42)]
    seed: u64,

    #[arg(long, help = "Number of iterations to run (for averaging)", default_value_t = 100)]
    num_iterations: usize,

    #[arg(long, help = "Number of vertices in the random graph")]
    num_vertices: usize,

    #[arg(long, help = "Number of edges in the random graph")]
    num_edges: usize,
}

#[derive(Parser)]
struct PresetGraphsArgs {
    #[arg(short = 'f', long, help = "File containing preset graphs (one in g6 format per line)")]
    graph_file: String,

    #[arg(long, help = "Number of iterations to run each algorithm on each graph (for averaging)", default_value_t = 1)]
    num_iterations: usize,
}

#[derive(Clone, Copy)]
pub enum AlgorithmArg {
    Exact(ExactAlgorithmArg),
    Approx(ApproxAlgorithmArg),
}

impl std::fmt::Debug for AlgorithmArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmArg::Exact(exact_alg) => write!(f, "{:?}", exact_alg),
            AlgorithmArg::Approx(approx_alg) => write!(f, "{:?}", approx_alg),
        }
    }
}

impl From<AlgorithmArg> for treewidth::Algorithm {
    fn from(arg: AlgorithmArg) -> Self {
        match arg {
            AlgorithmArg::Exact(exact_alg) => treewidth::Algorithm::Exact(exact_alg.into()),
            AlgorithmArg::Approx(approx_alg) => treewidth::Algorithm::Approx(approx_alg.into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExperimentResult {
    name: Option<String>,
    graph_g6: String,
    iteration: Option<usize>,
    runtime: u128,
    timeout: bool,
    treewidth: Option<usize>,
}

pub fn run(args: BenchmarkArgs, with_bitset: bool, timeout: Option<std::time::Duration>) -> Result<(), Box<dyn std::error::Error>> {
    let algorithm_arg = if let Some(exact_alg) = args.exact_algorithm {
        AlgorithmArg::Exact(exact_alg)
    } else if let Some(approx_alg) = args.approximate_algorithm {
        AlgorithmArg::Approx(approx_alg)
    } else {
        return Err("No algorithm specified".into());
    };

    match args.scenario {
        BenchmarkScenario::RandomGraphs(random_args) => benchmark_random_graphs(
            random_args,
            algorithm_arg,
            with_bitset,
            timeout,
            args.show_progress_bar,
            args.force,
        ),
        BenchmarkScenario::PresetGraphs(preset_args) => benchmark_preset_graphs(
            preset_args,
            algorithm_arg,
            with_bitset,
            timeout,
            args.show_progress_bar,
            args.force,
        ),
    }
}

fn benchmark_random_graphs(
    args: RandomGraphsArgs,
    algorithm: AlgorithmArg,
    with_bitset: bool,
    timeout: Option<std::time::Duration>,
    show_progress_bar: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {

    if !force && verify_benchmark_random_graphs(&args, algorithm, with_bitset)? {
        println!("Benchmark results already exist and are valid, skipping benchmark.");
        return Ok(());
    }

    let num_iterations = args.num_iterations;
    let num_vertices = args.num_vertices;
    let num_edges = args.num_edges;
    let seed = args.seed;

    let dir = match algorithm {
        AlgorithmArg::Exact(_) => "exact",
        AlgorithmArg::Approx(_) => "approx",
    };

    let dir_path = format!("benchmarks/random_graphs/{}", dir);
    create_dir_all(&dir_path)?;

    let with_bitset_str = if with_bitset { "_bitset" } else { "" };

    let file = File::create(format!("{}/{:?}_n{}_m{}_s{}{}.csv", dir_path, algorithm, num_vertices, num_edges, seed, with_bitset_str))?;
    let mut writer = Writer::from_writer(file);

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let progress = progress_bar::create_benchmark_progress_bars(
        show_progress_bar,
        num_iterations,
        None,
    )?;

    for _ in 0..num_iterations {
        let graph = Graph::generate_random_with_rng(num_vertices, num_edges, &mut rng).to_g6();

        progress_bar::start_graph_progress(&progress, &graph, None);
        run_algorithm(algorithm, &graph, with_bitset, timeout, &mut writer, None, None)?;
        progress_bar::finish_graph_progress(&progress);
    }

    progress_bar::finish_benchmark_progress(&progress);
    Ok(())
}

fn verify_benchmark_random_graphs(
    args: &RandomGraphsArgs,
    algorithm: AlgorithmArg,
    with_bitset: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let num_iterations = args.num_iterations;
    let num_vertices = args.num_vertices;
    let num_edges = args.num_edges;
    let seed = args.seed;

    let dir = match algorithm {
        AlgorithmArg::Exact(_) => "exact",
        AlgorithmArg::Approx(_) => "approx",
    };

    let dir_path = format!("benchmarks/random_graphs/{}", dir);
    let with_bitset_str = if with_bitset { "_bitset" } else { "" };

    let file_path = format!("{}/{:?}_n{}_m{}_s{}{}.csv", dir_path, algorithm, num_vertices, num_edges, seed, with_bitset_str);

    if !std::path::Path::new(&file_path).exists() {
        return Ok(false);
    }

    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let mut count = 0;
    for result in reader.deserialize() {
        let record: ExperimentResult = result?;
        let expected_graph = Graph::generate_random_with_rng(num_vertices, num_edges, &mut rng).to_g6();
        count += 1;

        if record.graph_g6 != expected_graph {
            return Ok(false);
        }
    }

    Ok(count == num_iterations)
}

fn benchmark_preset_graphs(
    args: PresetGraphsArgs,
    algorithm: AlgorithmArg,
    with_bitset: bool,
    timeout: Option<std::time::Duration>,
    show_progress_bar: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {

    if !force && verify_benchmark_preset_graphs(&args, algorithm, with_bitset)? {
        println!("Benchmark results already exist and are valid, skipping benchmark.");
        return Ok(());
    }

    let input_path = std::path::Path::new(&args.graph_file);
    let filename = input_path.file_name().ok_or("Invalid graph file path")?.to_string_lossy();

    let input_file = std::fs::File::open(input_path)?;
    let reader = std::io::BufReader::new(input_file);

    let dir = match algorithm {
        AlgorithmArg::Exact(_) => "exact",
        AlgorithmArg::Approx(_) => "approx",
    };

    let dir_path = format!("benchmarks/preset_graphs/{}", dir);
    create_dir_all(&dir_path)?;

    let with_bitset_str = if with_bitset { "_bitset" } else { "" };
    let output_file = File::create(format!("{}/{:?}_{}{}.csv", dir_path, algorithm, filename, with_bitset_str))?;
    let mut writer = Writer::from_writer(output_file);

    let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
    let num_graphs = lines.len();

    let progress = progress_bar::create_benchmark_progress_bars(
        show_progress_bar,
        num_graphs,
        Some(args.num_iterations),
    )?;

    for line in lines.into_iter() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (graph_name, g6) = match parts.as_slice() {
            [g6] => (None, *g6),
            [name, g6] => (Some(*name), *g6),
            _ => return Err(format!("Invalid line format: {}", line).into()),
        };

        progress_bar::start_graph_progress(&progress, graph_name.unwrap_or(g6), Some(args.num_iterations));

        for iteration in 0..args.num_iterations {
            run_algorithm(algorithm, g6, with_bitset, timeout, &mut writer, graph_name, Some(iteration))?;
            progress_bar::inc_iteration_progress(&progress);
        }
        progress_bar::finish_graph_progress(&progress);
    }

    progress_bar::finish_benchmark_progress(&progress);
    Ok(())
}

fn verify_benchmark_preset_graphs(
    args: &PresetGraphsArgs,
    algorithm: AlgorithmArg,
    with_bitset: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let input_path = std::path::Path::new(&args.graph_file);
    let filename = input_path.file_name().ok_or("Invalid graph file path")?.to_string_lossy();

    let dir = match algorithm {
        AlgorithmArg::Exact(_) => "exact",
        AlgorithmArg::Approx(_) => "approx",
    };

    let dir_path = format!("benchmarks/preset_graphs/{}", dir);
    let with_bitset_str = if with_bitset { "_bitset" } else { "" };
    let file_path = format!("{}/{:?}_{}{}.csv", dir_path, algorithm, filename, with_bitset_str);

    if !std::path::Path::new(&file_path).exists() {
        return Ok(false);
    }

    let input_file = File::open(input_path)?;
    let reader = std::io::BufReader::new(input_file);

    let file = File::open(file_path)?;
    let mut csv_reader = csv::Reader::from_reader(file);

    let mut graph_iter = reader.lines();
    let mut result_iter = csv_reader.deserialize();

    while let Some(graph_line) = graph_iter.next() {
        let graph_line = graph_line?;
        let parts: Vec<&str> = graph_line.split_whitespace().collect();
        let g6 = match parts.as_slice() {
            [g6] => *g6,
            [_, g6] => *g6,
            _ => return Err(format!("Invalid line format: {}", graph_line).into()),
        };

        for _ in 0..args.num_iterations {
            if let Some(result) = result_iter.next() {
                let record: ExperimentResult = result?;

                if record.graph_g6 != g6 {
                    return  Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn run_algorithm(
    algorithm: AlgorithmArg,
    graph_g6: &str,
    with_bitset: bool,
    timeout_opt: Option<std::time::Duration>,
    output: &mut Writer<File>,
    name: Option<&str>,
    iteration: Option<usize>,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let mut timeout_flag = false;

    let (tw, runtime) = if let Some(timeout) = timeout_opt {
        match timeout::compute_or_approximate_treewidth(graph_g6, algorithm, with_bitset, timeout) {
            Ok(result) => (Some(result.0), result.1),
            Err(timeout::TreewidthProcessError::Timeout { timeout }) => {
                timeout_flag = true;
                (None, timeout)
            },
            Err(e) => return Err(e.into()),
        }
    } else {
        let result = treewidth::compute_or_approximate_treewidth(graph_g6, algorithm.into(), with_bitset)?;
        (Some(result.0), result.1)
    };

    let experiment_result = ExperimentResult {
        graph_g6: graph_g6.to_string(),
        iteration,
        runtime: runtime.as_nanos(),
        timeout: timeout_flag,
        treewidth: tw,
        name: name.map(|s| s.to_string()),
    };

    output.serialize(experiment_result)?;
    Ok(runtime)
}
