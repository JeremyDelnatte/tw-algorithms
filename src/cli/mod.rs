pub mod compute_treewidth;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(
        short = 'b',
        long = "bitset",
        global = true,
        help = "Allow using bitset-based graph representation for algorithms that support it"
    )]
    with_bitset: bool,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Command::ComputeTreewidth(compute_args) => {
                match compute_args.input_type() {
                    InputType::SingleGraph(g6) => compute_treewidth::compute_treewidth_single(
                        &g6,
                        compute_args.algorithm,
                        self.with_bitset,
                        compute_args.expected_treewidth,
                    ),
                    InputType::GraphsFile(filename) => compute_treewidth::compute_treewidth_file(
                        &filename,
                        compute_args.algorithm,
                        self.with_bitset,
                        compute_args.expected_treewidth,
                    ),
                }
            }
            // Command::Benchmark => crate::benchmark::run_benchmarks(),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    #[command(visible_aliases = ["tw"])]
    ComputeTreewidth(ComputeTreewidthArgs),

    // #[command(visible_aliases = ["bm", "b"])]
    // Benchmark,
}

#[derive(Clone, ValueEnum, Debug)]
enum AlgorithmArg {
    #[value(alias("dp"))]
    DynamicProg,

    #[value(alias("rec"))]
    Recursive,

    #[value(alias("imprec"))]
    ImprovedRec,

    #[value(alias("bb"))]
    BranchBound,
}

impl From<AlgorithmArg> for tw_algorithms::treewidth::Algorithm {
    fn from(arg: AlgorithmArg) -> Self {
        match arg {
            AlgorithmArg::DynamicProg => Self::DynamicProg,
            AlgorithmArg::Recursive => Self::Recursive,
            AlgorithmArg::ImprovedRec => Self::ImprovedRec,
            AlgorithmArg::BranchBound => Self::BranchBound,
        }
    }
}

#[derive(Parser)]
#[command(
    group(
        ArgGroup::new("input")
            .required(true)
            .args(["graph", "graphs_file"])
    )
)] // Ensure that exactly one of --graph or --file is provided
struct ComputeTreewidthArgs {
    #[arg(short = 'a', long, value_enum, default_value_t = AlgorithmArg::DynamicProg)]
    algorithm: AlgorithmArg,

    #[arg(short = 'f', long = "file")]
    graphs_file: Option<String>,

    #[arg(short = 'g', long)]
    graph: Option<String>,

    #[arg(short = 't', long = "treewidth")]
    expected_treewidth: Option<usize>,
}

enum InputType {
    SingleGraph(String),
    GraphsFile(String),
}

impl ComputeTreewidthArgs {
    fn input_type(&self) -> InputType {
        match (&self.graphs_file, &self.graph) {
            (Some(file), None) => InputType::GraphsFile(file.to_string()),
            (None, Some(g)) => InputType::SingleGraph(g.to_string()),
            _ => {
                unreachable!("Clap should ensure that exactly one of --file or --graph is provided")
            }
        }
    }
}

#[derive(Parser)]
struct BenchmarkArgs {
    #[arg(short = 'a', long, value_enum, default_value_t = AlgorithmArg::DynamicProg)]
    algorithm: AlgorithmArg,

    #[arg(short = 'r', long = "random", help = "Whether to generate random graphs for benchmarking")]
    generate_random_graphs: bool,

    #[arg(short = 'i', long = "iterations", default_value_t = 100, help = "Number of iterations for benchmarking")]
    num_iterations: usize,

    #[arg(short = 'n', long = "vertices", default_value_t = 10, help = "Number of vertices for random graphs")]
    num_vertices: usize,

    #[arg(short = 'm', long = "edges", default_value_t = 15, help = "Number of edges for random graphs")]
    num_edges: usize,
}
