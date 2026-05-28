use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;

use crate::graph;
use crate::graph::adjlist::Graph;
use crate::graph::bitset;
use crate::utils::bitset::BitSet;

use super::{
    Separator, SeparatorBitSet, add_arc, approx_treewidth_generic, build_base_flow_network,
    build_base_flow_network_bitset, minimum_vertex_separator, minimum_vertex_separator_bitset,
    vin, vout,
};

pub fn approx_treewidth(graph: &graph::Graph) -> usize {
    approx_treewidth_generic(
        graph,
        two_third_vertex_separator,
        two_third_vertex_separator_bitset,
    )
}

pub fn two_third_vertex_separator(
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

        let remaining: Vec<usize> =
            w_vec.iter().copied().filter(|v| !w1_set.contains(v)).collect();

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

// #[cfg(test)]
// mod tests {
//     use std::collections::HashSet;
//
//     use crate::graph::{self, Graph};
//     use crate::treewidth::approx::four_approx::{minimum_vertex_separator, two_third_vertex_separator};
//     use std::fs::File;
//     use std::io::{BufRead, BufReader};
//
//     fn validate_two_way_separator(
//         graph: &Graph,
//         subset: &HashSet<usize>,
//         w: &HashSet<usize>,
//         sep: &HashSet<usize>,
//         c1: &HashSet<usize>,
//         c2: &HashSet<usize>,
//     ) -> bool {
//         // disjointness
//         if !sep.is_disjoint(c1) || !sep.is_disjoint(c2) || !c1.is_disjoint(c2) {
//             return false;
//         }
//
//         // cover
//         let union: HashSet<_> = sep.union(c1).copied().collect::<HashSet<_>>()
//             .union(c2).copied().collect();
//         if &union != subset {
//             return false;
//         }
//
//         // no path from c1 to c2 in subset \ sep
//         let allowed: HashSet<_> = subset.difference(sep).copied().collect();
//         let mut seen = HashSet::new();
//         let mut stack: Vec<usize> = c1.iter().copied().collect();
//         seen.extend(c1.iter().copied());
//
//         while let Some(u) = stack.pop() {
//             if c2.contains(&u) {
//                 return false;
//             }
//             if let Some(neigh) = graph.neighbors_ref(u) {
// _test_v4                for &v in neigh {
//                     if allowed.contains(&v) && !seen.contains(&v) {
//                         seen.insert(v);
//                         stack.push(v);
//                     }
//                 }
//             }
//         }
//
//         let bound = (2 * w.len() + 2) / 3; // ceil(2|w|/3)
//         c1.intersection(w).count() <= bound && c2.intersection(w).count() <= bound
//     }
//
//     #[test]
//     fn test_two_third_separator_from_g6_file() {
//         let path = "tests/graphs.g6";
//         let file = File::open(path).expect("failed to open g6 file");
//         let reader = BufReader::new(file);
//
//         for (lineno, line) in reader.lines().enumerate() {
//             let line = line.expect("failed to read line");
//             let line = line.trim();
//
//             if line.is_empty() {
//                 continue;
//             }
//
//             let g = graph::Graph::from_g6(line);
//             let subset: HashSet<_> = (0..g.n()).collect();
//             let w = subset.clone();
//
//             // This test is only meaningful when k=1 could plausibly work.
//             // Skip tiny graphs where the separator procedure's assumptions may not fit.
//             if g.n() < 2 {
//                 continue;
//             }
//
//             let sep = two_third_vertex_separator(&g, &subset, &w, 1);
//
//             match sep {
//                 Some(sep) => {
//                     assert!(
//                         sep.sep.len() <= 1,
//                         "line {}: separator too large for graph {}",
//                         lineno + 1,
//                         line
//                     );
//                     assert!(
//                         validate_two_way_separator(&g, &subset, &w, &sep.sep, &sep.c1, &sep.c2),
//                         "line {}: invalid separator for graph {}",
//                         lineno + 1,
//                         line
//                     );
//                 }
//                 None => {
//                     panic!(
//                         "line {}: expected a 2/3 separator with k=1, but got None for graph {}",
//                         lineno + 1,
//                         line
//                     );
//                 }
//             }
//         }
//     }
//
//     // #[test]
//     // fn test_minimum_vertex_separator() {
//     //     let g6 = "E?^o";
//     //     let graph = Graph::from_g6(g6);
//     //
//     //     let separator = minimum_vertex_separator(&graph, 0, 3);
//     //     assert_eq!(separator.sep.len(), 1);
//     //     assert_eq!(separator.sep.contains(&1) || separator.sep.contains(&2), true);
//     // }
// }

pub fn two_third_vertex_separator_bitset(
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
                if separator.sep.len() <= k
                    && !separator.c1.is_empty()
                    && !separator.c2.is_empty()
                {
                    return Some(separator);
                }
            }
        }
    }

    None
}
