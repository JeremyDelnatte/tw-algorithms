use std::collections::{HashMap, HashSet};

use crate::graph::{Graph, adjlist};

pub fn treewidth(graph: &Graph) -> usize {
    match graph {
        Graph::AdjList(g) => {
            let lower_bound = minor_min_width(g);
            let upper_bound = min_fill(g);

            if lower_bound == upper_bound {
                return lower_bound;
            }

            branch_bound_sub(
                g,
                0,
                lower_bound,
                upper_bound,
                None,
                None,
                &mut HashSet::new(),
                &mut HashMap::new(),
            )
        }
        Graph::BitSet(_) => {
            todo!()
        }
    }
}

fn branch_bound_sub(
    graph: &adjlist::Graph,
    g: usize,
    f: usize,
    mut upper_bound: usize,
    last_vertex: Option<usize>,
    last_vertex_neighbors: Option<&Vec<usize>>,
    previous_child_eliminated: &mut HashSet<(usize, usize)>, // For Theorem 6.1
    ancestor_child_eliminated_neighbors: &mut HashMap<usize, Vec<HashSet<usize>>>, // For Theorem 6.2
) -> usize {
    if graph.n() < 2 {
        return std::cmp::min(upper_bound, f);
    }

    let mut new_previous_child_eliminated = HashSet::new(); // For Theorem 6.1
    let mut vertex_elim_added_edges: Vec<HashSet<(usize, usize)>> = Vec::new(); // For Theorem 6.4
    let mut vertex_elim_simplicial_vertices: HashSet<(usize, usize)> = HashSet::new(); // For Theorem 6.3
    let mut child_eliminated_neighbors: Vec<(usize, HashSet<usize>)> = Vec::new(); // For Theorem 6.2

    'outer: for v in 0..graph.n() {
        if let Some(neighbors) = last_vertex_neighbors
            && neighbors.contains(&v)
        {
            continue;

        // For Theorem 6.1
        } else if let Some(lv) = last_vertex
            && previous_child_eliminated.contains(&(v, lv))
        {
            continue;
        }

        // let neighbors = graph.neighbors_ref(v).unwrap().iter().cloned().collect();
        let neighbors = graph.neighbors_ref(v).unwrap(); // NOTE: May be faster to use a vector
        // directly instead of a HashSet here, but need to benchmark.

        // For Theorem 6.2 - Slower for small graphs, but will probably help with larger graphs.
        let neighbors_set: HashSet<usize> = neighbors.iter().cloned().collect();

        if ancestor_child_eliminated_neighbors
            .get(&v)
            .map_or(false, |sets| sets.iter().any(|s| s == &neighbors_set))
        {
            continue;
        }

        ancestor_child_eliminated_neighbors
            .entry(v)
            .or_insert_with(Vec::new)
            .push(neighbors_set.clone());

        child_eliminated_neighbors.push((v, neighbors_set));

        let mut new_graph = graph.clone();
        let added_edges = elim_edges(&mut new_graph, v);

        // For Theorem 6.4 - Slower for small graphs, but will probably help with larger graphs.
        for previous_added_edges in &vertex_elim_added_edges {
            if previous_added_edges.is_subset(&added_edges) {
                continue 'outer;
            }
        }

        vertex_elim_added_edges.push(added_edges);

        if let Some(lv) = last_vertex {
            previous_child_eliminated.insert((lv, v));
        }

        // edge_addition(&mut new_graph, upper_bound); // NOTE: A lot slower with the edge addition rule.

        let new_g = std::cmp::max(g, graph.degree(v));
        let lower_bound = minor_min_width(&new_graph);
        let new_f = std::cmp::max(new_g, lower_bound);

        // For Theorem 6.3 - Improves performance.
        for simplicial in find_all_simplicial_vertices(&new_graph, lower_bound) {
            if vertex_elim_simplicial_vertices.contains(&(simplicial, v)) {
                continue 'outer;
            }

            vertex_elim_simplicial_vertices.insert((v, simplicial));
        }

        // NOTE: For what I've tested, this reduction does not seem to help with performance.
        // In fact, it seems to make it worse.
        let (new_g, new_f) = reduce_graph(&mut new_graph, lower_bound, new_g, new_f);

        if new_f < upper_bound {
            upper_bound = branch_bound_sub(
                &new_graph,
                new_g,
                new_f,
                upper_bound,
                Some(v),
                Some(neighbors),
                &mut new_previous_child_eliminated,
                ancestor_child_eliminated_neighbors,
            );
        }
    }

    // For Theorem 6.2
    for (v, neighbors) in child_eliminated_neighbors {
        ancestor_child_eliminated_neighbors
            .entry(v)
            .or_insert_with(Vec::new)
            .retain(|s| s != &neighbors);
    }

    return upper_bound;
}

