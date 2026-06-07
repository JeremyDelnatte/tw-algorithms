//! A 4-approximation algorithm for treewidth. It uses two-third vertex separators inside the
//! generic approximation function.

use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;

use crate::graph;
use crate::graph::adjlist::Graph;
use crate::graph::bitset;
use crate::utils::bitset::BitSet;

use super::{
    Separator, SeparatorBitSet, add_arc, approx_treewidth_generic, build_base_flow_network,
    build_base_flow_network_bitset, minimum_vertex_separator, minimum_vertex_separator_bitset, vin,
    vout,
};

/// Approximates the treewidth using the 4-approximation algorithm.
pub fn approx_treewidth(graph: &graph::Graph) -> usize {
    approx_treewidth_generic(
        graph,
        two_third_vertex_separator,
        two_third_vertex_separator_bitset,
    )
}

// Finds a two-third vertex separator for an adjacency-list graph.
fn two_third_vertex_separator(
    graph: &Graph,
    subset: &HashSet<usize>,
    w: &HashSet<usize>,
    k: usize,
) -> Option<Separator> {
    let size_w1 = (w.len() + 1) / 2;
    let size_w2 = (w.len() + 2) / 3;
    let inf = subset.len() + 2;

    let w_vec: Vec<usize> = w.iter().copied().collect();
    let subset_vec: Vec<usize> = subset.iter().copied().collect();

    let mut node_map: HashMap<usize, usize> =
        subset.iter().enumerate().map(|(i, &u)| (u, i)).collect();

    let source = subset.len();
    let sink = subset.len() + 1;

    node_map.insert(source, source);
    node_map.insert(sink, sink);

    let (base_edges, base_cap) =
        build_base_flow_network(graph, subset, &subset_vec, &node_map, inf);

    let src_out = vout(source);
    let sink_in = vin(sink);

    for w1_refs in w_vec.iter().combinations(size_w1) {
        let w1: Vec<usize> = w1_refs.into_iter().copied().collect();
        let w1_set: HashSet<usize> = w1.iter().copied().collect();

        let remaining: Vec<usize> = w_vec
            .iter()
            .copied()
            .filter(|v| !w1_set.contains(v))
            .collect();

        let mut w1_edges = base_edges.clone();
        let mut w1_cap = base_cap.clone();

        // Add W1 clique.
        for (&u, &v) in w1.iter().tuple_combinations() {
            let u = node_map[&u];
            let v = node_map[&v];
            add_arc(&mut w1_edges, &mut w1_cap, vout(u), vin(v), inf);
            add_arc(&mut w1_edges, &mut w1_cap, vout(v), vin(u), inf);
        }

        // Connect source to W1.
        for &u in &w1 {
            let u = node_map[&u];
            add_arc(&mut w1_edges, &mut w1_cap, src_out, vin(u), inf);
        }

        for w2_refs in remaining.iter().combinations(size_w2) {
            let w2: Vec<usize> = w2_refs.into_iter().copied().collect();

            let mut w2_edges = w1_edges.clone();
            let mut w2_cap = w1_cap.clone();

            // Add W2 clique.
            for (&u, &v) in w2.iter().tuple_combinations() {
                let u = node_map[&u];
                let v = node_map[&v];
                add_arc(&mut w2_edges, &mut w2_cap, vout(u), vin(v), inf);
                add_arc(&mut w2_edges, &mut w2_cap, vout(v), vin(u), inf);
            }

            // Connect W2 to sink.
            for &u in &w2 {
                let u = node_map[&u];
                add_arc(&mut w2_edges, &mut w2_cap, vout(u), sink_in, inf);
            }

            let separator = minimum_vertex_separator(
                subset.len() * 2 + 4,
                &w2_edges,
                &w2_cap,
                src_out,
                sink_in,
                k,
                &subset_vec,
                &node_map,
            );

            if let Some(separator) = separator
                && separator.sep.len() <= k
                && !separator.c1.is_empty()
                && !separator.c2.is_empty()
            {
                return Some(separator);
            }
        }
    }

    None
}

// Finds a two-third vertex separator for a bitset-based graph.
fn two_third_vertex_separator_bitset(
    graph: &bitset::Graph,
    subset: &BitSet,
    w: &BitSet,
    k: usize,
) -> Option<SeparatorBitSet> {
    let size_w1 = (w.len() + 1) / 2;
    let size_w2 = (w.len() + 2) / 3;
    let inf = subset.len() + 2;

    let w_vec: Vec<usize> = w.iter().collect();
    let subset_vec: Vec<usize> = subset.iter().collect();

    let mut node_map: HashMap<usize, usize> = HashMap::with_capacity(subset.len() + 2);

    for (i, u) in subset_vec.iter().copied().enumerate() {
        node_map.insert(u, i);
    }

    let source = subset.len();
    let sink = subset.len() + 1;

    let flow_n = 2 * (subset.len() + 2);

    let (base_edges, base_cap) =
        build_base_flow_network_bitset(graph, subset, &subset_vec, &node_map, inf);

    let src_out = vout(source);
    let sink_in = vin(sink);

    for w1_refs in w_vec.iter().combinations(size_w1) {
        let w1: Vec<usize> = w1_refs.into_iter().copied().collect();

        let mut w1_set = BitSet::new(graph.n());
        for &v in &w1 {
            w1_set.insert(v);
        }

        let remaining: Vec<usize> = w_vec
            .iter()
            .copied()
            .filter(|v| !w1_set.contains(*v))
            .collect();

        let mut w1_edges = base_edges.clone();
        let mut w1_cap = base_cap.clone();

        for (&u_orig, &v_orig) in w1.iter().tuple_combinations() {
            let u = node_map[&u_orig];
            let v = node_map[&v_orig];

            add_arc(&mut w1_edges, &mut w1_cap, vout(u), vin(v), inf);
            add_arc(&mut w1_edges, &mut w1_cap, vout(v), vin(u), inf);
        }

        for &u_orig in &w1 {
            let u = node_map[&u_orig];
            add_arc(&mut w1_edges, &mut w1_cap, src_out, vin(u), inf);
        }

        for w2_refs in remaining.iter().combinations(size_w2) {
            let w2: Vec<usize> = w2_refs.into_iter().copied().collect();

            let mut w2_edges = w1_edges.clone();
            let mut w2_cap = w1_cap.clone();

            for (&u_orig, &v_orig) in w2.iter().tuple_combinations() {
                let u = node_map[&u_orig];
                let v = node_map[&v_orig];

                add_arc(&mut w2_edges, &mut w2_cap, vout(u), vin(v), inf);
                add_arc(&mut w2_edges, &mut w2_cap, vout(v), vin(u), inf);
            }

            for &u_orig in &w2 {
                let u = node_map[&u_orig];
                add_arc(&mut w2_edges, &mut w2_cap, vout(u), sink_in, inf);
            }

            let separator = minimum_vertex_separator_bitset(
                flow_n,
                &w2_edges,
                &w2_cap,
                src_out,
                sink_in,
                k,
                &subset_vec,
                &node_map,
                graph.n(),
            );

            if let Some(separator) = separator {
                if separator.sep.len() <= k && !separator.c1.is_empty() && !separator.c2.is_empty()
                {
                    return Some(separator);
                }
            }
        }
    }

    None
}
