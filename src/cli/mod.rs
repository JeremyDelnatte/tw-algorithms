//! Defines the command-line interface.

use std::time::Duration;

use clap::{Parser, Subcommand};

use crate::cli::{
    approximate_treewidth::ApproximateTreewidthArgs, benchmark::BenchmarkArgs,
    compute_treewidth::ComputeTreewidthArgs, heuristic_treewidth::HeuristicTreewidthArgs,
};

pub mod approximate_treewidth;
pub mod benchmark;
pub mod compute_treewidth;
pub mod heuristic_treewidth;
pub mod progress_bar;

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

    #[arg(
        long = "timeout",
        global = true,
        help = "Set a timeout for treewidth computations (e.g., 30s, 5m, 1h)",
        value_parser = parse_duration,
    )]
    timeout: Option<Duration>,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Command::ComputeTreewidth(args) => {
                compute_treewidth::run(args, self.with_bitset, self.timeout)
            }
            Command::ApproximateTreewidth(args) => {
                approximate_treewidth::run(args, self.with_bitset, self.timeout)
            }
            Command::HeuristicTreewidth(args) => {
                heuristic_treewidth::run(args, self.with_bitset, self.timeout)
            }
            Command::Benchmark(args) => benchmark::run(args, self.with_bitset, self.timeout),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    #[command(visible_aliases = ["tw"])]
    ComputeTreewidth(ComputeTreewidthArgs),

    #[command(visible_aliases = ["atw", "approx", "a", "apx"])]
    ApproximateTreewidth(ApproximateTreewidthArgs),

    #[command(visible_aliases = ["htw", "heuristic", "h", "heu"])]
    HeuristicTreewidth(HeuristicTreewidthArgs),

    #[command(visible_aliases = ["bm", "b"])]
    Benchmark(BenchmarkArgs),
}

enum InputType {
    SingleGraph(String),
    GraphsFile(String),
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("timeout cannot be empty".to_string());
    }

    let split_at = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());

    let number = &s[..split_at];
    let unit = &s[split_at..];
    let value: u64 = number
        .parse()
        .map_err(|_| format!("invalid timeout number: {number}"))?;

    match unit {
        "" | "s" | "sec" | "secs" | "second" | "seconds" => Ok(Duration::from_secs(value)),
        "m" | "min" | "mins" | "minute" | "minutes" => Ok(Duration::from_secs(value * 60)),
        "h" | "hr" | "hrs" | "hour" | "hours" => Ok(Duration::from_secs(value * 60 * 60)),
        "ms" => Ok(Duration::from_millis(value)),
        "ns" => Ok(Duration::from_nanos(value)),
        "us" => Ok(Duration::from_micros(value)),
        _ => Err(format!("invalid timeout unit: {unit}")),
    }
}