fn minor_min_width(g: &adjlist::Graph) -> usize {
    let mut lb = 0;
    let mut g = g.clone();

    loop {
        if g.n() == 0 {
            return lb;
        }

        let min_deg_vertex = g.min_degree_vertex();
        let degree = g.degree(min_deg_vertex);

        if degree == 0 {
            g.remove_vertex(min_deg_vertex);
            continue;
        }

        let min_deg_neighbor = g.min_degree_neighbor(min_deg_vertex);

        lb = lb.max(degree);
        contract_edge(&mut g, min_deg_vertex, min_deg_neighbor);
    }
}

fn contract_edge(g: &mut adjlist::Graph, mut u: usize, v: usize) {
    let neighbors = g.remove_vertex_neighbors(v).unwrap();

    // After removing v, u's index may have decreased by 1.
    if u > v {
        u -= 1;
    }

    for neighbor in neighbors {
        if neighbor != u {
            g.add_edge(u, neighbor);
        }
    }
}

fn elim(g: &mut adjlist::Graph, v: usize) {
    let neighbors = g.remove_vertex_neighbors(v).unwrap();
    let num_neighbors = neighbors.len();

    if num_neighbors == 0 {
        return;
    }

    for i in 0..(num_neighbors - 1) {
        let vertex_i = neighbors[i];
        for j in (i + 1)..num_neighbors {
            g.add_edge(vertex_i, neighbors[j]);
        }
    }
}

fn elim_edges(g: &mut adjlist::Graph, v: usize) -> HashSet<(usize, usize)> {
    let neighbors = g.remove_vertex_neighbors(v).unwrap();
    let num_neighbors = neighbors.len();
    let mut added_edges = HashSet::new();

    if num_neighbors == 0 {
        return added_edges;
    }

    for i in 0..(num_neighbors - 1) {
        let vertex_i = neighbors[i];
        for j in (i + 1)..num_neighbors {
            if g.add_edge(vertex_i, neighbors[j]) {
                added_edges.insert((vertex_i, neighbors[j]));
            }
        }
    }

    added_edges
}

fn edge_fill_vertex(g: &adjlist::Graph, v: usize) -> usize {
    let neighbors = g.neighbors_ref(v).unwrap();
    let num_neighbors = neighbors.len();

    if num_neighbors == 0 {
        return 0;
    }

    let mut edges_missing = 0;
    for i in 0..(num_neighbors - 1) {
        let vertex_i = neighbors[i];
        for j in (i + 1)..num_neighbors {
            if !g.has_edge(vertex_i, neighbors[j]) {
                edges_missing += 1;
            }
        }
    }

    edges_missing
}

fn least_edge_fill_vertex(g: &adjlist::Graph) -> usize {
    let mut min = edge_fill_vertex(g, 0);
    let mut vertex_min = 0;

    // NOTE: In the case of ties, we choose the vertex with the smallest degree, as this will
    // probably lead to a smaller min-fill.
    let mut min_degree = g.degree(0);

    for v in 1..g.n() {
        let fill = edge_fill_vertex(g, v);
        if fill < min || (fill == min && g.degree(v) < min_degree) {
            min = fill;
            vertex_min = v;
            min_degree = g.degree(v);
        }
    }

    vertex_min
}

