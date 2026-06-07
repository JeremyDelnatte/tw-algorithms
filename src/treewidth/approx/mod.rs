//! Separator-based approximation algorithms for treewidth.

use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use serde::Serialize;
use strum::EnumIter;

use crate::{
    graph::{self, adjlist, bitset},
    utils::{bitset::BitSet, max_flow::max_flow_reachable},
};

pub mod four_approx;
pub mod four_half_approx;

/// Approximation algorithms for treewidth based on vertex separators.
#[derive(EnumIter, Serialize, Debug, Clone, Copy)]
pub enum ApproxAlgorithm {
    /// A 4-approximation based on two-third vertex separators.
    FourApprox,

    /// A 4.5-approximation based on one-half vertex separators.
    FourHalfApprox,
}

impl ApproxAlgorithm {
    /// Returns the worst-case treewidth bound for this approximation when the optimal treewidth is
    /// known.
    pub fn worst_case_from_optimal(&self, optimal: usize) -> usize {
        match self {
            ApproxAlgorithm::FourApprox => 4 * optimal,
            ApproxAlgorithm::FourHalfApprox => 4 * optimal + optimal / 2 + 1,
        }
    }
}

// Represents a separator-finding function. Since both algorithms use the same recursive structure,
// we could make a generic recursive function to which we pass the separator-finding function as an
// argument. This allows us to avoid code duplication between the two approximation algorithms,
// which holy differ in a few lines of code.
type SeparatorFn =
    fn(&adjlist::Graph, &HashSet<usize>, &HashSet<usize>, usize) -> Option<Separator>;

// Bitset version of the separator-finding function type.
type SeparatorBitSetFn = fn(&bitset::Graph, &BitSet, &BitSet, usize) -> Option<SeparatorBitSet>;

/// A separator and the two vertex sets it separates in an adjacency-list graph.
#[derive(Debug)]
pub struct Separator {
    /// The separator vertices.
    pub sep: HashSet<usize>,

    /// The first separated component.
    pub c1: HashSet<usize>,

    /// The second separated component.
    pub c2: HashSet<usize>,
}

/// A separator and the two vertex sets it separates in a bitset-base graph representation.
#[derive(Debug, Clone)]
pub struct SeparatorBitSet {
    /// The separator vertices.
    pub sep: BitSet,

    /// The first separated component.
    pub c1: BitSet,

    /// The second separated component.
    pub c2: BitSet,
}

// Generic implementation of the separator-based approximation algorithms. The `separator_finder` and
// `separator_finder_bitset` arguments are used to specify the separator-finding function to use for
// adjacency-list and bitset-based graphs, respectively.
pub(crate) fn approx_treewidth_generic(
    graph: &graph::Graph,
    separator_finder: SeparatorFn,
    separator_finder_bitset: SeparatorBitSetFn,
) -> usize {
    match graph {
        graph::Graph::AdjList(g) => {
            let subset: HashSet<usize> = (0..g.n()).collect();

            for k in 1..=g.n() {
                let mut triangulated = g.clone();
                let mut max_bag = 0;

                if treewidth_recursive(
                    &mut triangulated,
                    &subset,
                    &HashSet::new(),
                    k,
                    &mut max_bag,
                    separator_finder,
                ) {
                    return max_bag - 1;
                }
            }

            unreachable!(
                "The treewidth of a graph with n vertices is at most n-1, so this loop should have found a solution (k = n should alaways succeed)"
            );
        }
        graph::Graph::BitSet(g) => {
            let mut subset = BitSet::new(g.n());
            for i in 0..g.n() {
                subset.insert(i);
            }

            let mut max_bag = 0;

            for k in 1..=g.n() {
                let mut triangulated = g.clone();

                if treewidth_recursive_bitset(
                    &mut triangulated,
                    &subset,
                    &BitSet::new(g.n()),
                    k,
                    &mut max_bag,
                    separator_finder_bitset,
                ) {
                    return max_bag - 1;
                }
            }

            unreachable!(
                "The treewidth of a graph with n vertices is at most n-1, so this loop should have found a solution (k = n should alaways succeed)"
            );
        }
    }
}

