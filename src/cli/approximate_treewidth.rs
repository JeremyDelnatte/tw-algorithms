use std::io::{BufRead, Write, stdout};

use tw_algorithms::treewidth::approx::ApproxAlgorithm;

use crate::{cli::ApproxAlgorithmArg, commands};

pub(super) fn approximate_treewidth_single(
    g6: &str,
    algorithm: ApproxAlgorithmArg,
    with_bitset: bool,
    optimal_treewidth: Option<usize>,
    time_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let algorithm: ApproxAlgorithm = algorithm.into();

    if !time_only {
        println!(
            "Approximating treewidth using {:?} algorithm (bitset: {}) on the graph {}",
            algorithm, with_bitset, g6
        );
    }
    let (tw, duration) = commands::approximate_treewidth(g6, algorithm, with_bitset)?;

    if let Some(optimal_tw) = optimal_treewidth {
        if tw < optimal_tw {
            panic!("Approximated treewidth {} is less than the optimal treewidth {} for graph {}", tw, optimal_tw, g6);
        }

        if tw > algorithm.worst_case_from_optimal(optimal_tw) {
            panic!("Approximated treewidth {} is greater than the worst case treewidth {} for optimal treewidth {} with algorithm {:?}", tw, algorithm.worst_case_from_optimal(optimal_tw), optimal_tw, algorithm);
        }
    }

    if !time_only {
        println!("Approximated treewidth: {}, Time taken: {:.2?}", tw, duration);
    } else {
        println!("{}", duration.as_nanos());
    }
    return Ok(());
}

pub(super) fn approximate_treewidth_file(
    filename: &str,
    algorithm: ApproxAlgorithmArg,
    with_bitset: bool,
    optimal_treewidth: Option<usize>,
    time_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut count = 0;
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    let algorithm: ApproxAlgorithm = algorithm.into();

    if !time_only {
        println!(
            "Approximating treewidth using {:?} algorithm (bitset: {}) on the graphs from {}",
            algorithm, with_bitset, filename
        );
    }

    let instant = std::time::Instant::now();

    for line in reader.lines() {
        let g6 = line?;

        let (tw, _) = commands::approximate_treewidth(&g6, algorithm, with_bitset)?;

        if let Some(optimal_tw) = optimal_treewidth {
            if tw < optimal_tw {
                panic!("\nApproximated treewidth {} is less than the optimal treewidth {} for graph {}", tw, optimal_tw, g6);
            }

            if tw > algorithm.worst_case_from_optimal(optimal_tw) {
                panic!("\nApproximated treewidth {} is greater than the worst case treewidth {} for optimal treewidth {} with algorithm {:?}", tw, algorithm.worst_case_from_optimal(optimal_tw), optimal_tw, algorithm);
            }
        }

        if !time_only {
            print!("\rProcessed {} graphs", count);
            stdout().flush()?;
        }
        count += 1;
    }

    let elapsed = instant.elapsed();
    if !time_only {
        println!(
            "\nAll {} graphs processed successfully in {:.2?}",
            count, elapsed
        );
    } else {
        println!("{}", elapsed.as_nanos());
    }

    Ok(())
}