fn min_fill(g: &adjlist::Graph) -> usize {
    let mut g = g.clone();
    let mut max_clique = 0;

    while g.n() > 0 {
        let v = least_edge_fill_vertex(&g);
        max_clique = max_clique.max(g.degree(v));
        elim(&mut g, v);
    }

    max_clique
}

// NOTE: Not necessary for the branch and bound algorithm, but is used to reduce the branching
// factor at each state by eliminating simplicial and almost simplicial vertices.
fn reduce_graph(
    graph: &mut adjlist::Graph,
    lower_bound: usize,
    mut g: usize,
    mut f: usize,
) -> (usize, usize) {
    while let Some(v) = find_simplicial_vertex(graph, lower_bound) {
        g = g.max(graph.degree(v));
        f = f.max(g);
        elim(graph, v);
    }

    (g, f)
}

fn edge_addition(g: &mut adjlist::Graph, upper_bound: usize) {
    for v in 0..(g.n() - 1) {
        let neighbors_v = g.neighbors_ref(v).unwrap();
        let set_v: HashSet<usize> = neighbors_v.iter().cloned().collect();

        for w in (v + 1)..g.n() {
            let neighbors_w = g.neighbors_ref(w).unwrap();
            let set_w: HashSet<usize> = neighbors_w.iter().cloned().collect();

            let common_neighbors_len = set_v.intersection(&set_w).count();

            if common_neighbors_len > upper_bound {
                g.add_edge(v, w);
            }
        }
    }
}

// Finds an arbitrary simplicial vertex, or an almost simplicial vertex that has a degree <=
// lower_bound, in the graph. Returns None if no such vertex exists.
fn find_simplicial_vertex(g: &adjlist::Graph, lower_bound: usize) -> Option<usize> {
    for v in 0..g.n() {
        let neighbors = g.neighbors_ref(v).unwrap();
        let num_neighbors = neighbors.len();

        let mut is_simplicial = true;
        let mut missing_edge = false;
        'outer: for i in 0..num_neighbors {
            let vertex_i = neighbors[i];
            for j in (i + 1)..num_neighbors {
                if !g.has_edge(vertex_i, neighbors[j]) {
                    if missing_edge || g.degree(v) > lower_bound {
                        // Not simplicial or almost simplicial with degree <= lower_bound.
                        is_simplicial = false;
                        break 'outer;
                    } else {
                        missing_edge = true;
                    }
                }
            }
        }

        if is_simplicial {
            return Some(v);
        }
    }

    None
}

fn find_all_simplicial_vertices(g: &adjlist::Graph, lower_bound: usize) -> Vec<usize> {
    let mut simplicial_vertices = Vec::new();

    for v in 0..g.n() {
        let neighbors = g.neighbors_ref(v).unwrap();
        let num_neighbors = neighbors.len();

        let mut is_simplicial = true;
        let mut missing_edge = false;
        'outer: for i in 0..num_neighbors {
            let vertex_i = neighbors[i];
            for j in (i + 1)..num_neighbors {
                if !g.has_edge(vertex_i, neighbors[j]) {
                    if missing_edge || g.degree(v) > lower_bound {
                        // Not simplicial or almost simplicial with degree <= lower_bound.
                        is_simplicial = false;
                        break 'outer;
                    } else {
                        missing_edge = true;
                    }
                }
            }
        }

        if is_simplicial {
            simplicial_vertices.push(v);
        }
    }

    simplicial_vertices
}

#[cfg(test)]
mod tests {
    use super::contract_edge;
    use crate::graph::adjlist;

    // TODO: fix
    // #[test]
    // fn test_contract_edge() {
    //     let mut g = adjlist::Graph::new(4);
    //     g.add_edge(0, 1);
    //     g.add_edge(1, 2);
    //     g.add_edge(2, 3);
    //     g.add_edge(0, 3);
    //
    //     let contracted_g = contract_edge(&g, 1, 2);
    //
    //     let mut expected_g = adjlist::Graph::new(3);
    //     expected_g.add_edge(0, 1);
    //     expected_g.add_edge(0, 2);
    //     expected_g.add_edge(1, 2);
    //
    //     assert_eq!(contracted_g, expected_g);
    // }
}
