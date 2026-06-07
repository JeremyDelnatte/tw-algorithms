//! Heuristic treewidth algorithms based on linear orderings. It chooses the next vertex to
//! eliminated based on a heuristic score, such as the number of fill-in edges that would be added
//! by eliminating it.

use serde::Serialize;
use strum::EnumIter;

use crate::graph::{Graph, adjlist, bitset};

/// Heuristic treewidth algorithms based on linear orderings.
#[derive(EnumIter, Serialize, Debug, Clone, Copy)]
pub enum HeuristicAlgorithm {
    /// Eliminates a vertex with minimum fill-in.
    MinFill,

    /// Eliminates a vertex with minimum degree.
    MinDegree,

    /// Eliminates a vertex minimizing the sum of degree and fill-in.
    MinDegreePlusFill,

    /// Eliminates a vertex minimizing fill-in minus degree.
    MinSparsestSubgraph,

    /// Eliminates a vertex with minimum fill-in, breaking ties by degree.
    MinFillDegree,

    /// Eliminates a vertex with minimum degree, breaking ties by fill-in.
    MinDegreeFill,
}

// Helper function to compute the width of a heuristic linear ordering. The `vertex_selector`
// function is used to select the next vertex to eliminate at each step of the algorithm.
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

// Helper function to compute the width of a heuristic linear ordering for bitset-based graphs. The
// `vertex_selector` function is used to select the next vertex to eliminate at each step of the
// algorithm.
fn min_heuristic_bitset(
    g: &bitset::Graph,
    vertex_selector: impl Fn(&bitset::Graph) -> usize,
) -> usize {
    let mut g = g.clone();
    let mut max_degree = 0;

    while g.n() > 0 {
        let v = vertex_selector(&g);
        max_degree = max_degree.max(g.degree(v));
        g.elim_vertex(v);
    }
    max_degree
}

/// Computes the width of the min-fill elimination heuristic.
pub fn min_fill(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| g.min_fill_in_count_vertex()),
        Graph::BitSet(g) => min_heuristic_bitset(g, |g| g.min_fill_in_count_vertex()),
    }
}

/// Computes the width of the min-degree elimination heuristic.
pub fn min_degree(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => min_heuristic(g, |g| g.min_degree_vertex()),
        Graph::BitSet(g) => min_heuristic_bitset(g, |g| g.min_degree_vertex()),
    }
}

/// Computes the width of the minimum degree-plus-fill heuristic.
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
        Graph::BitSet(g) => min_heuristic_bitset(g, |g| {
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
    }
}

/// Computes the width of the minimum sparsest-subgraph heuristic.
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
        Graph::BitSet(g) => min_heuristic_bitset(g, |g| {
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
    }
}

/// Computes the width of the min-fill heuristic with degree-based tie-breaking.
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
        Graph::BitSet(g) => min_heuristic_bitset(g, |g| {
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
    }
}

/// Computes the width of the min-degree heuristic with fill-in-based tie-breaking.
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
        Graph::BitSet(g) => min_heuristic_bitset(g, |g| {
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
    }
}
