//! Exact algorithms for computing treewidth.

use std::collections::HashSet;

use itertools::Itertools;
use serde::Serialize;
use strum::EnumIter;

use crate::{
    graph::{adjlist::Graph, bitset},
    utils::bitset::BitSet,
};

pub mod branch_bound;
pub mod dynamic_prog;
pub mod improved_rec;
pub mod rec;

/// Exact treewidth algorithms.
#[derive(EnumIter, Serialize, Debug, Clone)]
pub enum ExactAlgorithm {
    /// Dynamic programming algorithm over linear orderings.
    DynamicProg,

    /// Recursive algorithm over linear orderings.
    Recursive,

    /// Recursive algorithm with separator-based improvements.
    ImprovedRec,

    /// Branch-and-bound search over linear orderings.
    BranchBound,
}

// Helper function to compute the q function for a vertex `v` in an adjacency-list graph, given a
// subset of vertexes. The q function counts the number of vertices that are not in the subset
// `subset` that are reachable from `v` using only vertices in `subset`.
//
// To achieve this, the function first computes the connected component of `v` in the subgraph
// induced by `subset`. Then, it iterates over all vertices that are not in `subset` and counts how
// many of them are connected to the component.
fn compute_q(v: usize, graph: &Graph, subset: &HashSet<usize>) -> usize {
    assert!(graph.has_vertex(v));
    let component = connected_component(v, graph, &subset);

    let mut num_vertices = 0;
    for w in 0..graph.n() {
        if !subset.contains(&w)
            && w != v
            && !component.contains(&w)
            && is_reachable_in_subset(w, graph, &component)
        {
            num_vertices += 1;
        }
    }

    num_vertices
}

// Helper function to check if a vertex `vertex` is reachable from any vertex in the `component`
// using only vertices in the `component`.
fn is_reachable_in_subset(vertex: usize, graph: &Graph, component: &HashSet<usize>) -> bool {
    assert!(graph.has_vertex(vertex));

    for neighbor in graph.neighbors_ref(vertex).unwrap() {
        if component.contains(&neighbor) {
            return true;
        }
    }
    false
}

// Helper function to compute the connected component of a vertex `vertex` in the subgraph induced by
// `subset`.
fn connected_component(vertex: usize, graph: &Graph, subset: &HashSet<usize>) -> HashSet<usize> {
    assert!(graph.has_vertex(vertex));

    let mut visited = HashSet::new();
    let mut stack = vec![vertex];
    visited.insert(vertex);

    while let Some(v) = stack.pop() {
        for neighbor in graph.neighbors_ref(v).unwrap() {
            if subset.contains(&neighbor) && !visited.contains(&neighbor) {
                visited.insert(*neighbor);
                stack.push(*neighbor);
            }
        }
    }

    visited
}

// Helper function to compute all connected components in the subgraph induced by `subset`.
fn all_connected_component(graph: &Graph, subset: &HashSet<usize>) -> Vec<HashSet<usize>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();

    for &v in subset {
        if !visited.contains(&v) {
            let component = connected_component(v, graph, subset);
            for &u in &component {
                visited.insert(u);
            }
            components.push(component);
        }
    }

    components
}

// BitSet versions of the `compute_q` function.
fn compute_q_bitset(v: usize, graph: &bitset::Graph, subset: &BitSet) -> usize {
    assert!(graph.has_vertex(v));

    let component = connected_component_bitset(v, graph, subset);

    let mut num_vertices = 0;

    for w in 0..graph.n() {
        if !subset.contains(w)
            && w != v
            && !component.contains(w)
            && is_reachable_in_subset_bitset(w, graph, &component)
        {
            num_vertices += 1;
        }
    }

    num_vertices
}

// BitSet version of the `is_reachable_in_subset` function.
fn is_reachable_in_subset_bitset(vertex: usize, graph: &bitset::Graph, component: &BitSet) -> bool {
    assert!(graph.has_vertex(vertex));

    let Some(neighbors) = graph.neighbors_ref(vertex) else {
        return false;
    };

    for neighbor in neighbors.iter() {
        if component.contains(neighbor) {
            return true;
        }
    }

    false
}

// BitSet version of the `connected_component` function.
fn connected_component_bitset(vertex: usize, graph: &bitset::Graph, subset: &BitSet) -> BitSet {
    assert!(graph.has_vertex(vertex));

    let mut visited = BitSet::new(graph.n());
    let mut stack = vec![vertex];

    visited.insert(vertex);

    while let Some(v) = stack.pop() {
        let Some(neighbors) = graph.neighbors_ref(v) else {
            continue;
        };

        for neighbor in neighbors.iter() {
            if subset.contains(neighbor) && !visited.contains(neighbor) {
                visited.insert(neighbor);
                stack.push(neighbor);
            }
        }
    }

    visited
}

