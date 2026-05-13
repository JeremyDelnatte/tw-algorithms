use std::time::Duration;

use tw_algorithms::{
    graph::{self, adjlist}, treewidth::approx::{ApproxAlgorithm, four_approx},
};

pub fn approximate_treewidth(
    g6: &str,
    algorithm: ApproxAlgorithm,
    with_bitset: bool,
) -> Result<(usize, Duration), Box<dyn std::error::Error>> {
    let g = if !with_bitset {
        graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?)
    } else {
        graph::Graph::NewBitSet(graph::newbitset::Graph::from_g6(g6)?)
    };

    let start_time = std::time::Instant::now();
    let tw = match algorithm {
        ApproxAlgorithm::FourApprox => four_approx::approx_treewidth(&g),
    };
    let duration = start_time.elapsed();
    Ok((tw, duration))
}
