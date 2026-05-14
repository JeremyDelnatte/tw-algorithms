use serde::Serialize;
use strum::EnumIter;

use crate::{graph::Graph, treewidth::exact::branch_bound::{self}};

#[derive(EnumIter, Serialize, Debug, Clone, Copy)]
pub enum HeuristicAlgorithm {
    MinFill,
}

pub fn min_fill(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(adjlist) => branch_bound::min_fill(adjlist),
        Graph::NewBitSet(_) => todo!(),
        _ => todo!("Remove other graph types and update this function accordingly"),
    }
}
