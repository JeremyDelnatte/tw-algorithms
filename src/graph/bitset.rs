#![allow(dead_code)]

use crate::{graph::adjlist, utils::{bitset::{BitSet, Bits}, g6::{self, get_edges, get_size}}};

#[derive(Debug, Clone)]
pub struct Graph {
    n: usize,
    m: usize,
    adjency: Vec<BitSet>,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        if n > Bits::BITS as usize {
            panic!("Graph size exceeds BitSet capacity");
        }

        Graph {
            n,
            m: 0,
            adjency: vec![BitSet::new(); n],
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

    pub fn neighbors(&self, i: usize) -> Option<BitSet> {
        self.adjency.get(i).cloned()
    }

    pub fn neighbors_ref(&self, i: usize) -> Option<&BitSet> {
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

    pub fn has_vertex(&self, i: usize) -> bool {
        i < self.n
    }

    pub fn from_g6(repr: &str) -> Result<Self, g6::Error> {
        let bytes = repr.as_bytes();
        let n = get_size(bytes)?;
        let edges = get_edges(&bytes[1..], n)?;
        let mut graph = Graph::new(n);
        for (i, j) in edges {
            graph.add_edge(i, j);
        }
        Ok(graph)
    }

    pub fn to_adjlist_graph(&self) -> adjlist::Graph {
        let mut new_graph = adjlist::Graph::new(self.n);
        for v in 0..self.n {
            for neighbor in self.neighbors(v).unwrap() {
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
}
