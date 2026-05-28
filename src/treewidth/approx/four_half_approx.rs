use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;

use crate::graph;
use crate::graph::adjlist::Graph;
use crate::utils::max_flow::max_flow_reachable;

#[derive(Debug)]
pub struct Separator {
    pub sep: HashSet<usize>,
    pub c1: HashSet<usize>,
    pub c2: HashSet<usize>,
}

pub fn approx_treewidth(graph: &graph::Graph) -> usize {

    match graph {
        graph::Graph::AdjList(g) => {
            let subset: HashSet<usize> = (0..g.n()).collect();

            for k in 1..=g.n() {
                let mut triangulated = g.clone();
                let mut max_bag = 0;

                if treewidth_recursive(&mut triangulated, &subset, &HashSet::new(), k, &mut max_bag) {
                    return max_bag - 1;
                }
            }

            unreachable!("The treewidth of a graph with n vertices is at most n-1, so this loop should have found a solution (k = n should alaways succeed)");
        },
        graph::Graph::BitSet(g) => {
            let mut subset = BitSet::new(g.n());
            for i in 0..g.n() {
                subset.insert(i);
            }

            let mut max_bag = 0;

            for k in 1..=g.n() {
                let mut triangulated = g.clone();

                if treewidth_recursive_bitset(&mut triangulated, &subset, &BitSet::new(g.n()), k, &mut max_bag) {
                    return max_bag - 1;
                }
            }

            unreachable!("The treewidth of a graph with n vertices is at most n-1, so this loop should have found a solution (k = n should alaways succeed)");
        },
    }
}

pub fn treewidth_recursive(graph: &mut Graph, subset: &HashSet<usize>, w: &HashSet<usize>, k: usize, max_bag: &mut usize) -> bool {
    if subset.len() <= 4 * k {
        for (u, v) in subset.iter().tuple_combinations() {
            graph.add_edge(*u, *v);
        }

        *max_bag = (*max_bag).max(subset.len());
        return true;
    }

    let mut w_bis = w.clone();
    for u in subset.difference(&w) {
        if w_bis.len() >= 3 * k + 2 {
            break;
        }

        w_bis.insert(*u);
    }

    let Some(separator) = one_half_vertex_separator(graph, subset, &w_bis, k) else {
        return false;
    };

    let w1: HashSet<_> = separator.c1.intersection(w)
        .cloned()
        .collect::<HashSet<_>>()
        .union(&separator.sep)
        .cloned()
        .collect();

    let subset1: HashSet<usize> = separator.c1.union(&separator.sep).cloned().collect();
    if !treewidth_recursive(graph, &subset1, &w1, k, max_bag) {
        return false;
    }

    let w2: HashSet<_> = separator.c2.intersection(w)
        .cloned()
        .collect::<HashSet<_>>()
        .union(&separator.sep)
        .cloned()
        .collect();

    let subset2: HashSet<usize> = separator.c2.union(&separator.sep).cloned().collect();
    if !treewidth_recursive(graph, &subset2, &w2, k, max_bag) {
        return false;
    }

    let clique = w.union(&separator.sep).cloned().collect::<Vec<_>>();

    for (u, v) in clique.iter().tuple_combinations() {
        graph.add_edge(*u, *v);
    }

    *max_bag = (*max_bag).max(clique.len());
    true
}

