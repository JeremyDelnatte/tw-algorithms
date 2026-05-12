use tw_algorithms::{
    graph::{self, adjlist}, treewidth::approx::{ApproxAlgorithm, four_approx},
};

pub fn approximate_treewidth(
    g6: &str,
    algorithm: ApproxAlgorithm,
    with_bitset: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let g = if !with_bitset {
        graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?)
    } else {
        todo!("Bitset-based graph representation is not yet implemented for approximate treewidth algorithms");
    };

    Ok(match algorithm {
        ApproxAlgorithm::FourApprox => four_approx::approx_treewidth(&g),
    })
}
