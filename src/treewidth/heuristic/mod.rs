use serde::Serialize;
use strum::EnumIter;

use crate::
    graph::{Graph, adjlist, newbitset}
;

#[derive(EnumIter, Serialize, Debug, Clone, Copy)]
pub enum HeuristicAlgorithm {
    MinFill,
    MinDegree,
    MinDegreePlusFill,
    MinSparsestSubgraph,
    MinFillDegree,
    MinDegreeFill,
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
        Graph::AdjList(g) => min_heuristic(g, |g| g.min_fill_in_count_vertex()),
        Graph::NewBitSet(g) => min_heuristic_bitset(g, |g| g.min_fill_in_count_vertex()),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}

pub fn min_degree(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| g.min_degree_vertex()),
        Graph::NewBitSet(g) => min_heuristic_bitset(g, |g| g.min_degree_vertex()),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}

pub fn min_degree_plus_fill(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| {
            let mut min = g.degree(0) + g.fill_in_count_vertex(0);
            let mut vertex_min = 0;

            for v in 1..g.n() {
                let degree_plus_fill = g.degree(v) + g.fill_in_count_vertex(v);
                if degree_plus_fill < min {
                    min = degree_plus_fill;
                    vertex_min = v;
                }
            }

            vertex_min
        }),
        Graph::NewBitSet(g) => min_heuristic_bitset(g, |g| {
            let mut min = g.degree(0) + g.fill_in_count_vertex(0);
            let mut vertex_min = 0;

            for v in 1..g.n() {
                let degree_plus_fill = g.degree(v) + g.fill_in_count_vertex(v);
                if degree_plus_fill < min {
                    min = degree_plus_fill;
                    vertex_min = v;
                }
            }

            vertex_min
        }),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}

pub fn min_sparsest_subgraph(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| {
            let mut min = g.fill_in_count_vertex(0) - g.degree(0);
            let mut vertex_min = 0;

            for v in 1..g.n() {
                let sparsity = g.fill_in_count_vertex(v) - g.degree(v);
                if sparsity < min {
                    min = sparsity;
                    vertex_min = v;
                }
            }

            vertex_min
        }),
        Graph::NewBitSet(g) => min_heuristic_bitset(g, |g| {
            let mut min = g.fill_in_count_vertex(0) - g.degree(0);
            let mut vertex_min = 0;

            for v in 1..g.n() {
                let sparsity = g.fill_in_count_vertex(v) - g.degree(v);
                if sparsity < min {
                    min = sparsity;
                    vertex_min = v;
                }
            }

            vertex_min
        }),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}

pub fn min_fill_degree(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| {
            let score = |v: usize| {
                let n = g.n() as f64;
                g.degree(v) as f64 + g.fill_in_count_vertex(v) as f64 / (n * n)
            };

            let mut min = score(0);
            let mut min_vertex = 0;

            for v in 1..g.n() {
                let s = score(v);

                if s < min {
                    min = s;
                    min_vertex = v;
                }
            }

            min_vertex
        }),
        Graph::NewBitSet(g) => min_heuristic_bitset(g, |g| {
            let score = |v: usize| {
                let n = g.n() as f64;
                g.degree(v) as f64 + g.fill_in_count_vertex(v) as f64 / (n * n)
            };

            let mut min = score(0);
            let mut min_vertex = 0;

            for v in 1..g.n() {
                let s = score(v);

                if s < min {
                    min = s;
                    min_vertex = v;
                }
            }

            min_vertex
        }),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}

pub fn min_degree_fill(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| {
            let score = |v: usize| {
                let n = g.n() as f64;
                g.fill_in_count_vertex(v) as f64 + g.degree(v) as f64 / n
            };

            let mut min = score(0);
            let mut min_vertex = 0;

            for v in 1..g.n() {
                let s = score(v);

                if s < min {
                    min = s;
                    min_vertex = v;
                }
            }

            min_vertex
        }),
        Graph::NewBitSet(g) => min_heuristic_bitset(g, |g| {
            let score = |v: usize| {
                let n = g.n() as f64;
                g.fill_in_count_vertex(v) as f64 + g.degree(v) as f64 / n
            };

            let mut min = score(0);
            let mut min_vertex = 0;

            for v in 1..g.n() {
                let s = score(v);

                if s < min {
                    min = s;
                    min_vertex = v;
                }
            }

            min_vertex
        }),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}
