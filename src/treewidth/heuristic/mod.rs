use serde::Serialize;
use strum::EnumIter;

use crate::
    graph::{Graph, adjlist, newbitset}
;

#[derive(EnumIter, Serialize, Debug, Clone, Copy)]
pub enum HeuristicAlgorithm {
    MinFill,
}

fn min_heuristic(g: &adjlist::Graph, vertex_selector: impl Fn(&adjlist::Graph) -> usize) -> usize {
    let mut g = g.clone();
    let mut max_degree = 0;

    while !g.is_empty() {
        let v = vertex_selector(&g);
        max_degree = max_degree.max(g.degree(v));
        g.elim_vertex(v);
    }
    max_degree
}

fn min_heuristic_bitset(g: &newbitset::Graph, vertex_selector: impl Fn(&newbitset::Graph) -> usize) -> usize {
    let mut g = g.clone();
    let mut max_degree = 0;

    while g.n() > 0 {
        let v = vertex_selector(&g);
        max_degree = max_degree.max(g.degree(v));
        g.elim_vertex(v);
    }
    max_degree
}

pub fn min_fill(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| g.least_fill_in_count_vertex()),
        Graph::NewBitSet(g) => min_heuristic_bitset(g, |g| g.least_fill_in_count_vertex()),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}
