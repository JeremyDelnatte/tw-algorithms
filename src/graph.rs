#![allow(dead_code)]
use std::{fs::File, io::{self, BufRead, BufReader}};

use crate::utils::g6::{self, get_edges, get_size};

#[derive(Debug, Clone)]
pub struct Graph {
    n: usize,
    m: usize,
    adjency: Vec<Vec<usize>>,
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
}

// impl PartialEq for Graph {
//     fn eq(&self, other: &Self) -> bool {
//         if self.n != other.n || self.m != other.m {
//             return false;
//         }
//
//         let num_neighbors_vertices: HashMap<usize, Vec<usize>> = self.adjency.iter()
//             .enumerate()
//             .map(|(i, neighbors)| (i, neighbors.len()))
//             .fold(HashMap::new(), |mut acc, (i, len)| {
//                 acc.entry(len).or_insert(Vec::new()).push(i);
//                 acc
//             });
//
//         
//         true
//     }
// }
//
// impl Eq for Graph {}
