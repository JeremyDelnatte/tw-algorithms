use std::time::Duration;

use crate::{graph::{self, adjlist}, treewidth::{approx::{ApproxAlgorithm, four_approx}, exact::{ExactAlgorithm, branch_bound, dynamic_prog, improved_rec, rec}}};

pub mod exact;
pub mod approx;
pub mod heuristic;

pub enum Algorithm {
    Exact(ExactAlgorithm),
    Approx(ApproxAlgorithm),
}

pub fn compute_or_approximate_treewidth(
    g6: &str,
    algorithm: Algorithm,
    with_bitset: bool,
) -> Result<(usize, Duration), Box<dyn std::error::Error>> {
    match algorithm {
        Algorithm::Exact(exact_alg) => compute_treewidth(g6, exact_alg, with_bitset),
        Algorithm::Approx(approx_alg) => approximate_treewidth(g6, approx_alg, with_bitset),
    }
}

pub fn compute_treewidth(
    g6: &str,
    algorithm: ExactAlgorithm,
    with_bitset: bool,
) -> Result<(usize, Duration), Box<dyn std::error::Error>> {
    let g = if with_bitset {
        graph::Graph::NewBitSet(graph::newbitset::Graph::from_g6(g6)?)
    } else {
        graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?)
    };

    let start_time = std::time::Instant::now();
    let tw = match algorithm {
        ExactAlgorithm::DynamicProg => dynamic_prog::treewidth(&g),
        ExactAlgorithm::Recursive => rec::treewidth(&g),
        ExactAlgorithm::ImprovedRec => improved_rec::treewidth(&g),
        ExactAlgorithm::BranchBound => branch_bound::treewidth(&g),
    };
    let duration = start_time.elapsed();
    Ok((tw, duration))
}

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