// Generic recursive function for the separator-based approximation algorithms. The `separator_finder`
// argument is used to specify the separator-finding function to use for adjacency-list graphs.
pub(crate) fn treewidth_recursive(
    graph: &mut adjlist::Graph,
    subset: &HashSet<usize>,
    w: &HashSet<usize>,
    k: usize,
    max_bag: &mut usize,
    separator_finder: SeparatorFn,
) -> bool {
    if subset.len() <= 4 * k {
        for (u, v) in subset.iter().tuple_combinations() {
            graph.add_edge(*u, *v);
        }

        *max_bag = (*max_bag).max(subset.len());
        return true;
    }

    let mut w_bis = w.clone();
    for u in subset.difference(w) {
        if w_bis.len() >= 3 * k + 2 {
            break;
        }

        w_bis.insert(*u);
    }

    let Some(separator) = separator_finder(graph, subset, &w_bis, k) else {
        return false;
    };

    let w1: HashSet<_> = separator
        .c1
        .intersection(w)
        .copied()
        .collect::<HashSet<_>>()
        .union(&separator.sep)
        .copied()
        .collect();

    let subset1: HashSet<usize> = separator.c1.union(&separator.sep).copied().collect();
    if !treewidth_recursive(graph, &subset1, &w1, k, max_bag, separator_finder) {
        return false;
    }

    let w2: HashSet<_> = separator
        .c2
        .intersection(w)
        .copied()
        .collect::<HashSet<_>>()
        .union(&separator.sep)
        .copied()
        .collect();

    let subset2: HashSet<usize> = separator.c2.union(&separator.sep).copied().collect();
    if !treewidth_recursive(graph, &subset2, &w2, k, max_bag, separator_finder) {
        return false;
    }

    let clique = w.union(&separator.sep).copied().collect::<Vec<_>>();

    for (u, v) in clique.iter().tuple_combinations() {
        graph.add_edge(*u, *v);
    }

    *max_bag = (*max_bag).max(clique.len());
    true
}

// Generic recursive function for the separator-based approximation algorithms. The `separator_finder_bitset`
// argument is used to specify the separator-finding function to use for bitset-based graphs.
pub(crate) fn treewidth_recursive_bitset(
    graph: &mut bitset::Graph,
    subset: &BitSet,
    w: &BitSet,
    k: usize,
    max_bag: &mut usize,
    separator_finder: SeparatorBitSetFn,
) -> bool {
    if subset.len() <= 4 * k {
        let vertices: Vec<_> = subset.iter().collect();

        for (&u, &v) in vertices.iter().tuple_combinations() {
            graph.add_edge(u, v);
        }

        *max_bag = (*max_bag).max(subset.len());
        return true;
    }

    let mut w_bis = w.clone();

    for u in subset.iter() {
        if w_bis.len() >= 3 * k + 2 {
            break;
        }

        if !w.contains(u) {
            w_bis.insert(u);
        }
    }

    let Some(separator) = separator_finder(graph, subset, &w_bis, k) else {
        return false;
    };

    let mut w1 = separator.sep.clone();
    for v in separator.c1.iter() {
        if w.contains(v) {
            w1.insert(v);
        }
    }

    let mut subset1 = separator.sep.clone();
    for v in separator.c1.iter() {
        subset1.insert(v);
    }

    if !treewidth_recursive_bitset(graph, &subset1, &w1, k, max_bag, separator_finder) {
        return false;
    }

    let mut w2 = separator.sep.clone();
    for v in separator.c2.iter() {
        if w.contains(v) {
            w2.insert(v);
        }
    }

    let mut subset2 = separator.sep.clone();
    for v in separator.c2.iter() {
        subset2.insert(v);
    }

    if !treewidth_recursive_bitset(graph, &subset2, &w2, k, max_bag, separator_finder) {
        return false;
    }

    let mut clique = w.clone();
    for v in separator.sep.iter() {
        clique.insert(v);
    }

    let clique_vertices: Vec<_> = clique.iter().collect();

    for (&u, &v) in clique_vertices.iter().tuple_combinations() {
        graph.add_edge(u, v);
    }

    *max_bag = (*max_bag).max(clique.len());
    true
}

// Helper functions for the separator-finding functions. These are used to build the flow network
// for the minimum vertex separator functions.
pub(crate) fn build_base_flow_network(
    graph: &adjlist::Graph,
    subset: &HashSet<usize>,
    subset_vec: &[usize],
    node_map: &HashMap<usize, usize>,
    inf: usize,
) -> (Vec<(usize, usize)>, Vec<usize>) {
    let estimated_edges = subset.len() + 2 * graph.m();
    let mut base_edges = Vec::with_capacity(estimated_edges);
    let mut base_cap = Vec::with_capacity(estimated_edges);

    for &orig_v in subset_vec {
        let v = node_map[&orig_v];
        add_arc(&mut base_edges, &mut base_cap, vin(v), vout(v), 1);

        let Some(neighbors) = graph.neighbors_ref(orig_v) else {
            continue;
        };

        for &orig_u in neighbors {
            if subset.contains(&orig_u) {
                let u = node_map[&orig_u];
                add_arc(&mut base_edges, &mut base_cap, vout(v), vin(u), inf);
            }
        }
    }

    (base_edges, base_cap)
}

