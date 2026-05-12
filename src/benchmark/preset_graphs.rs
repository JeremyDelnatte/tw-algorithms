use std::{fs::{File, create_dir_all}, io::{BufRead, Write, stdout}};

use csv::Writer;

use crate::{benchmark::{run_algorithm, run_algorithm_timeout}, graph::{self, adjlist}, treewidth::exact::ExactAlgorithm};

pub fn run_preset_graphs_benchmark(algorithm: &ExactAlgorithm, graph_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = std::path::Path::new(graph_file);
    let filename = input_path.file_name().ok_or("Invalid graph file path")?.to_string_lossy();

    let input_file = std::fs::File::open(input_path)?;
    let reader = std::io::BufReader::new(input_file);

    create_dir_all("benchmarks/instances")?;
    let output_file = File::create(format!("benchmarks/instances/{:?}_{}.csv", algorithm, filename))?;
    let mut writer = Writer::from_writer(output_file);

    let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
    let num_graphs = lines.len();

    let mut mean_runtime = 0.0;

    // for (i, line) in lines.into_iter().enumerate() {
    //     let g6 = line;
    //
    //     let graph = graph::Graph::AdjList(adjlist::Graph::from_g6(&g6)?);
    //     let runtime = run_algorithm(algorithm, &graph, &mut writer);
    //
    //     mean_runtime = ((mean_runtime * i as f64) + runtime) / ((i + 1) as f64);
    //     let eta = mean_runtime * (num_graphs - i - 1) as f64;
    //
    //     print!("Graphs {}/{}: Runtime = {:.2} seconds, Mean Runtime = {:.2} seconds, ETA = {:.2} seconds\r", i + 1, num_graphs, runtime, mean_runtime, eta);
    //     stdout().flush()?;
    // }
    for (i, line) in lines.into_iter().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (graph_name, g6) = match parts.as_slice() {
            [g6] => (None, *g6),
            [name, g6] => (Some(*name), *g6),
            _ => return Err(format!("Invalid line format: {}", line).into()),
        };

        let graph = graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?);
        let runtime = run_algorithm(algorithm, &graph, &mut writer, graph_name);

        mean_runtime = ((mean_runtime * i as f64) + runtime) / ((i + 1) as f64);
        let eta = mean_runtime * (num_graphs - i - 1) as f64;

        print!("Graphs {}/{}", i + 1, num_graphs);

        if let Some(name) = graph_name {
            print!(" ({})", name);
        }

        print!(
            ": Runtime = {:.2} seconds, Mean Runtime = {:.2} seconds, ETA = {:.2} seconds\r",
            runtime,
            mean_runtime,
            eta
        );
        stdout().flush()?;
    }
    println!();
    Ok(())
}

pub async fn run_preset_graphs_benchmark_timeout(algorithm: &ExactAlgorithm, graph_file: &str, timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = std::path::Path::new(graph_file);
    let filename = input_path.file_name().ok_or("Invalid graph file path")?.to_string_lossy();

    let input_file = std::fs::File::open(input_path)?;
    let reader = std::io::BufReader::new(input_file);

    create_dir_all("benchmarks/instances")?;
    let output_file = File::create(format!("benchmarks/instances/{:?}_{}.csv", algorithm, filename))?;
    let mut writer = Writer::from_writer(output_file);

    let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
    let num_graphs = lines.len();

    let mut mean_runtime = 0.0;

    for (i, line) in lines.into_iter().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (graph_name, g6) = match parts.as_slice() {
            [g6] => (None, *g6),
            [name, g6] => (Some(*name), *g6),
            ["#", ..] => continue,
            _ => return Err(format!("Invalid line format: {}", line).into()),
        };

        let graph = graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?);
        let runtime = run_algorithm_timeout(algorithm, &graph, &mut writer, graph_name, timeout).await;

        print!("Graphs {}/{}", i + 1, num_graphs);

        if let Some(name) = graph_name {
            print!(" ({})", name);
        }

        if let Some(runtime) = runtime {
            mean_runtime = ((mean_runtime * i as f64) + runtime) / ((i + 1) as f64);
            let eta = mean_runtime * (num_graphs - i - 1) as f64;

            print!(
                ": Runtime = {:.2} seconds, Mean Runtime = {:.2} seconds, ETA = {:.2} seconds\r",
                runtime,
                mean_runtime,
                eta
            );
        } else {
            print!(": Timeout after {:.2} seconds\r", timeout.as_secs_f64());
        }
        stdout().flush()?;
    }
    println!();
    Ok(())
}
