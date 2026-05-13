use std::collections::HashSet;

use serde::Serialize;
use strum::EnumIter;
use ::fixedbitset::FixedBitSet;
use itertools::Itertools;

use crate::{graph::{adjlist::Graph, bitset, fixedbitset, newbitset}, utils::{bitset::BitSet, newbitset::NewBitSet}};

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

fn compute_q_bitset(v: usize, graph: &bitset::Graph, subset: BitSet) -> usize {
    assert!(graph.has_vertex(v));
    let component = connected_component_bitset(v, graph, subset);

    let mut num_vertices = 0;
    for w in 0..graph.n() {
        if !subset.contains(w)
            && w != v
            && !component.contains(w)
            && is_reachable_in_subset_bitset(w, graph, component)
        {
            num_vertices += 1;
        }
    }

    num_vertices
}

fn compute_q_fixedbitset(v: usize, graph: &fixedbitset::Graph, subset: &FixedBitSet) -> usize {
    assert!(graph.has_vertex(v));

    let component = connected_component_fixedbitset(v, graph, subset);

    let mut num_vertices = 0;

    for w in 0..graph.n() {
        if !subset.contains(w)
            && w != v
            && !component.contains(w)
            && is_reachable_in_subset_fixedbitset(w, graph, &component)
        {
            num_vertices += 1;
        }
    }

    num_vertices
}

fn compute_q_newbitset(v: usize, graph: &newbitset::Graph, subset: &NewBitSet) -> usize {
    assert!(graph.has_vertex(v));

    let component = connected_component_newbitset(v, graph, subset);

    let mut num_vertices = 0;

    for w in 0..graph.n() {
        if !subset.contains(w)
            && w != v
            && !component.contains(w)
            && is_reachable_in_subset_newbitset(w, graph, &component)
        {
            num_vertices += 1;
        }
    }

    num_vertices
}

fn is_reachable_in_subset_bitset(vertex: usize, graph: &bitset::Graph, component: BitSet) -> bool {
    assert!(graph.has_vertex(vertex));

    for neighbor in graph.neighbors(vertex).unwrap() {
        if component.contains(neighbor) {
            return true;
        }
    }
    false
}

fn is_reachable_in_subset_fixedbitset(
    vertex: usize,
    graph: &fixedbitset::Graph,
    component: &FixedBitSet,
) -> bool {
    assert!(graph.has_vertex(vertex));

    let Some(mut neighbors) = graph.neighbors_iter(vertex) else {
        return false;
    };

    neighbors.any(|neighbor| component.contains(neighbor))
}

