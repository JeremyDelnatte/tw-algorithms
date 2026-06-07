//! An adjacency-list graph representation for undirected graphs. This representation is less
//! efficient as it uses vector-based adjacency lists that stores neighbors.

use std::
    collections::HashSet
;

use rand::{RngExt, rngs::StdRng};

use crate::utils::g6::{self, get_edges, get_size, to_edges, to_size};

/// An undirected graph represented by an adjacency list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    n: usize,
    m: usize,
    adjency: Vec<Vec<usize>>,
}

impl Graph {
    /// Creates a graph with `n` vertices and no edges.
    pub fn new(n: usize) -> Self {
        Graph {
            n,
            m: 0,
            adjency: vec![Vec::new(); n],
        }
    }

    /// Creates a cycle graph on `n` vertices.
    pub fn new_cycle(n: usize) -> Self {
        let mut graph = Graph::new(n);
        for i in 0..n {
            graph.add_edge(i, (i + 1) % n);
        }
        graph
    }

    /// Creates a complete graph on `n` vertices.
    pub fn new_complete(n: usize) -> Self {
        let mut graph = Graph::new(n);
        for i in 0..n {
            for j in (i + 1)..n {
                graph.add_edge(i, j);
            }
        }
        graph
    }

    /// Creates a path graph on `n` vertices.
    pub fn new_path(n: usize) -> Self {
        let mut graph = Graph::new(n);
        for i in 0..(n - 1) {
            graph.add_edge(i, i + 1);
        }
        graph
    }

    /// Returns the number of vertices.
    pub fn n(&self) -> usize {
        self.n
    }

    /// Returns the number of edges.
    pub fn m(&self) -> usize {
        self.m
    }

    /// Returns true when the graph has no vertices.
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Returns a copy of the neighbors of vertex `i`, or `None` if `i` is not a vertex.
    pub fn neighbors(&self, i: usize) -> Option<Vec<usize>> {
        self.adjency.get(i).cloned()
    }

    /// Returns a reference to the neighbors of vertex `i`, or `None` if `i` is not a vertex.
    pub fn neighbors_ref(&self, i: usize) -> Option<&Vec<usize>> {
        self.adjency.get(i)
    }

    fn add_arc(&mut self, i: usize, j: usize) -> bool {
        if j >= self.n {
            panic!("Index out of bounds");
        }

        if self
            .adjency
            .get(i)
            .expect("index out of bounds")
            .contains(&j)
        {
            return false;
        }

        self.adjency.get_mut(i).unwrap().push(j);
        true
    }

    /// Adds an undirected edge between vertices `i` and `j`.
    ///
    /// Returns true if the edge was inserted and false if it was already present.
    pub fn add_edge(&mut self, i: usize, j: usize) -> bool {
        let added = self.add_arc(i, j) && self.add_arc(j, i);
        if added {
            self.m += 1;
        }
        added
    }

    fn remove_arc(&mut self, i: usize, j: usize) -> bool {
        if !self
            .adjency
            .get_mut(i)
            .expect("index out of bounds")
            .contains(&j)
        {
            return false;
        }

        self.adjency.get_mut(i).unwrap().retain(|v| *v != j);
        true
    }

    /// Removes the undirected edge between vertices `i` and `j`.
    ///
    /// Returns true if the edge was present.
    pub fn remove_edge(&mut self, i: usize, j: usize) -> bool {
        let removed = self.remove_arc(i, j) && self.remove_arc(j, i);
        if removed {
            self.m -= 1;
        }
        removed
    }

    /// Returns true when vertices `i` and `j` are adjacent.
    pub fn has_edge(&self, i: usize, j: usize) -> bool {
        if i >= self.n || j >= self.n {
            return false;
        }

        self.adjency
            .get(i)
            .expect("index out of bounds")
            .contains(&j)
    }

    /// Returns true when `i` is a vertex of the graph.
    pub fn has_vertex(&self, i: usize) -> bool {
        i < self.n
    }

