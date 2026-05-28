use std::time::Duration;

use crate::{
    graph::{self, adjlist},
    treewidth::{
        approx::{ApproxAlgorithm, four_approx, four_half_approx},
        exact::{ExactAlgorithm, branch_bound, dynamic_prog, improved_rec, rec},
        heuristic::HeuristicAlgorithm,
    },
};

pub mod exact;
pub mod approx;
pub mod heuristic;

pub enum Algorithm {
    Exact(ExactAlgorithm),
    Approx(ApproxAlgorithm),
    Heuristic(HeuristicAlgorithm),
}

pub fn compute_or_approximate_treewidth(
    g6: &str,
    algorithm: Algorithm,
    with_bitset: bool,
) -> Result<(usize, Duration), Box<dyn std::error::Error>> {
    match algorithm {
        Algorithm::Exact(exact_alg) => compute_treewidth(g6, exact_alg, with_bitset),
        Algorithm::Approx(approx_alg) => approximate_treewidth(g6, approx_alg, with_bitset),
        Algorithm::Heuristic(heuristic_alg) => heuristic_treewidth(g6, heuristic_alg, with_bitset),
    }
}

pub fn compute_treewidth(
    g6: &str,
    algorithm: ExactAlgorithm,
    with_bitset: bool,
) -> Result<(usize, Duration), Box<dyn std::error::Error>> {
    let g = if with_bitset {
        graph::Graph::BitSet(graph::bitset::Graph::from_g6(g6)?)
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
        graph::Graph::BitSet(graph::bitset::Graph::from_g6(g6)?)
    };

    let start_time = std::time::Instant::now();
    let tw = match algorithm {
        ApproxAlgorithm::FourApprox => four_approx::approx_treewidth(&g),
        ApproxAlgorithm::FourHalfApprox => four_half_approx::approx_treewidth(&g),
    };
    let duration = start_time.elapsed();
    Ok((tw, duration))
}

pub fn heuristic_treewidth(
    g6: &str,
    algorithm: HeuristicAlgorithm,
    with_bitset: bool,
) -> Result<(usize, Duration), Box<dyn std::error::Error>> {
    let g = if with_bitset {
        graph::Graph::BitSet(graph::bitset::Graph::from_g6(g6)?)
    } else {
        graph::Graph::AdjList(adjlist::Graph::from_g6(g6)?)
    };

    let start_time = std::time::Instant::now();
    let tw = match algorithm {
        HeuristicAlgorithm::MinFill => heuristic::min_fill(&g),
        HeuristicAlgorithm::MinDegree => heuristic::min_degree(&g),
        HeuristicAlgorithm::MinDegreePlusFill => heuristic::min_degree_plus_fill(&g),
        HeuristicAlgorithm::MinSparsestSubgraph => heuristic::min_sparsest_subgraph(&g),
        HeuristicAlgorithm::MinFillDegree => heuristic::min_fill_degree(&g),
        HeuristicAlgorithm::MinDegreeFill => heuristic::min_degree_fill(&g),
    };
    let duration = start_time.elapsed();
    Ok((tw, duration))
}
