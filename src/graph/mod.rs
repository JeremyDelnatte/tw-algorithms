use crate::utils::{bitset::Bits, g6::{self, get_edges, get_size}};

pub mod adjlist;
pub mod bitset;

#[derive(Debug, Clone)]
pub enum Graph {
    AdjList(adjlist::Graph),
    BitSet(bitset::Graph),
}

impl Graph {
    pub fn from_g6(repr: &str) -> Result<Self, g6::Error> {
        let bytes = repr.as_bytes();
        let n = get_size(bytes)?;
        let edges = get_edges(&bytes[1..], n)?;

        if n <= Bits::BITS as usize {
            let mut graph = bitset::Graph::new(n);
            for (i, j) in edges {
                graph.add_edge(i, j);
            }
            Ok(Graph::BitSet(graph))
        } else {
            let mut graph = adjlist::Graph::new(n);
            for (i, j) in edges {
                graph.add_edge(i, j);
            }
            Ok(Graph::AdjList(graph))
        }
    }

    pub fn to_g6(&self) -> String {
        match self {
            Graph::AdjList(g) => g.to_g6(),
            Graph::BitSet(g) => unimplemented!(),
        }
    }

    pub fn generate_random(n: usize, m: usize) -> Self {
        if n <= Bits::BITS as usize {
            Graph::BitSet(bitset::Graph::generate_random(n, m))
        } else {
            Graph::AdjList(adjlist::Graph::generate_random(n, m))
        }
    }
}
