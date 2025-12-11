use std::collections::{HashMap, HashSet};

use crate::{graph::Graph, treewidth::{compute_q, compute_q_bitset}};

pub fn treewidth(graph: &Graph) -> usize {
    if graph.n() > 64 {
        treewidth_vec(graph)
    } else {
        treewidth_bitset(graph)
    }
}


fn treewidth_vec(graph: &Graph) -> usize {
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

fn treewidth_bitset(graph: &Graph) -> usize {
    let mut pred: HashMap<u64, usize> = HashMap::new();
    pred.insert(0u64, 0);

    let mut current: HashMap<u64, usize> = HashMap::new();

    for _ in 0..graph.n() {
        for subset in pred.keys() {
            for candidate in 0..graph.n() {
                if subset & (1 << candidate) != 0 {
                    continue;
                }

                let mut new_subset = *subset;
                new_subset |= 1 << candidate;

                if current.contains_key(&new_subset) {
                    continue;
                }

                let mut min = usize::MAX;

                let mut tmp = new_subset;

                while tmp != 0 {
                    let v = tmp.trailing_zeros() as usize;
                    tmp &= tmp - 1;

                    let q_value = compute_q_bitset(v, graph, new_subset);

                    new_subset &= !(1 << v);
                    let tw = pred.get(&new_subset).unwrap();
                    new_subset |= 1 << v;

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
