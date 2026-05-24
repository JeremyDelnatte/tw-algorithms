use std::collections::HashSet;

use serde::Serialize;
use strum::EnumIter;
use itertools::Itertools;

use crate::{graph::{adjlist::Graph, bitset}, utils::{bitset::BitSet}};

pub mod dynamic_prog;
pub mod rec;
pub mod improved_rec;
pub mod branch_bound;

#[derive(EnumIter, Serialize, Debug, Clone)]
pub enum ExactAlgorithm {
    DynamicProg,
    Recursive,
    ImprovedRec,
    BranchBound,
}

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

fn is_reachable_in_subset(vertex: usize, graph: &Graph, component: &HashSet<usize>) -> bool {
    assert!(graph.has_vertex(vertex));

    for neighbor in graph.neighbors_ref(vertex).unwrap() {
        if component.contains(&neighbor) {
            return true;
        }
    }
    false
}

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

fn is_reachable_in_subset_bitset(
    vertex: usize,
    graph: &bitset::Graph,
    component: &BitSet,
) -> bool {
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

fn connected_component_bitset(
    vertex: usize,
    graph: &bitset::Graph,
    subset: &BitSet,
) -> BitSet {
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

fn all_connected_component_bitset(
    graph: &bitset::Graph,
    subset: &BitSet,
) -> Vec<BitSet> {
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

fn combinations_bitset(
    subset: &BitSet,
    n: usize,
    k: usize,
) -> impl Iterator<Item = BitSet> {
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

    use crate::{graph::{adjlist}};
    use super::{connected_component, is_reachable_in_subset, compute_q};

    use crate::{
        graph::bitset,
        utils::bitset::BitSet,
    };

    use super::{
        compute_q_bitset,
        connected_component_bitset,
        is_reachable_in_subset_bitset,
        all_connected_component_bitset,
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

        assert_eq!(
            compute_q_bitset(2, &g, &bitset_from(5, &[0, 1])),
            0
        );

        assert_eq!(
            compute_q_bitset(2, &g, &bitset_from(5, &[0, 1, 3])),
            0
        );

        assert_eq!(
            compute_q_bitset(2, &g, &bitset_from(5, &[1])),
            1
        );
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
