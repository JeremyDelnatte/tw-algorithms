use tw_algorithms::{
    graph::{self, adjlist}, treewidth::exact::{ExactAlgorithm, branch_bound, dynamic_prog, improved_rec, rec},
};

pub fn compute_treewidth(
    g6: &str,
    algorithm: ExactAlgorithm,
    with_bitset: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let g = if !with_bitset {
        graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?)
    } else {
        graph::Graph::from_g6(g6)?
    };

    Ok(match algorithm {
        ExactAlgorithm::DynamicProg => dynamic_prog::treewidth(&g),
        ExactAlgorithm::Recursive => rec::treewidth(&g),
        ExactAlgorithm::ImprovedRec => improved_rec::treewidth(&g),
        ExactAlgorithm::BranchBound => branch_bound::treewidth(&g),
    })
}
