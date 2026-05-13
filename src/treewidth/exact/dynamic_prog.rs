use std::collections::{HashMap, HashSet};

use crate::{graph::{Graph, adjlist, bitset, fixedbitset, newbitset}, treewidth::exact::{compute_q, compute_q_bitset, compute_q_fixedbitset, compute_q_newbitset}, utils::{bitset::BitSet, newbitset::NewBitSet}};
use ::fixedbitset::FixedBitSet;

pub fn treewidth(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => treewidth_vec(g),
        Graph::BitSet(g) => treewidth_bitset(g),
        Graph::FixedBitSet(g) => treewidth_fixedbitset(g),
        Graph::NewBitSet(g) => treewidth_newbitset(g),
    }
}


fn treewidth_vec(graph: &adjlist::Graph) -> usize {
    let mut pred: HashMap<Vec<bool>, usize> = HashMap::new();
    pred.insert(vec![false; graph.n()], 0);

    let mut current: HashMap<Vec<bool>, usize> = HashMap::new();

    for _ in 0..graph.n() {
        for subset in pred.keys() {
            for candidate in 0..graph.n() {
                if subset[candidate] {
                    continue;
                }

                let mut new_subset = subset.clone();
                new_subset[candidate] = true;

                if current.contains_key(&new_subset) {
                    continue;
                }

                let mut min = usize::MAX;

                for v in 0..graph.n() {
                    if !new_subset[v] {
                        continue;
                    }

                    let mut subset: HashSet<usize> = new_subset
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &b)| if b { Some(i) } else { None })
                        .collect();
                    subset.remove(&v);

                    let q_value = compute_q(v, graph, &subset);

                    new_subset[v] = false;
                    let tw = pred.get(&new_subset).unwrap();
                    new_subset[v] = true;

                    let val = std::cmp::max(*tw, q_value);
                    if val < min {
                        min = val;
                    }
                }

                current.insert(new_subset, min);
            }
        }

        pred = current;
        current = HashMap::new();
    }

    *pred.values().next().unwrap()
}

fn treewidth_bitset(graph: &bitset::Graph) -> usize {
    let mut pred: HashMap<BitSet, usize> = HashMap::new();
    pred.insert(BitSet::new(), 0);

    let mut current: HashMap<BitSet, usize> = HashMap::new();

    for _ in 0..graph.n() {
        for subset in pred.keys() {
            for candidate in 0..graph.n() {
                if subset.contains(candidate) {
                    continue;
                }

                let mut new_subset = *subset;
                new_subset.insert(candidate);

                if current.contains_key(&new_subset) {
                    continue;
                }

                let mut min = usize::MAX;

                for v in new_subset {
                    let q_value = compute_q_bitset(v, graph, new_subset);

                    new_subset.remove(v);
                    let tw = pred.get(&new_subset).unwrap();
                    new_subset.insert(v);

                    let val = std::cmp::max(*tw, q_value);
                    if val < min {
                        min = val;
                    }
                }

                current.insert(new_subset, min);
            }
        }

        pred = current;
        current = HashMap::new();
    }

    *pred.values().next().unwrap()
}

fn treewidth_fixedbitset(graph: &fixedbitset::Graph) -> usize {
    let mut pred: HashMap<FixedBitSet, usize> = HashMap::new();

    let empty = FixedBitSet::with_capacity(graph.n());
    pred.insert(empty, 0);

    let mut current: HashMap<FixedBitSet, usize> = HashMap::new();

    for _ in 0..graph.n() {
        for subset in pred.keys() {
            for candidate in 0..graph.n() {
                if subset.contains(candidate) {
                    continue;
                }

                let mut new_subset = subset.clone();
                new_subset.insert(candidate);

                if current.contains_key(&new_subset) {
                    continue;
                }

                let mut min = usize::MAX;

                for v in new_subset.ones().collect::<Vec<_>>() {
                    let q_value = compute_q_fixedbitset(v, graph, &new_subset);

                    new_subset.set(v, false);
                    let tw = pred.get(&new_subset).unwrap();
                    new_subset.insert(v);

                    let val = std::cmp::max(*tw, q_value);
                    min = min.min(val);
                }

                current.insert(new_subset, min);
            }
        }

        pred = current;
        current = HashMap::new();
    }

    *pred.values().next().unwrap()
}

fn treewidth_newbitset(graph: &newbitset::Graph) -> usize {
    let mut pred: HashMap<NewBitSet, usize> = HashMap::new();
    pred.insert(NewBitSet::new(graph.n()), 0);

    let mut current: HashMap<NewBitSet, usize> = HashMap::new();

    for _ in 0..graph.n() {
        for subset in pred.keys() {
            for candidate in 0..graph.n() {
                if subset.contains(candidate) {
                    continue;
                }

                let mut new_subset = subset.clone();
                new_subset.insert(candidate);

                if current.contains_key(&new_subset) {
                    continue;
                }

                let mut min = usize::MAX;

                let vertices: Vec<usize> = new_subset.iter().collect();

                for v in vertices {
                    let q_value = compute_q_newbitset(v, graph, &new_subset);

                    new_subset.remove(v);
                    let tw = pred.get(&new_subset).unwrap();
                    new_subset.insert(v);

                    let val = std::cmp::max(*tw, q_value);
                    min = min.min(val);
                }

                current.insert(new_subset, min);
            }
        }

        pred = current;
        current = HashMap::new();
    }

    *pred.values().next().unwrap()
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

    #[test]
    fn test_treewidth_fixedbitset() {
        let g = Graph::FixedBitSet(fixedbitset::Graph::new_cycle(3));
        assert_eq!(treewidth(&g), 2);

        let g = Graph::FixedBitSet(fixedbitset::Graph::new_path(4));
        assert_eq!(treewidth(&g), 1);

        let g = Graph::FixedBitSet(fixedbitset::Graph::new_cycle(5));
        assert_eq!(treewidth(&g), 2);

        let g = Graph::FixedBitSet(fixedbitset::Graph::new_complete(4));
        assert_eq!(treewidth(&g), 3);
    }
}
