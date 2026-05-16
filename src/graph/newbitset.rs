#![allow(dead_code)]

use std::collections::HashSet;

use itertools::Itertools;
use rand::{RngExt, rngs::StdRng};

use crate::{graph::adjlist, utils::{g6::{self, get_edges, get_size}, newbitset::NewBitSet}};

#[derive(Debug, Clone)]
pub struct Graph {
    n: usize,
    m: usize,
    adjency: Vec<NewBitSet>,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        Graph {
            n,
            m: 0,
            adjency: vec![NewBitSet::new(n); n],
        }
    }

    pub fn new_cycle(n: usize) -> Self {
        let mut graph = Graph::new(n);
        for i in 0..n {
            graph.add_edge(i, (i + 1) % n);
        }
        graph
    }

    pub fn new_complete(n: usize) -> Self {
        let mut graph = Graph::new(n);
        for i in 0..n {
            for j in (i + 1)..n {
                graph.add_edge(i, j);
            }
        }
        graph
    }

    pub fn new_path(n: usize) -> Self {
        let mut graph = Graph::new(n);
        for i in 0..(n - 1) {
            graph.add_edge(i, i + 1);
        }
        graph
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn m(&self) -> usize {
        self.m
    }

    pub fn neighbors(&self, i: usize) -> Option<NewBitSet> {
        self.adjency.get(i).cloned()
    }

    pub fn neighbors_ref(&self, i: usize) -> Option<&NewBitSet> {
        self.adjency.get(i)
    }

    fn add_arc(&mut self, i: usize, j: usize) -> bool {
        if j >= self.n {
            panic!("Index out of bounds");
        }

        if self.adjency.get(i).expect("index out of bounds").contains(j) {
            return false;
        }

        self.adjency.get_mut(i).unwrap().insert(j);
        true
    }

    pub fn remove_arc(&mut self, i: usize, j: usize) -> bool {
        if !self.adjency.get_mut(i).expect("index out of bounds").contains(j) {
            return false;
        }

        self.adjency.get_mut(i).unwrap().remove(j);
        true
    }

    pub fn add_edge(&mut self, i: usize, j: usize) -> bool {
        let added = self.add_arc(i, j) && self.add_arc(j, i);
        if added {
            self.m += 1;
        }
        added
    }

    pub fn remove_edge(&mut self, i: usize, j: usize) -> bool {
        let removed = self.remove_arc(i, j) && self.remove_arc(j, i);
        if removed {
            self.m -= 1;
        }
        removed
    }

    pub fn has_edge(&self, i: usize, j: usize) -> bool {
        if i >= self.n || j >= self.n {
            return false;
        }

        self.adjency.get(i).expect("index out of bounds").contains(j)
    }

    pub fn has_vertex(&self, i: usize) -> bool {
        i < self.n
    }

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
            neighbors.right_shift_from(v);
        }

        true
    }

    pub fn remove_vertex_neighbors(&mut self, v: usize) -> Option<NewBitSet> {
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
            neighbors.right_shift_from(v);
        }

        neighbors_vertex.right_shift_from(v);
        Some(neighbors_vertex)
    }

    pub fn elim_vertex(&mut self, v: usize) {
        let neighbors = self.remove_vertex_neighbors(v).unwrap();
        let num_neighbors = neighbors.len();

        if num_neighbors == 0 {
            return;
        }

        for (u, v) in neighbors.iter().tuple_combinations() {
            self.add_edge(u, v);
        }
    }

    pub fn elim_vertex_edges(&mut self, v: usize) -> HashSet<(usize, usize)> {
        let neighbors = self.remove_vertex_neighbors(v).unwrap();
        let num_neighbors = neighbors.len();
        let mut added_edges = HashSet::new();

        if num_neighbors == 0 {
            return added_edges;
        }

        for (u, v) in neighbors.iter().tuple_combinations() {
            if self.add_edge(u, v) {
                added_edges.insert((u, v));
            }
        }

        added_edges
    }

    fn fill_in_count_vertex(&self, v: usize) -> usize {
        let neighbors = self.neighbors_ref(v).unwrap();
        let num_neighbors = neighbors.len();

        if num_neighbors == 0 {
            return 0;
        }

        let mut edges_missing = 0;
        for (u, v) in neighbors.iter().tuple_combinations() {
            if !self.has_edge(u, v) {
                edges_missing += 1;
            }
        }

        edges_missing
    }

    pub fn least_fill_in_count_vertex(&self) -> usize {
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

    pub fn contract_edge(&mut self, mut u: usize, v: usize) {
        let neighbors = self.remove_vertex_neighbors(v).unwrap();

        // After removing v, u's index may have decreased by 1.
        if u > v {
            u -= 1;
        }

        for neighbor in neighbors.iter() {
            if neighbor != u {
                self.add_edge(u, neighbor);
            }
        }
    }

    pub fn degree(&self, v: usize) -> usize {
        if v >= self.n {
            panic!("Index out of bounds");
        }

        self.adjency[v].len()
    }

    pub fn min_degree_vertex(&self) -> usize {
        if self.n == 0 {
            panic!("Graph has no vertices");
        }

        self.adjency.iter()
            .enumerate()
            .min_by_key(|(_, neighbors)| neighbors.len())
            .map(|(i, _)| i)
            .unwrap()
    }

    pub fn min_degree_neighbor(&self, v: usize) -> usize {
        if v >= self.n {
            panic!("Index out of bounds");
        }

        let neighbors = self.neighbors_ref(v).unwrap();

        if neighbors.is_empty() {
            panic!("Vertex has no neighbors");
        }

        neighbors.iter()
            .min_by_key(|&neighbor| self.degree(neighbor))
            .unwrap()
    }

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

    pub fn to_adjlist_graph(&self) -> adjlist::Graph {
        let mut new_graph = adjlist::Graph::new(self.n);
        for v in 0..self.n {
            for neighbor in self.neighbors_ref(v).unwrap().iter() {
                new_graph.add_edge(v, neighbor);
            }
        }
        new_graph
    }

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
    use super::*;

    #[test]
    fn test_remove_vertex_neighbors() {
        let mut graph = Graph::new(5);
        graph.add_edge(0, 1);
        graph.add_edge(0, 2);
        graph.add_edge(1, 3);
        graph.add_edge(2, 4);

        let neighbors = graph.remove_vertex_neighbors(0).unwrap();

        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(0));
        assert!(neighbors.contains(1));

        assert_eq!(graph.degree(0), 1);
        assert_eq!(graph.degree(1), 1);

        assert!(graph.has_edge(0, 2));
        assert!(graph.has_edge(1, 3));
    }
}