    /// Removes vertex `v` and shifts higher vertex indices down by one.
    ///
    /// Returns false if `v` is not a vertex.
    pub fn remove_vertex(&mut self, v: usize) -> bool {
        if v >= self.n {
            return false;
        }

        for u in 0..self.n {
            if self.remove_arc(u, v) {
                self.m -= 1;
            }
        }

        self.adjency.remove(v);
        self.n -= 1;

        for neighbors in self.adjency.iter_mut() {
            for neighbor in neighbors.iter_mut() {
                if *neighbor > v {
                    *neighbor -= 1;
                }
            }
        }

        true
    }

    /// Removes vertex `v` and returns its former neighbors with shifted indices.
    ///
    /// Returns `None` if `v` is not a vertex.
    pub fn remove_vertex_neighbors(&mut self, v: usize) -> Option<Vec<usize>> {
        if v >= self.n {
            return None;
        }

        for u in 0..self.n {
            if self.remove_arc(u, v) {
                self.m -= 1;
            }
        }

        let mut neighbors_vertex = self.adjency.remove(v);
        self.n -= 1;

        for neighbors in self.adjency.iter_mut() {
            for neighbor in neighbors.iter_mut() {
                if *neighbor > v {
                    *neighbor -= 1;
                }
            }
        }

        for neighbor in neighbors_vertex.iter_mut() {
            if *neighbor > v {
                *neighbor -= 1;
            }
        }

        Some(neighbors_vertex)
    }

    /// Eliminates vertex `v` by removing it and turning its neighborhood into a clique.
    pub fn elim_vertex(&mut self, v: usize) {
        let neighbors = self.remove_vertex_neighbors(v).unwrap();
        let num_neighbors = neighbors.len();

        if num_neighbors == 0 {
            return;
        }

        for i in 0..(num_neighbors - 1) {
            let vertex_i = neighbors[i];
            for j in (i + 1)..num_neighbors {
                self.add_edge(vertex_i, neighbors[j]);
            }
        }
    }

    /// Eliminates vertex `v` and returns the fill-in edges that were added.
    pub fn elim_vertex_edges(&mut self, v: usize) -> HashSet<(usize, usize)> {
        let neighbors = self.remove_vertex_neighbors(v).unwrap();
        let num_neighbors = neighbors.len();
        let mut added_edges = HashSet::new();

        if num_neighbors == 0 {
            return added_edges;
        }

        for i in 0..(num_neighbors - 1) {
            let vertex_i = neighbors[i];
            for j in (i + 1)..num_neighbors {
                if self.add_edge(vertex_i, neighbors[j]) {
                    added_edges.insert((vertex_i, neighbors[j]));
                }
            }
        }

        added_edges
    }

    /// Counts the fill-in edges needed to eliminate vertex `v`.
    pub fn fill_in_count_vertex(&self, v: usize) -> usize {
        let neighbors = self.neighbors_ref(v).unwrap();
        let num_neighbors = neighbors.len();

        if num_neighbors == 0 {
            return 0;
        }

        let mut edges_missing = 0;
        for i in 0..(num_neighbors - 1) {
            let vertex_i = neighbors[i];
            for j in (i + 1)..num_neighbors {
                if !self.has_edge(vertex_i, neighbors[j]) {
                    edges_missing += 1;
                }
            }
        }

        edges_missing
    }

    /// Returns a vertex with the smallest fill-in count, breaking ties by degree.
    pub fn min_fill_in_count_vertex(&self) -> usize {
        let mut min = self.fill_in_count_vertex(0);
        let mut vertex_min = 0;

        // NOTE: In the case of ties, we choose the vertex with the smallest degree, as this will
        // probably lead to a smaller min-fill.
        let mut min_degree = self.degree(0);

        for v in 1..self.n() {
            let fill = self.fill_in_count_vertex(v);
            if fill < min || (fill == min && self.degree(v) < min_degree) {
                min = fill;
                vertex_min = v;
                min_degree = self.degree(v);
            }
        }

        vertex_min
    }

