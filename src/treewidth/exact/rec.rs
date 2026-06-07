//! A recursive exact treewidth algorithm.

use std::collections::HashSet;

use itertools::Itertools;

use crate::{
    graph::{Graph, adjlist, bitset},
    treewidth::exact::{combinations_bitset, compute_q, compute_q_bitset},
    utils::bitset::BitSet,
};

/// Computes the exact treewidth using the recursive algorithm.
pub fn treewidth(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => {
            let vertices: HashSet<usize> = (0..g.n()).collect();
            treewdith_recursive(g, &HashSet::new(), vertices)
        }
        Graph::BitSet(g) => {
            let vertices = BitSet::full(g.n());
            treewdith_recursive_bitset(g, &BitSet::new(g.n()), vertices)
        }
    }
}

// Computes the exact treewidth using the recursive algorithm for adjacency-list graphs.
pub(super) fn treewdith_recursive(
    graph: &adjlist::Graph,
    left: &HashSet<usize>,
    vertices: HashSet<usize>,
) -> usize {
    if vertices.len() == 1 {
        let v = *vertices.iter().next().unwrap();
        return compute_q(v, graph, &left);
    }

    let mut opt = usize::MAX;

    let vec: Vec<usize> = vertices.iter().cloned().collect();
    let set_size = vec.len() / 2;

    for subset_vec in vec.into_iter().combinations(set_size) {
        let subset: HashSet<usize> = subset_vec.into_iter().collect();
        let complement: HashSet<usize> = vertices.difference(&subset).cloned().collect();

        let mut new_left = left.clone();
        for &v in &subset {
            new_left.insert(v);
        }

        let tw1 = treewdith_recursive(graph, &left, subset);
        let tw2 = treewdith_recursive(graph, &new_left, complement);

        let curr_opt = std::cmp::max(tw1, tw2);
        if curr_opt < opt {
            opt = curr_opt;
        }
    }

    opt
}

// Computes the exact treewidth using the recursive algorithm for bitset-based graphs.
pub(super) fn treewdith_recursive_bitset(
    graph: &bitset::Graph,
    left: &BitSet,
    vertices: BitSet,
) -> usize {
    if vertices.has_one_bit() {
        let v = vertices.first_bit().unwrap();
        return compute_q_bitset(v, graph, &left);
    }

    let mut opt = usize::MAX;

    for subset in combinations_bitset(&vertices, graph.n(), vertices.len() / 2) {
        let complement = vertices.difference(&subset);
        let new_left = left | &subset;

        let tw1 = treewdith_recursive_bitset(graph, left, subset);
        let tw2 = treewdith_recursive_bitset(graph, &new_left, complement);

        let curr_opt = std::cmp::max(tw1, tw2);
        if curr_opt < opt {
            opt = curr_opt;
        }
    }

    opt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treewidth_vec() {
        let g = Graph::AdjList(adjlist::Graph::new_cycle(3));
        assert_eq!(treewidth(&g), 2);

        let g = Graph::AdjList(adjlist::Graph::new_path(4));
        assert_eq!(treewidth(&g), 1);

        let g = Graph::AdjList(adjlist::Graph::new_cycle(5));
        assert_eq!(treewidth(&g), 2);

        let g = Graph::AdjList(adjlist::Graph::new_complete(4));
        assert_eq!(treewidth(&g), 3);
    }

    #[test]
    fn test_treewidth_bitset() {
        let g = Graph::BitSet(bitset::Graph::new_cycle(3));
        assert_eq!(treewidth(&g), 2);

        let g = Graph::BitSet(bitset::Graph::new_path(4));
        assert_eq!(treewidth(&g), 1);

        let g = Graph::BitSet(bitset::Graph::new_cycle(5));
        assert_eq!(treewidth(&g), 2);

        let g = Graph::BitSet(bitset::Graph::new_complete(4));
        assert_eq!(treewidth(&g), 3);
    }
}