pub fn one_half_vertex_separator(
    graph: &Graph,
    subset: &HashSet<usize>,
    w: &HashSet<usize>,
    k: usize,
) -> Option<Separator> {
    let size_w1 = (w.len() + 1) / 2;
    let inf = subset.len() + 2;

    let w_vec: Vec<usize> = w.iter().copied().collect();
    let subset_vec: Vec<usize> = subset.iter().copied().collect();

    let mut node_map: HashMap<usize, usize> =
        subset.iter().enumerate().map(|(i, &u)| (u, i)).collect();

    let source = subset.len();
    let sink = subset.len() + 1;

    node_map.insert(source, source);
    node_map.insert(sink, sink);

    let estimated_edges = subset.len() + 2 * graph.m();

    let mut base_edges = Vec::with_capacity(estimated_edges);
    let mut base_cap = Vec::with_capacity(estimated_edges);

    for &orig_v in subset_vec.iter() {
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

    let src_out = vout(source);
    let sink_in = vin(sink);

    for w1_refs in w_vec.iter().combinations(size_w1) {
        let w1: Vec<usize> = w1_refs.into_iter().copied().collect();
        let w1_set: HashSet<usize> = w1.iter().copied().collect();

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

        let w2: Vec<usize> =
            w_vec.iter().copied().filter(|v| !w1_set.contains(v)).collect();

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
            && separator.sep.len() <= 3 * k / 2
            && !separator.c1.is_empty()
            && !separator.c2.is_empty()
        {
            return Some(separator);
        }
    }

    None
}


fn minimum_vertex_separator(n: usize, edges: &[(usize, usize)], capacities: &[usize], src_out: usize, sink_in: usize, k: usize, subset: &Vec<usize>, node_map: &HashMap<usize, usize>) -> Option<Separator> {

    let Some(reachable) = max_flow_reachable(n, edges, capacities, src_out, sink_in, k + 1) else {
        return None;
    };

    let mut separator = HashSet::new();
    let mut c1 = HashSet::new();
    let mut c2 = HashSet::new();

    for &v in subset.iter() {

        let v_mapped = node_map[&v];
        let v_in = vin(v_mapped);
        let v_out = vout(v_mapped);

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

fn add_arc(edges: &mut Vec<(usize, usize)>, capacities: &mut Vec<usize>, from: usize, to: usize, cap: usize) {
    edges.push((from, to));
    capacities.push(cap);
}

fn vin(i: usize) -> usize { 2 * i }
fn vout(i: usize) -> usize { 2 * i + 1 }

// TODO: Need to check the implementation
use crate::{
    graph::bitset,
    utils::bitset::BitSet,
};

#[derive(Debug, Clone)]
pub struct SeparatorBitSet {
    pub sep: BitSet,
    pub c1: BitSet,
    pub c2: BitSet,
}

pub fn treewidth_recursive_bitset(
    graph: &mut bitset::Graph,
    subset: &BitSet,
    w: &BitSet,
    k: usize,
    max_bag: &mut usize,
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

    let Some(separator) = one_half_vertex_separator_bitset(graph, subset, &w_bis, k) else {
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

    if !treewidth_recursive_bitset(graph, &subset1, &w1, k, max_bag) {
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

    if !treewidth_recursive_bitset(graph, &subset2, &w2, k, max_bag) {
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

pub fn one_half_vertex_separator_bitset(
    graph: &bitset::Graph,
    subset: &BitSet,
    w: &BitSet,
    k: usize,
) -> Option<SeparatorBitSet> {
    let size_w1 = (w.len() + 1) / 2;
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

    let mut base_edges = Vec::with_capacity(subset.len() + 2 * graph.m());
    let mut base_cap = Vec::with_capacity(subset.len() + 2 * graph.m());

    for &orig_v in &subset_vec {
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

    let src_out = vout(source);
    let sink_in = vin(sink);

    for w1_refs in w_vec.iter().combinations(size_w1) {
        let w1: Vec<usize> = w1_refs.into_iter().copied().collect();

        let mut w1_set = BitSet::new(graph.n());
        for &v in &w1 {
            w1_set.insert(v);
        }

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

        let w2: Vec<usize> = w_vec
            .iter()
            .copied()
            .filter(|v| !w1_set.contains(*v))
            .collect();

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
            if separator.sep.len() <= k
                && !separator.c1.is_empty()
                && !separator.c2.is_empty()
            {
                return Some(separator);
            }
        }
    }

    None
}

pub fn minimum_vertex_separator_bitset(
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
