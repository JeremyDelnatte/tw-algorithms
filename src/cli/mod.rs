use clap::{ArgGroup, Parser, Subcommand, ValueEnum};

pub mod compute_treewidth;
pub mod approximate_treewidth;

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
            Command::ApproximateTreewidth(approx_args) => {
                match approx_args.input_type() {
                    InputType::SingleGraph(g6) => approximate_treewidth::approximate_treewidth_single(
                        &g6,
                        approx_args.algorithm,
                        self.with_bitset,
                        approx_args.optimal_treewidth,
                    ),
                    InputType::GraphsFile(filename) => approximate_treewidth::approximate_treewidth_file(
                        &filename,
                        approx_args.algorithm,
                        self.with_bitset,
                        approx_args.optimal_treewidth,
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

    #[command(visible_aliases = ["atw", "approx", "a", "apx"])]
    ApproximateTreewidth(ApproximateTreewidthArgs),

    // #[command(visible_aliases = ["bm", "b"])]
    // Benchmark,
}

#[derive(Clone, ValueEnum, Debug)]
enum ExactAlgorithmArg {
    #[value(alias("dp"))]
    DynamicProg,

    #[value(alias("rec"))]
    Recursive,

    #[value(alias("imprec"))]
    ImprovedRec,

    #[value(alias("bb"))]
    BranchBound,
}

impl From<ExactAlgorithmArg> for tw_algorithms::treewidth::exact::ExactAlgorithm {
    fn from(arg: ExactAlgorithmArg) -> Self {
        match arg {
            ExactAlgorithmArg::DynamicProg => Self::DynamicProg,
            ExactAlgorithmArg::Recursive => Self::Recursive,
            ExactAlgorithmArg::ImprovedRec => Self::ImprovedRec,
            ExactAlgorithmArg::BranchBound => Self::BranchBound,
        }
    }
}

#[derive(Clone, ValueEnum, Debug)]
enum ApproxAlgorithmArg {
    #[value(alias("4apx"))]
    FourApprox,
}

impl From<ApproxAlgorithmArg> for tw_algorithms::treewidth::approx::ApproxAlgorithm {
    fn from(arg: ApproxAlgorithmArg) -> Self {
        match arg {
            ApproxAlgorithmArg::FourApprox => Self::FourApprox,
        }
    }
}

enum InputType {
    SingleGraph(String),
    GraphsFile(String),
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
    #[arg(short = 'a', long, value_enum, default_value_t = ExactAlgorithmArg::DynamicProg)]
    algorithm: ExactAlgorithmArg,

    #[arg(short = 'f', long = "file")]
    graphs_file: Option<String>,

    #[arg(short = 'g', long)]
    graph: Option<String>,

    #[arg(short = 't', long = "treewidth")]
    expected_treewidth: Option<usize>,
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
#[command(
    group(
        ArgGroup::new("input")
            .required(true)
            .args(["graph", "graphs_file"])
    )
)] // Ensure that exactly one of --graph or --file is provided
struct ApproximateTreewidthArgs {
    #[arg(short = 'a', long, value_enum, default_value_t = ApproxAlgorithmArg::FourApprox)]
    algorithm: ApproxAlgorithmArg,

    #[arg(short = 'f', long = "file")]
    graphs_file: Option<String>,

    #[arg(short = 'g', long)]
    graph: Option<String>,

    #[arg(short = 't', long = "treewidth")]
    optimal_treewidth: Option<usize>,
}

impl ApproximateTreewidthArgs {
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

// #[derive(Parser)]
// struct BenchmarkArgs {
//     #[arg(short = 'a', long, value_enum, default_value_t = ExactAlgorithmArg::DynamicProg)]
//     algorithm: ExactAlgorithmArg,
//
//     #[arg(short = 'r', long = "random", help = "Whether to generate random graphs for benchmarking")]
//     generate_random_graphs: bool,
//
//     #[arg(short = 'i', long = "iterations", default_value_t = 100, help = "Number of iterations for benchmarking")]
//     num_iterations: usize,
//
//     #[arg(short = 'n', long = "vertices", default_value_t = 10, help = "Number of vertices for random graphs")]
//     num_vertices: usize,
//
//     #[arg(short = 'm', long = "edges", default_value_t = 15, help = "Number of edges for random graphs")]
//     num_edges: usize,
// }