// Helper functions for the separator-finding functions. These are used to build the flow network
// for the minimum vertex separator functions for bitset-based graphs.
pub(crate) fn build_base_flow_network_bitset(
    graph: &bitset::Graph,
    subset: &BitSet,
    subset_vec: &[usize],
    node_map: &HashMap<usize, usize>,
    inf: usize,
) -> (Vec<(usize, usize)>, Vec<usize>) {
    let estimated_edges = subset.len() + 2 * graph.m();
    let mut base_edges = Vec::with_capacity(estimated_edges);
    let mut base_cap = Vec::with_capacity(estimated_edges);

    for &orig_v in subset_vec {
        let v = node_map[&orig_v];
        add_arc(&mut base_edges, &mut base_cap, vin(v), vout(v), 1);

        let Some(neighbors) = graph.neighbors_ref(orig_v) else {
            continue;
        };

        for orig_u in neighbors.iter() {
            if subset.contains(orig_u) {
                let u = node_map[&orig_u];
                add_arc(&mut base_edges, &mut base_cap, vout(v), vin(u), inf);
            }
        }
    }

    (base_edges, base_cap)
}

// Computes a minimum vertex separator for the given graph and subset of vertices. The `src_out` and
// `sink_in` arguments specify the source and sink vertices in the flow network, which are used to
// compute the minimum vertex separator using a max flow algorithm. The `k` argument is used to set
// the flow limit for the max flow algorithm, which allows us to stop the algorithm early if
// the flow exceeds the limit, which means that there is no separator of size at most `k`.
pub(crate) fn minimum_vertex_separator(
    n: usize,
    edges: &[(usize, usize)],
    capacities: &[usize],
    src_out: usize,
    sink_in: usize,
    k: usize,
    subset: &[usize],
    node_map: &HashMap<usize, usize>,
) -> Option<Separator> {
    let Some(reachable) = max_flow_reachable(n, edges, capacities, src_out, sink_in, k + 1) else {
        return None;
    };

    let mut separator = HashSet::new();
    let mut c1 = HashSet::new();
    let mut c2 = HashSet::new();

    for &v in subset {
        let mapped = node_map[&v];
        let v_in = vin(mapped);
        let v_out = vout(mapped);

        if reachable[v_in] && !reachable[v_out] {
            separator.insert(v);
        } else if reachable[v_in] {
            c1.insert(v);
        } else {
            c2.insert(v);
        }
    }

    Some(Separator {
        sep: separator,
        c1,
        c2,
    })
}

// Computes a minimum vertex separator for the given bitset-based graph and subset of vertices. The
// `src_out` and `sink_in` arguments specify the source and sink vertices in the flow network,
// which are used to compute the minimum vertex separator using a max flow algorithm. The `k`
// argument is used to set the flow limit for the max flow algorithm, which allows us to stop the
// algorithm early if the flow exceeds the limit, which means that there is no separator of size at
// most `k`.
pub(crate) fn minimum_vertex_separator_bitset(
    n: usize,
    edges: &[(usize, usize)],
    capacities: &[usize],
    src_out: usize,
    sink_in: usize,
    k: usize,
    subset: &[usize],
    node_map: &HashMap<usize, usize>,
    graph_n: usize,
) -> Option<SeparatorBitSet> {
    let Some(reachable) = max_flow_reachable(n, edges, capacities, src_out, sink_in, k + 1) else {
        return None;
    };

    let mut separator = BitSet::new(graph_n);
    let mut c1 = BitSet::new(graph_n);
    let mut c2 = BitSet::new(graph_n);

    for &v in subset {
        let mapped = node_map[&v];
        let v_in = vin(mapped);
        let v_out = vout(mapped);

        if reachable[v_in] && !reachable[v_out] {
            separator.insert(v);
        } else if reachable[v_in] {
            c1.insert(v);
        } else {
            c2.insert(v);
        }
    }

    Some(SeparatorBitSet {
        sep: separator,
        c1,
        c2,
    })
}

// Helper function to add an arc to the flow network.
pub(crate) fn add_arc(
    edges: &mut Vec<(usize, usize)>,
    capacities: &mut Vec<usize>,
    from: usize,
    to: usize,
    cap: usize,
) {
    edges.push((from, to));
    capacities.push(cap);
}

// Helper functions to compute the in vertex indices in the flow network from the original graph vertex indices.
pub(crate) fn vin(i: usize) -> usize {
    2 * i
}


// Helper functions to compute the out vertex indices in the flow network from the original graph vertex indices.
pub(crate) fn vout(i: usize) -> usize {
    2 * i + 1
}
