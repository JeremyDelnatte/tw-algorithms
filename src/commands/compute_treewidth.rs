use tw_algorithms::{
    graph::{self, adjlist},
    treewidth::{self, Algorithm, dynamic_prog, improved_rec, rec},
};

pub fn compute_treewidth(
    g6: &str,
    algorithm: Algorithm,
    with_bitset: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let g = if !with_bitset {
        graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?)
    } else {
        graph::Graph::from_g6(g6)?
    };

    Ok(match algorithm {
        Algorithm::DynamicProg => dynamic_prog::treewidth(&g),
        Algorithm::Recursive => rec::treewidth(&g),
        Algorithm::ImprovedRec => improved_rec::treewidth(&g),
        Algorithm::BranchBound => treewidth::branch_bound::treewidth(&g),
    })
}
