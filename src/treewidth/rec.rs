use std::collections::HashSet;

use itertools::Itertools;

use crate::{graph::Graph, treewidth::{compute_q, compute_q_bitset}};

pub fn treewidth(graph: &Graph) -> usize {
    if graph.n() > 64 {
        let subset = (0..graph.n()).collect();
        treewdith_recursive(graph, &HashSet::new(), subset)

    } else {
        treewdith_recursive_bitset(graph, 0, (1u64 << graph.n()) - 1)
    }
}

fn treewdith_recursive(graph: &Graph, left: &HashSet<usize>, vertices: HashSet<usize>) -> usize {
    if vertices.len() == 1 {
        let v = *vertices.iter().next().unwrap();
        return compute_q(v, graph, &left);
    }

    let mut opt = usize::MAX;

    let vec: Vec<usize> = vertices.iter().cloned().collect();
    let set_size = vec.len() / 2;

    for subset_vec in vec.into_iter().combinations(set_size) {
        let subset: HashSet<usize> = subset_vec.into_iter().collect();
        let complement: HashSet<usize> = vertices
            .difference(&subset)
            .cloned()
            .collect();

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

fn treewdith_recursive_bitset(graph: &Graph, left: u64, vertices: u64) -> usize {

    // If there is only one vertex in vertices.
    // This is true if vertices is a power of two (i.e. one bit to 1).
    if vertices != 0 && (vertices & (vertices - 1)) == 0 {
        let v = vertices.trailing_zeros() as usize;

        return compute_q_bitset(v, graph, left);
    }

    let mut opt = usize::MAX;

    for subset in combinations(vertices) {
        let complement = vertices & !subset;

        let new_left = left | subset;

        let tw1 = treewdith_recursive_bitset(graph, left, subset);
        let tw2 = treewdith_recursive_bitset(graph, new_left, complement);

        let curr_opt = std::cmp::max(tw1, tw2);
        if curr_opt < opt {
            opt = curr_opt;
        }
    }

    opt
}

pub fn combinations(mut subset: u64) -> Vec<u64> {
    let mut positions = Vec::new();

    while subset != 0 {
        let u = subset.trailing_zeros() as usize;
        positions.push(u);
        subset &= subset - 1;
    }

    let k = positions.len() / 2;
    let mut result = Vec::new();
    let mut combination = (1u64 << k) - 1;
    let limit = 1u64 << positions.len();

    while combination < limit {
        let mut subset = 0u64;
        let mut tmp = combination;

        while tmp != 0 {
            let bit_pos = tmp.trailing_zeros() as usize;
            subset |= 1u64 << positions[bit_pos];
            tmp &= tmp - 1;
        }

        result.push(subset);

        let x = combination & (!combination + 1);
        let y = combination + x;
        combination = (((combination & !y) / x) >> 1) | y;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_treewidth() {
        let g = Graph::new_cycle(3);
        assert_eq!(treewidth(&g), 2);

        let g = Graph::new_path(4);
        assert_eq!(treewidth(&g), 1);

        let g = Graph::new_cycle(5);
        assert_eq!(treewidth(&g), 2);

        let g = Graph::new_complete(4);
        assert_eq!(treewidth(&g), 3);
    }
}
