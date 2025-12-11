use std::collections::HashSet;

use crate::graph::Graph;

pub mod dynamic_prog;
pub mod rec;

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

fn compute_q_bitset(v: usize, graph: &Graph, subset: u64) -> usize {
    assert!(graph.has_vertex(v));
    let component = connected_component_bitset(v, graph, subset);

    let mut num_vertices = 0;
    for w in 0..graph.n() {
        if subset & (1 << w) == 0
            && w != v
            && component & (1 << w) == 0
            && is_reachable_in_subset_bitset(w, graph, component)
        {
            num_vertices += 1;
        }
    }

    num_vertices
}

fn is_reachable_in_subset_bitset(vertex: usize, graph: &Graph, component: u64) -> bool {
    assert!(graph.has_vertex(vertex));

    for neighbor in graph.neighbors_ref(vertex).unwrap() {
        if component & (1 << neighbor) != 0 {
            return true;
        }
    }
    false
}

fn connected_component_bitset(vertex: usize, graph: &Graph, subset: u64) -> u64 {
    assert!(graph.has_vertex(vertex));

    let mut visited = 0u64;
    let mut stack = vec![vertex];
    visited |= 1 << vertex;

    while let Some(v) = stack.pop() {
        for neighbor in graph.neighbors_ref(v).unwrap() {
            if subset & (1 << neighbor) != 0 && visited & (1 << neighbor) == 0 {
                visited |= 1 << neighbor;
                stack.push(*neighbor);
            }
        }
    }

    visited
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_connected_component() {
        let mut g = Graph::new(5);
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
        let mut g = Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);
        let mut vertices = 0b11011; // Vertices 0,1,3,4

        let cc_0 = connected_component_bitset(0, &g, vertices);
        assert_eq!(cc_0, 0b00011); // Vertices 0 and 1

        vertices |= 0b100; // Add vertex 2

        let cc_0 = connected_component_bitset(0, &g, vertices);
        assert_eq!(cc_0, 0b00111); // Vertices 0,1,2

        let cc_3 = connected_component_bitset(3, &g, vertices);
        assert_eq!(cc_3, 0b11000); // Vertices 3 and 4
    }

    #[test]
    fn test_is_reachable_in_subset() {
        let mut g = Graph::new(5);
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
        let mut g = Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        assert!(is_reachable_in_subset_bitset(2, &g, 0b00011));
        assert!(!is_reachable_in_subset_bitset(2, &g, 0b11000));
    }

    #[test]
    fn test_compute_q() {
        let mut g = Graph::new(5);
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
        let mut g = Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(3, 4);

        // Vertices 3 and 4 are not connected to the component {0,1,2}
        let subset = 0b00011; // Vertices 0 and 1
        let q_value = compute_q_bitset(2, &g, subset);
        assert_eq!(q_value, 0);

        // Vertex 4 is not connected to the component {0,1,2}
        let subset = 0b01011; // Vertices 0,1,3
        let q_value = compute_q_bitset(2, &g, subset);
        assert_eq!(q_value, 0);

        // Vertex 0 is connected to the component {1,2}, but 3 and 4 are not.
        let subset = 0b00010; // Vertex 1
        let q_value = compute_q_bitset(2, &g, subset);
        assert_eq!(q_value, 1);
    }
}
