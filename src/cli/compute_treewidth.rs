use std::io::{BufRead, Write, stdout};

use crate::{cli::ExactAlgorithmArg, commands};

pub(super) fn compute_treewidth_single(
    g6: &str,
    algorithm: ExactAlgorithmArg,
    with_bitset: bool,
    expected_treewidth: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Computing treewidth using {:?} algorithm (bitset: {}) on the graph {}",
        algorithm, with_bitset, g6
    );
    let tw = commands::compute_treewidth(g6, algorithm.into(), with_bitset)?;

    if let Some(expected_tw) = expected_treewidth
        && tw != expected_tw
    {
        // TODO: Should not panic, but return an error instead.
        panic!("Expected treewidth 4, got {} with graph {}", tw, g6);
    }

    println!("Computed treewidth: {}", tw);
    return Ok(());
}

pub(super) fn compute_treewidth_file(
    filename: &str,
    algorithm: ExactAlgorithmArg,
    with_bitset: bool,
    expected_treewidth: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut count = 0;
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);

    println!(
        "Computing treewidth using {:?} algorithm (bitset: {}) on the graphs from {}",
        algorithm, with_bitset, filename
    );

    let instant = std::time::Instant::now();

    for line in reader.lines() {
        let g6 = line?;

        let tw = commands::compute_treewidth(&g6, algorithm.clone().into(), with_bitset)?;

        if let Some(expected_tw) = expected_treewidth
            && tw != expected_tw
        {
            // TODO: Should not panic, but return an error instead.
            panic!("\nExpected treewidth 4, got {} with graph {}", tw, g6);
        }

        print!("\rProcessed {} graphs", count);
        stdout().flush()?;
        count += 1;
    }

    let elapsed = instant.elapsed();
    println!(
        "\nAll {} graphs processed successfully in {:.2?}",
        count, elapsed
    );

    Ok(())
}