// BitSet version of the `all_connected_component` function.
fn all_connected_component_bitset(graph: &bitset::Graph, subset: &BitSet) -> Vec<BitSet> {
    let mut visited = BitSet::new(graph.n());
    let mut components = Vec::new();

    for v in subset.iter() {
        if !visited.contains(v) {
            let component = connected_component_bitset(v, graph, subset);

            for u in component.iter() {
                visited.insert(u);
            }

            components.push(component);
        }
    }

    components
}

// Helper function to compute all combinations of size `k` of vertices in a subset represented as a
// BitSet. The function returns an iterator over the combinations in order to avoid generating all
// combinations at once, which can be very large for large subsets and values of `k`.
fn combinations_bitset(subset: &BitSet, n: usize, k: usize) -> impl Iterator<Item = BitSet> {
    subset
        .to_vec()
        .into_iter()
        .combinations(k)
        .map(move |combo| {
            let mut bs = BitSet::new(n);

            for v in combo {
                bs.insert(v);
            }

            bs
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{compute_q, connected_component, is_reachable_in_subset};
    use crate::graph::adjlist;

    use crate::{graph::bitset, utils::bitset::BitSet};

    use super::{
        all_connected_component_bitset, compute_q_bitset, connected_component_bitset,
        is_reachable_in_subset_bitset,
    };

    fn bitset_from(n: usize, vertices: &[usize]) -> BitSet {
        let mut bs = BitSet::new(n);
        for &v in vertices {
            bs.insert(v);
        }
        bs
    }

    #[test]
    fn test_connected_component() {
        let mut g = adjlist::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);
        let mut vertices: HashSet<usize> = vec![0, 1, 3, 4].into_iter().collect();

        let cc_0 = connected_component(0, &g, &vertices);
        assert_eq!(cc_0, vec![0, 1].into_iter().collect());

        vertices.insert(2);

        let cc_0 = connected_component(0, &g, &vertices);
        assert_eq!(cc_0, vec![0, 1, 2].into_iter().collect());

        let cc_3 = connected_component(3, &g, &vertices);
        assert_eq!(cc_3, vec![3, 4].into_iter().collect());
    }

    #[test]
    fn test_is_reachable_in_subset() {
        let mut g = adjlist::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert!(is_reachable_in_subset(
            2,
            &g,
            &vec![0, 1].into_iter().collect()
        ));
        assert!(!is_reachable_in_subset(
            2,
            &g,
            &vec![3, 4].into_iter().collect()
        ));
    }

    #[test]
    fn test_compute_q() {
        let mut g = adjlist::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        // Vertices 3 and 4 are not connected to the component {0,1,2}
        let subset: HashSet<usize> = vec![0, 1].into_iter().collect();
        let q_value = compute_q(2, &g, &subset);
        assert_eq!(q_value, 0);

        // Vertex 4 is not connected to the component {0,1,2}
        let subset: HashSet<usize> = vec![0, 1, 3].into_iter().collect();
        let q_value = compute_q(2, &g, &subset);
        assert_eq!(q_value, 0);

        // Vertex 0 is connected to the component {1,2}, but 3 and 4 are not.
        let subset: HashSet<usize> = vec![1].into_iter().collect();
        let q_value = compute_q(2, &g, &subset);
        assert_eq!(q_value, 1);
    }

    #[test]
    fn test_connected_component_bitset() {
        let mut g = bitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        let mut vertices = bitset_from(5, &[0, 1, 3, 4]);

        assert_eq!(
            connected_component_bitset(0, &g, &vertices),
            bitset_from(5, &[0, 1])
        );

        vertices.insert(2);

        assert_eq!(
            connected_component_bitset(0, &g, &vertices),
            bitset_from(5, &[0, 1, 2])
        );

        assert_eq!(
            connected_component_bitset(3, &g, &vertices),
            bitset_from(5, &[3, 4])
        );
    }

    #[test]
    fn test_is_reachable_in_subset_bitset() {
        let mut g = bitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert!(is_reachable_in_subset_bitset(
            2,
            &g,
            &bitset_from(5, &[0, 1])
        ));

        assert!(!is_reachable_in_subset_bitset(
            2,
            &g,
            &bitset_from(5, &[3, 4])
        ));
    }

    #[test]
    fn test_compute_q_bitset() {
        let mut g = bitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert_eq!(compute_q_bitset(2, &g, &bitset_from(5, &[0, 1])), 0);

        assert_eq!(compute_q_bitset(2, &g, &bitset_from(5, &[0, 1, 3])), 0);

        assert_eq!(compute_q_bitset(2, &g, &bitset_from(5, &[1])), 1);
    }

    #[test]
    fn test_all_connected_component_bitset() {
        let mut g = bitset::Graph::new(6);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        let subset = bitset_from(6, &[0, 1, 2, 3, 4, 5]);

        let mut components = all_connected_component_bitset(&g, &subset);

        components.sort_by_key(|c| c.first_bit().unwrap_or(usize::MAX));

        assert_eq!(components.len(), 3);
        assert_eq!(components[0], bitset_from(6, &[0, 1, 2]));
        assert_eq!(components[1], bitset_from(6, &[3, 4]));
        assert_eq!(components[2], bitset_from(6, &[5]));
    }
}