fn is_reachable_in_subset_newbitset(
    vertex: usize,
    graph: &newbitset::Graph,
    component: &NewBitSet,
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

fn connected_component_bitset(vertex: usize, graph: &bitset::Graph, subset: BitSet) -> BitSet {
    assert!(graph.has_vertex(vertex));

    let mut stack = vec![vertex];
    let mut visited = BitSet::new();
    visited.insert(vertex); 

    while let Some(v) = stack.pop() {
        for neighbor in graph.neighbors(v).unwrap() {
            if subset.contains(neighbor) && !visited.contains(neighbor) {
                visited.insert(neighbor);
                stack.push(neighbor);
            }
        }
    }

    visited
}

fn connected_component_fixedbitset(
    vertex: usize,
    graph: &fixedbitset::Graph,
    subset: &FixedBitSet,
) -> FixedBitSet {
    assert!(graph.has_vertex(vertex));

    let mut stack = vec![vertex];
    let mut visited = FixedBitSet::with_capacity(graph.n());
    visited.insert(vertex);

    while let Some(v) = stack.pop() {
        let Some(neighbors) = graph.neighbors_iter(v) else {
            continue;
        };

        for neighbor in neighbors {
            if subset.contains(neighbor) && !visited.contains(neighbor) {
                visited.insert(neighbor);
                stack.push(neighbor);
            }
        }
    }

    visited
}

fn connected_component_newbitset(
    vertex: usize,
    graph: &newbitset::Graph,
    subset: &NewBitSet,
) -> NewBitSet {
    assert!(graph.has_vertex(vertex));

    let mut visited = NewBitSet::new(graph.n());
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

fn all_connected_component_bitset(graph: &bitset::Graph, subset: BitSet) -> Vec<BitSet> {
    let mut visited = BitSet::new();
    let mut components = Vec::new();

    for v in subset {
        if !visited.contains(v) {
            let component = connected_component_bitset(v, graph, subset);
            for u in component {
                visited.insert(u);
            }
            components.push(component);
        }
    }

    components
}

fn all_connected_component_fixedbitset(
    graph: &fixedbitset::Graph,
    subset: &FixedBitSet,
) -> Vec<FixedBitSet> {
    let mut visited = FixedBitSet::with_capacity(graph.n());
    let mut components = Vec::new();

    for v in subset.ones() {
        if !visited.contains(v) {
            let component = connected_component_fixedbitset(v, graph, subset);

            for u in component.ones() {
                visited.insert(u);
            }

            components.push(component);
        }
    }

    components
}

fn all_connected_component_newbitset(
    graph: &newbitset::Graph,
    subset: &NewBitSet,
) -> Vec<NewBitSet> {
    let mut visited = NewBitSet::new(graph.n());
    let mut components = Vec::new();

    for v in subset.iter() {
        if !visited.contains(v) {
            let component = connected_component_newbitset(v, graph, subset);

            for u in component.iter() {
                visited.insert(u);
            }

            components.push(component);
        }
    }

    components
}

fn combinations_bitset(subset: BitSet, k: usize) -> Vec<BitSet> {
    let positions = subset.to_vec();

    let mut result = Vec::new();
    let mut combination = (1 << k) - 1;
    let limit = 1 << positions.len();

    while combination < limit {
        let mut subset = BitSet::new();
        let tmp = BitSet::from_bits(combination);

        // while tmp != 0 {
        //     let bit_pos = tmp.trailing_zeros() as usize;
        //     subset |= 1u64 << positions[bit_pos];
        //     tmp &= tmp - 1;
        // }

        for bit in tmp {
            subset.insert(positions[bit]);
        }

        result.push(subset);

        let x = combination & (!combination + 1);
        let y = combination + x;
        combination = (((combination & !y) / x) >> 1) | y;
    }

    result
}

fn for_each_combination_fixedbitset<F>(
    subset: &FixedBitSet,
    k: usize,
    mut f: F,
)
where
    F: FnMut(FixedBitSet),
{
    let positions: Vec<usize> = subset.ones().collect();

    for combo in positions.iter().copied().combinations(k) {
        let mut bs = FixedBitSet::with_capacity(subset.len());
        for v in combo {
            bs.insert(v);
        }
        f(bs);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{graph::{adjlist, bitset}, utils::bitset::BitSet};
    use super::{connected_component, connected_component_bitset, is_reachable_in_subset, is_reachable_in_subset_bitset, compute_q, compute_q_bitset};


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
    fn test_connected_component_bitset() {
        let mut g = bitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);
        let mut vertices = BitSet::from_bits(0b11011); // Vertices 0,1,3,4

        let cc_0 = connected_component_bitset(0, &g, vertices);
        assert_eq!(cc_0, BitSet::from_bits(0b00011)); // Vertices 0 and 1

        vertices |= BitSet::from_bits(0b100); // Add vertex 2

        let cc_0 = connected_component_bitset(0, &g, vertices);
        assert_eq!(cc_0, BitSet::from_bits(0b00111)); // Vertices 0,1,2

        let cc_3 = connected_component_bitset(3, &g, vertices);
        assert_eq!(cc_3, BitSet::from_bits(0b11000)); // Vertices 3 and 4
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
    fn test_is_reachable_in_subset_bitset() {
        let mut g = bitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert!(is_reachable_in_subset_bitset(2, &g, BitSet::from_bits(0b00011)));
        assert!(!is_reachable_in_subset_bitset(2, &g, BitSet::from_bits(0b11000)));
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
    fn test_compute_q_bitset() {
        let mut g = bitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        // Vertices 3 and 4 are not connected to the component {0,1,2}
        let subset = BitSet::from_bits(0b00011); // Vertices 0 and 1
        let q_value = compute_q_bitset(2, &g, subset);
        assert_eq!(q_value, 0);

        // Vertex 4 is not connected to the component {0,1,2}
        let subset = BitSet::from_bits(0b01011); // Vertices 0,1,3
        let q_value = compute_q_bitset(2, &g, subset);
        assert_eq!(q_value, 0);

        // Vertex 0 is connected to the component {1,2}, but 3 and 4 are not.
        let subset = BitSet::from_bits(0b00010); // Vertex 1
        let q_value = compute_q_bitset(2, &g, subset);
        assert_eq!(q_value, 1);
    }

    // TODO: tests for all_connected_component and all_connected_component_bitset and
    // combinations_bitset
}

#[cfg(test)]
mod fixedbitset_tests {
    use ::fixedbitset::FixedBitSet;

    use crate::graph::{fixedbitset};
    use super::{
        connected_component_fixedbitset,
        is_reachable_in_subset_fixedbitset,
        compute_q_fixedbitset,
        all_connected_component_fixedbitset,
    };

    fn bitset_from(n: usize, vertices: &[usize]) -> FixedBitSet {
        let mut bs = FixedBitSet::with_capacity(n);
        for &v in vertices {
            bs.insert(v);
        }
        bs
    }

    #[test]
    fn test_connected_component_fixedbitset() {
        let mut g = fixedbitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        let mut vertices = bitset_from(5, &[0, 1, 3, 4]);

        let cc_0 = connected_component_fixedbitset(0, &g, &vertices);
        assert_eq!(cc_0, bitset_from(5, &[0, 1]));

        vertices.insert(2);

        let cc_0 = connected_component_fixedbitset(0, &g, &vertices);
        assert_eq!(cc_0, bitset_from(5, &[0, 1, 2]));

        let cc_3 = connected_component_fixedbitset(3, &g, &vertices);
        assert_eq!(cc_3, bitset_from(5, &[3, 4]));
    }

    #[test]
    fn test_is_reachable_in_subset_fixedbitset() {
        let mut g = fixedbitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert!(is_reachable_in_subset_fixedbitset(
            2,
            &g,
            &bitset_from(5, &[0, 1])
        ));

        assert!(!is_reachable_in_subset_fixedbitset(
            2,
            &g,
            &bitset_from(5, &[3, 4])
        ));
    }

    #[test]
    fn test_compute_q_fixedbitset() {
        let mut g = fixedbitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        let subset = bitset_from(5, &[0, 1]);
        assert_eq!(compute_q_fixedbitset(2, &g, &subset), 0);

        let subset = bitset_from(5, &[0, 1, 3]);
        assert_eq!(compute_q_fixedbitset(2, &g, &subset), 0);

        let subset = bitset_from(5, &[1]);
        assert_eq!(compute_q_fixedbitset(2, &g, &subset), 1);
    }

    #[test]
    fn test_all_connected_component_fixedbitset() {
        let mut g = fixedbitset::Graph::new(6);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        let subset = bitset_from(6, &[0, 1, 2, 3, 4, 5]);

        let mut components = all_connected_component_fixedbitset(&g, &subset);

        components.sort_by_key(|c| c.ones().next().unwrap_or(usize::MAX));

        assert_eq!(components.len(), 3);
        assert_eq!(components[0], bitset_from(6, &[0, 1, 2]));
        assert_eq!(components[1], bitset_from(6, &[3, 4]));
        assert_eq!(components[2], bitset_from(6, &[5]));
    }
}

#[cfg(test)]
mod newbitset_tests {
    use crate::{
        graph::newbitset,
        utils::newbitset::NewBitSet,
    };

    use super::{
        compute_q_newbitset,
        connected_component_newbitset,
        is_reachable_in_subset_newbitset,
        all_connected_component_newbitset,
    };

    fn bitset_from(n: usize, vertices: &[usize]) -> NewBitSet {
        let mut bs = NewBitSet::new(n);
        for &v in vertices {
            bs.insert(v);
        }
        bs
    }

    #[test]
    fn test_connected_component_newbitset() {
        let mut g = newbitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        let mut vertices = bitset_from(5, &[0, 1, 3, 4]);

        assert_eq!(
            connected_component_newbitset(0, &g, &vertices),
            bitset_from(5, &[0, 1])
        );

        vertices.insert(2);

        assert_eq!(
            connected_component_newbitset(0, &g, &vertices),
            bitset_from(5, &[0, 1, 2])
        );

        assert_eq!(
            connected_component_newbitset(3, &g, &vertices),
            bitset_from(5, &[3, 4])
        );
    }

    #[test]
    fn test_is_reachable_in_subset_newbitset() {
        let mut g = newbitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert!(is_reachable_in_subset_newbitset(
            2,
            &g,
            &bitset_from(5, &[0, 1])
        ));

        assert!(!is_reachable_in_subset_newbitset(
            2,
            &g,
            &bitset_from(5, &[3, 4])
        ));
    }

    #[test]
    fn test_compute_q_newbitset() {
        let mut g = newbitset::Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert_eq!(
            compute_q_newbitset(2, &g, &bitset_from(5, &[0, 1])),
            0
        );

        assert_eq!(
            compute_q_newbitset(2, &g, &bitset_from(5, &[0, 1, 3])),
            0
        );

        assert_eq!(
            compute_q_newbitset(2, &g, &bitset_from(5, &[1])),
            1
        );
    }

    #[test]
    fn test_all_connected_component_newbitset() {
        let mut g = newbitset::Graph::new(6);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        let subset = bitset_from(6, &[0, 1, 2, 3, 4, 5]);

        let mut components = all_connected_component_newbitset(&g, &subset);

        components.sort_by_key(|c| c.first_bit().unwrap_or(usize::MAX));

        assert_eq!(components.len(), 3);
        assert_eq!(components[0], bitset_from(6, &[0, 1, 2]));
        assert_eq!(components[1], bitset_from(6, &[3, 4]));
        assert_eq!(components[2], bitset_from(6, &[5]));
    }
}
