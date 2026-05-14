use std::{
    io::{BufRead, Write, stdout},
    time::Duration,
};

use clap::{ArgGroup, Parser, ValueEnum};
use serde_json::json;
use tw_algorithms::treewidth;

use crate::{
    cli::InputType,
    timeout::{self, TreewidthProcessError},
};

#[derive(Clone, ValueEnum, Debug, Copy)]
pub enum HeuristicAlgorithmArg {
    #[value(alias("mf"))]
    MinFill,
}

impl From<HeuristicAlgorithmArg> for tw_algorithms::treewidth::heuristic::HeuristicAlgorithm {
    fn from(arg: HeuristicAlgorithmArg) -> Self {
        match arg {
            HeuristicAlgorithmArg::MinFill => Self::MinFill,
        }
    }
}

impl std::fmt::Display for HeuristicAlgorithmArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HeuristicAlgorithmArg::MinFill => "min-fill",
        };
        write!(f, "{}", s)
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
pub(super) struct HeuristicTreewidthArgs {
    #[arg(short = 'a', long, value_enum, default_value_t = HeuristicAlgorithmArg::MinFill)]
    algorithm: HeuristicAlgorithmArg,

    #[arg(short = 'f', long = "file")]
    graphs_file: Option<String>,

    #[arg(short = 'g', long)]
    graph: Option<String>,

    #[arg(short = 't', long = "treewidth")]
    optimal_treewidth: Option<usize>,

    #[arg(
        long = "json",
        global = true,
        help = "Output results in JSON format, including the approximated treewidth and execution time in nanoseconds"
    )]
    output_json: bool,
}

impl HeuristicTreewidthArgs {
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

pub(super) fn run(
    args: HeuristicTreewidthArgs,
    with_bitset: bool,
    timeout_opt: Option<Duration>,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.input_type() {
        InputType::SingleGraph(g6) => heuristic_treewidth_single(
            &g6,
            args.algorithm,
            with_bitset,
            args.optimal_treewidth,
            args.output_json,
            timeout_opt,
        ),
        InputType::GraphsFile(filename) => heuristic_treewidth_file(
            &filename,
            args.algorithm,
            with_bitset,
            args.optimal_treewidth,
            args.output_json,
            timeout_opt,
        ),
    }
}

fn heuristic_treewidth_single(
    g6: &str,
    algorithm: HeuristicAlgorithmArg,
    with_bitset: bool,
    optimal_treewidth: Option<usize>,
    output_json: bool,
    timeout_opt: Option<Duration>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !output_json {
        println!(
            "Computing heuristic treewidth using {:?} algorithm (bitset: {}) on the graph {}",
            algorithm, with_bitset, g6
        );
    }

    let (tw, duration) = if let Some(timeout) = timeout_opt {
        match timeout::heuristic_treewidth(g6, algorithm, with_bitset, timeout) {
            Ok(result) => result,
            Err(TreewidthProcessError::Timeout { timeout }) => {
                if output_json {
                    let output = json!({
                        "status": "timeout",
                        "duration_ns": timeout.as_nanos(),
                    });
                    println!("{}", output);
                } else {
                    println!("Heuristic computation timed out after {:.2?}", timeout);
                }

                return Ok(());
            }
            Err(e) => return Err(e.into()),
        }
    } else {
        treewidth::heuristic_treewidth(g6, algorithm.into(), with_bitset)?
    };

    if let Some(optimal_tw) = optimal_treewidth
        && optimal_tw > tw
    {
        panic!("Optimal treewidth {optimal_tw} is greater than heuristic treewidth {tw} with graph {g6}");
    }

    if !output_json {
        println!("Computed heuristic treewidth: {}, Time taken: {:.2?}", tw, duration);
    } else {
        let output = json!({
            "status": "ok",
            "treewidth": tw,
            "duration_ns": duration.as_nanos()
        });
        println!("{}", output);
    }

    Ok(())
}

fn heuristic_treewidth_file(
    filename: &str,
    algorithm: HeuristicAlgorithmArg,
    with_bitset: bool,
    optimal_treewidth: Option<usize>,
    output_json: bool,
    timeout_opt: Option<Duration>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut count = 0;
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);

    if !output_json {
        println!(
            "Computing heuristic treewidth using {:?} algorithm (bitset: {}) on the graphs from {}",
            algorithm, with_bitset, filename
        );
    }

    let instant = std::time::Instant::now();
    let mut num_timeouts = 0;

    for line in reader.lines() {
        let g6 = line?;

        let (tw, _) = if let Some(timeout) = timeout_opt {
            match timeout::heuristic_treewidth(&g6, algorithm, with_bitset, timeout) {
                Ok(result) => result,
                Err(TreewidthProcessError::Timeout { .. }) => {
                    num_timeouts += 1;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            treewidth::heuristic_treewidth(&g6, algorithm.into(), with_bitset)?
        };

        if let Some(optimal_tw) = optimal_treewidth
            && optimal_tw > tw
        {
            panic!("\nOptimal treewidth {optimal_tw} is greater than heuristic treewidth {tw} with graph {g6}");
        }

        if !output_json {
            print!("\rProcessed {} graphs", count);
            stdout().flush()?;
        }
        count += 1;
    }

    if timeout_opt.is_some() {
        if !output_json {
            println!(
                "\nAll {} graphs processed successfully, but {} timed out, in {:.2?}",
                count,
                num_timeouts,
                instant.elapsed()
            );
        } else {
            let output = json!({
                "num_graphs": count,
                "num_timeouts": num_timeouts,
                "duration_ns": instant.elapsed().as_nanos()
            });
            println!("{}", output);
        }
        return Ok(());
    }

    let elapsed = instant.elapsed();
    if !output_json {
        println!(
            "\nAll {} graphs processed successfully in {:.2?}",
            count, elapsed
        );
    } else {
        let output = json!({
            "num_graphs": count,
            "duration_ns": elapsed.as_nanos()
        });
        println!("{}", output);
    }

    Ok(())
}
