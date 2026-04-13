#![allow(dead_code)]
use std::{fs::File, io::{self, BufRead, BufReader}};

use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::utils::g6::{self, get_edges, get_size, to_edges, to_size};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    n: usize,
    m: usize,
    adjency: Vec<Vec<usize>>, // TODO: Test with HashSet
}

impl Graph {
    pub fn new(n: usize) -> Self {
        Graph {
            n,
            m: 0,
            adjency: vec![Vec::new(); n],
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

    pub fn neighbors(&self, i: usize) -> Option<Vec<usize>> {
        self.adjency.get(i).cloned()
    }

    pub fn neighbors_ref(&self, i: usize) -> Option<&Vec<usize>> {
        self.adjency.get(i)
    }

    fn add_arc(&mut self, i: usize, j: usize) -> bool {
        if j >= self.n {
            panic!("Index out of bounds");
        }

        if self.adjency.get(i).expect("index out of bounds").contains(&j) {
            return false;
        }

        self.adjency.get_mut(i).unwrap().push(j);
        true
    }

    pub fn add_edge(&mut self, i: usize, j: usize) -> bool {
        let added = self.add_arc(i, j) && self.add_arc(j, i);
        if added {
            self.m += 1;
        }
        added
    }

    pub fn remove_arc(&mut self, i: usize, j: usize) -> bool {
        if !self.adjency.get_mut(i).expect("index out of bounds").contains(&j) {
            return false;
        }

        self.adjency.get_mut(i).unwrap().retain(|v| *v != j);
        true
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

        self.adjency.get(i).expect("index out of bounds").contains(&j)
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
            for neighbor in neighbors.iter_mut() {
                if *neighbor > v {
                    *neighbor -= 1;
                }
            }
        }

        true
    }

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

        *neighbors.iter()
            .min_by_key(|&&u| self.neighbors_ref(u).unwrap().len())
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

    pub fn from_file(file_path: &str) -> io::Result<Graph> {
        // let file_path = format!("graphs/{}", file_path);
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        let mut reader = BufReader::new(file).lines();

        let first_line = reader.next()
            .expect("Could not load graph, because the file is empty")?;

        let n = first_line.parse::<usize>()
            .expect("Could not load graph, because the first line of the file does not contain the number of vertices in the graph");

        let mut graph = Graph::new(n);

        let err_msg = "Could not load graph, because one line in the file is not in the correct format";
        for line in reader {
            let line = line?;
            let vertices: Vec<&str> = line.split(' ').collect();

            if vertices.len() != 2 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, err_msg));
            }

            let i = vertices.get(0).unwrap().parse::<usize>().expect(err_msg);
            let j = vertices.get(1).unwrap().parse::<usize>().expect(err_msg);

            if !graph.add_edge(i, j) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Could not load an edge of the graph"));
            }
        }

        Ok(graph)
    }

    // TODO: Maybe can use generate_random_with_rng with a default seeded RNG instead of this
    // method.
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


    // #[test]
    // fn test_graph_equality() {
    //     let mut g1 = Graph::new(3);
    //     g1.add_edge(0, 1);
    //     g1.add_edge(1, 2);
    //
    //     let mut g2 = Graph::new(3);
    //     g2.add_edge(0, 1);
    //     g2.add_edge(1, 2);
    //
    //     let mut g3 = Graph::new(3);
    //     g3.add_edge(0, 2);
    //     g3.add_edge(1, 2);
    //
    //     assert_eq!(g1, g2);
    //     assert_ne!(g1, g3);
    // }
}

// impl PartialEq for Graph {
//     fn eq(&self, other: &Self) -> bool {
//         if self.n != other.n || self.m != other.m {
//             return false;
//         }
//
//         // let num_neighbors_vertices: HashMap<usize, Vec<usize>> = self.adjency.iter()
//         //     .enumerate()
//         //     .map(|(i, neighbors)| (i, neighbors.len()))
//         //     .fold(HashMap::new(), |mut acc, (i, len)| {
//         //         acc.entry(len).or_insert(Vec::new()).push(i);
//         //         acc
//         //     });
//         //
//         // 
//         // true
//
//         // for permutation in (0..graph.n()).permutations(graph.n()) {
//         //     let mut matched = true;
//         //
//         //     for i in 0..graph.n() {
//         //         let mapped_i = permutation[i];
//         //         let self_neighbors: Vec<usize> = match self.neighbors(i) {
//         //             Some(neighbors) => neighbors.iter().map(|&v| permutation[v]).collect(),
//         //             None => Vec::new(),
//         //         };
//         //         let other_neighbors: Vec<usize> = match other.neighbors(mapped_i) {
//         //             Some(neighbors) => neighbors.clone(),
//         //             None => Vec::new(),
//         //         };
//         //
//         //         let mut self_neighbors_sorted = self_neighbors.clone();
//         //         self_neighbors_sorted.sort_unstable();
//         //         let mut other_neighbors_sorted = other_neighbors.clone();
//         //         other_neighbors_sorted.sort_unstable();
//         //
//         //         if self_neighbors_sorted != other_neighbors_sorted {
//         //             matched = false;
//         //             break;
//         //         }
//         //     }
//         //
//         //     if matched {
//         //         return true;
//         //     }
//         // }
//     }
// }
//
// impl Eq for Graph {}
