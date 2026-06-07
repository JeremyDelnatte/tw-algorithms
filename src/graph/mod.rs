//! Provides graph representations used by the treewidth algorithms. Graphs can be stored as an
//! adjacency list or as a bitset-based adjacency structure.

use crate::utils::g6::{self, get_edges, get_size};

pub mod adjlist;
pub mod bitset;

/// A graph stored in one of the supported internal representations.
#[derive(Debug, Clone)]
pub enum Graph {
    /// A graph represented  by adjacency lists.
    AdjList(adjlist::Graph),

    /// A graph represented by bitsets for each neighborhood.
    BitSet(bitset::Graph),
}

impl Graph {
    /// Parses a graph from graph6 format.
    ///
    /// When `with_bitset` is true, the graph is stored as a bitset-based graph. Otherwise, it is
    /// stored as an adjacency-list graph.
    pub fn from_g6(repr: &str, with_bitset: bool) -> Result<Self, g6::Error> {
        let bytes = repr.as_bytes();
        let n = get_size(bytes)?;
        let edges = get_edges(&bytes[1..], n)?;

        if with_bitset {
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

    /// Converts the graph to graph6 format.
    pub fn to_g6(&self) -> String {
        match self {
            Graph::AdjList(g) => g.to_g6(),
            Graph::BitSet(_) => unimplemented!(),
        }
    }
}