    /// Contracts edge `(u, v)` by removing `v` and connecting its neighbors to `u`.
    pub fn contract_edge(&mut self, mut u: usize, v: usize) {
        let neighbors = self.remove_vertex_neighbors(v).unwrap();

        // After removing v, u's index may have decreased by 1.
        if u > v {
            u -= 1;
        }

        for neighbor in neighbors {
            if neighbor != u {
                self.add_edge(u, neighbor);
            }
        }
    }

    /// Returns the degree of vertex `v`.
    pub fn degree(&self, v: usize) -> usize {
        if v >= self.n {
            panic!("Index out of bounds");
        }

        self.adjency[v].len()
    }

    /// Returns a vertex with minimum degree.
    pub fn min_degree_vertex(&self) -> usize {
        if self.n == 0 {
            panic!("Graph has no vertices");
        }

        self.adjency
            .iter()
            .enumerate()
            .min_by_key(|(_, neighbors)| neighbors.len())
            .map(|(i, _)| i)
            .unwrap()
    }

    /// Returns a neighbor of `v` with minimum degree.
    pub fn min_degree_neighbor(&self, v: usize) -> usize {
        if v >= self.n {
            panic!("Index out of bounds");
        }

        let neighbors = self.neighbors_ref(v).unwrap();

        if neighbors.is_empty() {
            panic!("Vertex has no neighbors");
        }

        *neighbors.iter().min_by_key(|&&u| self.degree(u)).unwrap()
    }

    /// Parses an adjacency-list graph from graph6 format.
    pub fn from_g6(repr: &str) -> Result<Self, g6::Error> {
        let bytes = repr.as_bytes();
        let n = get_size(bytes)?;

        let start_index = if n <= 62 { 1 } else { 4 };
        let edges = get_edges(&bytes[start_index..], n)?;

        let mut graph = Graph::new(n);
        for (i, j) in edges {
            graph.add_edge(i, j);
        }
        Ok(graph)
    }

    /// Converts the graph to graph6 format.
    pub fn to_g6(&self) -> String {
        let mut buf = to_size(self.n);
        let mut edges = Vec::new();

        for i in 0..self.n {
            for &j in self.adjency[i].iter() {
                if i < j {
                    edges.push((i, j));
                }
            }
        }

        to_edges(&edges, self.n, &mut buf);
        String::from_utf8(buf).expect("Failed to convert g6 representation to string")
    }

    /// Generates a random graph with `n` vertices and `m` edges using the default random number
    /// generator.
    pub fn generate_random(n: usize, m: usize) -> Self {
        if m > n * (n - 1) / 2 {
            panic!("Too many edges for the number of vertices");
        }

        let mut graph = Graph::new(n);
        let mut edges_added = 0;

        while edges_added < m {
            let i = rand::random::<u64>() as usize % n;
            let j = rand::random::<u64>() as usize % n;

            if i != j && graph.add_edge(i, j) {
                edges_added += 1;
            }
        }

        graph
    }

    /// Generates a random graph with `n` vertices and `m` edges using the provided random number
    /// generator.
    pub fn generate_random_with_rng(n: usize, m: usize, rng: &mut StdRng) -> Self {
        if m > n * (n - 1) / 2 {
            panic!("Too many edges for the number of vertices");
        }

        let mut graph = Graph::new(n);
        let mut edges_added = 0;

        while edges_added < m {
            let i = rng.random_range(0..n);
            let j = rng.random_range(0..n);

            if i != j && graph.add_edge(i, j) {
                edges_added += 1;
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::adjlist::Graph;

    #[test]
    fn test_remove_vertex() {
        let mut g = Graph::new_complete(5);
        g.remove_vertex(2);

        assert_eq!(g, Graph::new_complete(4));
    }

    #[test]
    fn test_min_degree_vertex() {
        let mut g = Graph::new(4);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(1, 2);
        g.add_edge(2, 3);

        assert_eq!(g.min_degree_vertex(), 3);
    }

    #[test]
    fn test_min_degree_neighbor() {
        let mut g = Graph::new(4);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(1, 2);
        g.add_edge(2, 3);

        assert_eq!(g.min_degree_neighbor(1), 0);
    }
}
