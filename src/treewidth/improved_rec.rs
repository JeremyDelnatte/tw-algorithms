use std::collections::HashSet;

use crate::{
    graph::{Graph, adjlist, bitset},
    treewidth::{all_connected_component, all_connected_component_bitset, combinations_bitset, rec}, utils::bitset::BitSet,
};

pub fn treewidth(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => {
            let mut k = g.n();
            while treewdith_recursive(g, k) {
                k -= 1;
            }
            k + 1
        }
        Graph::BitSet(g) => {
            let mut k = g.n();
            while treewdith_recursive_bitset(g, k) {
                k -= 1;
            }
            k + 1
        }
    }
}

fn treewdith_recursive(graph: &adjlist::Graph, k: usize) -> bool {
    if graph.n() <= k + 1 {
        return true;
    }

    if k <= (0.25 * graph.n() as f64) as usize || k >= (0.4203 * graph.n() as f64) as usize {
        let vertices: HashSet<usize> = (0..graph.n()).collect();

        for subset_vec in itertools::Itertools::combinations(0..graph.n(), k + 1) {
            let subset: HashSet<usize> = subset_vec.into_iter().collect();
            let complement: HashSet<usize> = vertices.difference(&subset).cloned().collect();

            let components = all_connected_component(graph, &complement);
            let max_size = max_size_all_components(&components);

            if max_size > (graph.n() - k) / 2 {
                continue;
            }

            let mut tbool = true;
            for component in components {
                tbool = tbool
                    && rec::treewdith_recursive(graph, &HashSet::new(), component.clone()) <= k;
            }

            if tbool {
                return true;
            }
        }
    } else {
        let vertices: HashSet<usize> = (0..graph.n()).collect();

        for subset_vec in itertools::Itertools::combinations(0..graph.n(), (0.4203 * (graph.n() as f64)) as usize + 1) {
            let subset: HashSet<usize> = subset_vec.into_iter().collect();
            let complement: HashSet<usize> = vertices.difference(&subset).cloned().collect();

            let components = all_connected_component(graph, &complement);
            let max_size = max_size_all_components(&components);

            if max_size > (graph.n() - subset.len() + 1) / 2 {
                continue;
            }

            let fill_in_graph = fill_in_graph(graph, &subset);

            let mut tbool = treewdith_recursive(&fill_in_graph, k);

            for component in components {
                tbool = tbool
                    && rec::treewdith_recursive(graph, &HashSet::new(), component.clone()) <= k;
            }

            if tbool {
                return true;
            }
        }
    }

    false
}

fn fill_in_graph(graph: &adjlist::Graph, subset: &HashSet<usize>) -> adjlist::Graph {
    let new_n = subset.len();
    let mut new_graph = adjlist::Graph::new(new_n);

    let complement: HashSet<usize> = (0..graph.n()).filter(|v| !subset.contains(v)).collect();

    let mut components = all_connected_component(graph, &complement);
    let subset_vec: Vec<usize> = subset.iter().cloned().collect();

    for v in 0..new_n {
        let v_orig = subset_vec[v];

        for component in &mut components {
            for neighbor in graph.neighbors_ref(v_orig).unwrap() {
                if component.contains(neighbor) {
                    component.insert(v_orig);
                }
            }
        }
    }

    for v in 0..(new_n - 1) {
        let v_orig = subset_vec[v];
        for w in (v + 1)..new_n {
            let w_orig = subset_vec[w];

            if graph.neighbors_ref(v_orig).unwrap().contains(&w_orig) {
                new_graph.add_edge(v, w);
                continue;
            }

            for component in &components {
                if component.contains(&v_orig) && component.contains(&w_orig) {
                    new_graph.add_edge(v, w);
                }
            }
        }
    }

    new_graph
}

fn max_size_all_components(components: &Vec<HashSet<usize>>) -> usize {
    components.iter().map(|c| c.len()).max().unwrap()
}

fn treewdith_recursive_bitset(graph: &bitset::Graph, k: usize) -> bool {
    if graph.n() <= k + 1 {
        return true;
    }
    let vertices = BitSet::from_bits((1 << graph.n()) - 1);

    if k <= (0.25 * graph.n() as f64) as usize || k >= (0.4203 * graph.n() as f64) as usize {
        for subset in combinations_bitset(vertices, k + 1) {
            let complement = vertices & !subset;

            let components = all_connected_component_bitset(graph, complement);
            let max_size = max_size_all_components_bitset(&components);

            if max_size > (graph.n() - k) / 2 {
                continue;
            }

            let mut tbool = true;
            for component in components {
                tbool = tbool
                    && rec::treewdith_recursive_bitset(graph, BitSet::new(), component) <= k;
            }

            if tbool {
                return true;
            }
        }
    } else {
        for subset in combinations_bitset(vertices, (0.4203 * (graph.n() as f64)) as usize + 1) {
            let complement = vertices & !subset;

            let components = all_connected_component_bitset(graph, complement);
            let max_size = max_size_all_components_bitset(&components);

            if max_size > (graph.n() - k) / 2 {
                continue;
            }

            let fill_in_graph = fill_in_graph_bitset(graph, subset);

            let mut tbool = treewdith_recursive_bitset(&fill_in_graph, k);

            for component in components {
                tbool = tbool
                    && rec::treewdith_recursive_bitset(graph, BitSet::new(), component) <= k;
            }

            if tbool {
                return true;
            }
        }
    }

    false
}

fn max_size_all_components_bitset(components: &Vec<BitSet>) -> usize {
    components.iter().map(|c| c.len()).max().unwrap()
}

fn fill_in_graph_bitset(graph: &bitset::Graph, subset: BitSet) -> bitset::Graph {
    let new_n = subset.len();
    let mut new_graph = bitset::Graph::new(new_n);

    let complement = BitSet::from_bits((1 << graph.n()) - 1) & !subset;

    let mut components = all_connected_component_bitset(graph, complement);
    let subset_vec: Vec<usize> = subset.to_vec();

    for v in 0..new_n {
        let v_orig = subset_vec[v];

        for component in &mut components {
            for neighbor in graph.neighbors(v_orig).unwrap() {
                if component.contains(neighbor) {
                    component.insert(v_orig);
                }
            }
        }
    }

    for v in 0..(new_n - 1) {
        let v_orig = subset_vec[v];
        for w in (v + 1)..new_n {
            let w_orig = subset_vec[w];

            if graph.neighbors(v_orig).unwrap().contains(w_orig) {
                new_graph.add_edge(v, w);
                continue;
            }

            for component in &components {
                if component.contains(v_orig) && component.contains(w_orig) {
                    new_graph.add_edge(v, w);
                }
            }
        }
    }

    new_graph
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

    // TODO:
    // #[test]
    // fn test_fill_in_graph() {
    //     let mut g = adjlist::Graph::new(5);
    //     g.add_edge(0, 1);
    //     g.add_edge(1, 2);
    //     g.add_edge(3, 4);
    //
    //     let subset: HashSet<usize> = vec![0, 1, 2].into_iter().collect();
    //     let filled_graph = fill_in_graph(&g, &subset);
    //
    //     let mut expected_graph = adjlist::Graph::new(3);
    //     expected_graph.add_edge(0, 1);
    //     expected_graph.add_edge(1, 2);
    //     expected_graph.add_edge(0, 2); // Filled edge
    //
    //     assert_eq!(filled_graph, expected_graph);
    // }
}
