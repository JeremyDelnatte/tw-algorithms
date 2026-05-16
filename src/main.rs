use clap::Parser;

mod cli;
mod timeout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // test();
    // Ok(())

    // let n_values = [10, 15, 20];
    // let m_values = [20, 30, 40];

    // let n_values = [11, 20];
    // let m_values = [20, 30];
    //
    // for n in n_values {
    //     for m in m_values {
    //         println!("Running benchmark for n = {}, m = {}", n, m);
    //         let benchmark_config = tw_algorithms::benchmark::BenchmarkConfig {
    //             algorithm: tw_algorithms::treewidth::Algorithm::Recursive,
    //             benchmark_type: tw_algorithms::benchmark::BenchmarkType::RandomGraphs {
    //                 seed: 42,
    //                 num_iterations: 100,
    //                 num_vertices: n,
    //                 num_edges: m,
    //             },
    //         };
    //         run_benchmarks(benchmark_config)?;
    //     }
    // }

    // for n in 3..=3 {
    //     let graph_file = format!("instances/house_of_graphs/graphs_n{}.g6", n);
    //
    //     let benchmark_config = tw_algorithms::benchmark::BenchmarkConfig {
    //         algorithm: tw_algorithms::treewidth::Algorithm::BranchBound,
    //         benchmark_type: tw_algorithms::benchmark::BenchmarkType::PresetGraphs {
    //             graph_file
    //         },
    //     };
    //     run_benchmarks(benchmark_config)?;
    // }
    // Ok(())

    // let graph_file = "instances/dimacs/dimacs.g6".to_string();
    //
    // let benchmark_config = tw_algorithms::benchmark::BenchmarkConfig {
    //     algorithm: tw_algorithms::treewidth::Algorithm::Recursive,
    //     benchmark_type: tw_algorithms::benchmark::BenchmarkType::PresetGraphs {
    //         graph_file
    //     },
    // };
    //
    // let timeout_duration = std::time::Duration::from_secs(10 * 60);
    // run_benchmarks_timeout(benchmark_config, timeout_duration).await

    // run_benchmarks(benchmark_config)

    cli::Cli::parse().run()
}
