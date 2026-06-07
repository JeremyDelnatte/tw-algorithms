//! This library provides the algorithms to compute the treewidth of a graph, as well as the data
//! structures to represent graphs and other related utils. Graphs can be represented in two ways:
//! as an adjacency list or as a bitset. The library provides the same algorithms for both
//! representations. Bitset representations are more memory efficient and are much faster to
//! compute with.
//!
//! # Examples
//!
//! ```
//! use tw_algorithms::graph::Graph;
//! use tw_algorithms::treewidth::exact;
//! use tw_algorithms::treewidth::heuristic;
//!
//! let graph = Graph::from_g6("FJ\\~w", true).unwrap();
//! let treewidth = exact::branch_bound::treewidth(&graph);
//! assert_eq!(treewidth, 5);
//!
//! let treewidth = exact::rec::treewidth(&graph);
//! assert_eq!(treewidth, 5);
//!
//! let upper_bound = heuristic::min_degree(&graph);
//! ````

pub mod graph;
pub mod treewidth;
pub mod utils;
