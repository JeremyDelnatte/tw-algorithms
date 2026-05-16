use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{graph::{Graph, adjlist, newbitset}, treewidth::heuristic::min_fill, utils::newbitset::NewBitSet};

pub fn treewidth(graph: &Graph) -> usize {
    let upper_bound = min_fill(graph);

    match graph {
        Graph::AdjList(g) => {
            let lower_bound = minor_min_width(g);

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
        Graph::NewBitSet(g) => {
            let lower_bound = minor_min_width_bitset(g);

            if lower_bound == upper_bound {
                return lower_bound;
            }

            branch_bound_sub_bitset(
                g,
                0,
                lower_bound,
                upper_bound,
                None,
                None,
                &mut HashSet::new(),
                &mut HashMap::new(),
            )
        },
        _ => panic!("Unsupported graph type"),
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

    // let mut child_eliminated_neighbors: Vec<(usize, HashSet<usize>)> = Vec::new(); // For Theorem 6.2
    let mut child_eliminated: Vec<usize> = Vec::new();

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

        // child_eliminated_neighbors.push((v, neighbors_set));
        child_eliminated.push(v);

        let mut new_graph = graph.clone();
        let added_edges = new_graph.elim_vertex_edges(v);

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

    // TODO: Since i push, i can probably have a more efficient way to remove the last pushed element here instead of using retain, which will be slower.
    // For Theorem 6.2
    // for (v, neighbors) in child_eliminated_neighbors {
    //     ancestor_child_eliminated_neighbors
    //         .entry(v)
    //         .or_insert_with(Vec::new)
    //         .retain(|s| s != &neighbors);
    // }

    for v in child_eliminated {
        ancestor_child_eliminated_neighbors
            .entry(v)
            .or_insert_with(Vec::new) // NOTE: Should always exist.
            .pop();
    }

    return upper_bound;
}

fn branch_bound_sub_bitset(
    graph: &newbitset::Graph,
    g: usize,
    f: usize,
    mut upper_bound: usize,
    last_vertex: Option<usize>,
    last_vertex_neighbors: Option<&NewBitSet>,
    previous_child_eliminated: &mut HashSet<(usize, usize)>, // For Theorem 6.1
    ancestor_child_eliminated_neighbors: &mut HashMap<usize, Vec<NewBitSet>>, // For Theorem 6.2
) -> usize {
    if graph.n() < 2 {
        return std::cmp::min(upper_bound, f);
    }

    let mut new_previous_child_eliminated = HashSet::new(); // For Theorem 6.1
    let mut vertex_elim_added_edges: Vec<HashSet<(usize, usize)>> = Vec::new(); // For Theorem 6.4
    let mut vertex_elim_simplicial_vertices: HashSet<(usize, usize)> = HashSet::new(); // For Theorem 6.3

    let mut child_eliminated: Vec<usize> = Vec::new();

    'outer: for v in 0..graph.n() {
        if let Some(neighbors) = last_vertex_neighbors
            && neighbors.contains(v)
        {
            continue;

        // For Theorem 6.1
        } else if let Some(lv) = last_vertex
            && previous_child_eliminated.contains(&(v, lv))
        {
            continue;
        }

        let neighbors = graph.neighbors_ref(v).unwrap();

        // For Theorem 6.2 - Slower for small graphs, but will probably help with larger graphs.
        let neighbors_set = neighbors.clone();

        if ancestor_child_eliminated_neighbors
            .get(&v)
            .map_or(false, |sets| sets.iter().any(|s| s == &neighbors_set))
        {
            continue;
        }

        ancestor_child_eliminated_neighbors
            .entry(v)
            .or_insert_with(Vec::new)
            .push(neighbors_set);

        child_eliminated.push(v);

        let mut new_graph = graph.clone();
        let added_edges = new_graph.elim_vertex_edges(v);

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

        // NOTE: Seems to be a bit slower with the edge addition rule.
        // edge_addition_bitset(&mut new_graph, upper_bound);

        let new_g = std::cmp::max(g, graph.degree(v));
        let lower_bound = minor_min_width_bitset(&new_graph);
        let new_f = std::cmp::max(new_g, lower_bound);

        // For Theorem 6.3 - Improves performance.
        for simplicial in find_all_simplicial_vertices_bitset(&new_graph, lower_bound) {
            if vertex_elim_simplicial_vertices.contains(&(simplicial, v)) {
                continue 'outer;
            }

            vertex_elim_simplicial_vertices.insert((v, simplicial));
        }

        // NOTE: This improves performance a lot for bigger graphs.
        let (new_g, new_f) = reduce_graph_bitset(&mut new_graph, lower_bound, new_g, new_f);

        if new_f < upper_bound {
            upper_bound = branch_bound_sub_bitset(
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

    for v in child_eliminated {
        ancestor_child_eliminated_neighbors
            .entry(v)
            .or_insert_with(Vec::new) // Should always exist.
            .pop();
    }

    upper_bound
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
        g.contract_edge(min_deg_vertex, min_deg_neighbor);
    }
}

fn minor_min_width_bitset(g: &newbitset::Graph) -> usize {
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
        g.contract_edge(min_deg_vertex, min_deg_neighbor);
    }
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
        graph.elim_vertex(v);
    }

    (g, f)
}

fn reduce_graph_bitset(
    graph: &mut newbitset::Graph,
    lower_bound: usize,
    mut g: usize,
    mut f: usize,
) -> (usize, usize) {
    while let Some(v) = find_simplicial_vertex_bitset(graph, lower_bound) {
        g = g.max(graph.degree(v));
        f = f.max(g);
        graph.elim_vertex(v);
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

fn edge_addition_bitset(g: &mut newbitset::Graph, upper_bound: usize) {
    for u in 0..(g.n() - 1) {
        for v in (u + 1)..g.n() {
            let neighbors_u = g.neighbors_ref(u).unwrap();
            let neighbors_v = g.neighbors_ref(v).unwrap();
            let common_neighbors_len = neighbors_u.intersection_len(neighbors_v);

            if common_neighbors_len > upper_bound {
                g.add_edge(u, v);
            }
        }
    }
}

// Finds an arbitrary simplicial vertex, or an almost simplicial vertex that has a degree <=
// lower_bound, in the graph. Returns None if no such vertex exists.
fn find_simplicial_vertex(g: &adjlist::Graph, lower_bound: usize) -> Option<usize> {
    for v in 0..g.n() {
        if is_simplicial_or_almost_simplicial(g, v, lower_bound) {
            return Some(v);
        }
    }

    None
}

fn find_simplicial_vertex_bitset(g: &newbitset::Graph, lower_bound: usize) -> Option<usize> {
    for v in 0..g.n() {
        if is_simplicial_or_almost_simplicial_bitset(g, v, lower_bound) {
            return Some(v);
        }
    }

    None
}

fn find_all_simplicial_vertices(g: &adjlist::Graph, lower_bound: usize) -> Vec<usize> {
    let mut simplicial_vertices = Vec::new();

    for v in 0..g.n() {
        if is_simplicial_or_almost_simplicial(g, v, lower_bound) {
            simplicial_vertices.push(v);
        }
    }

    simplicial_vertices
}

fn find_all_simplicial_vertices_bitset(g: &newbitset::Graph, lower_bound: usize) -> Vec<usize> {
    let mut simplicial_vertices = Vec::new();

    for v in 0..g.n() {
        if is_simplicial_or_almost_simplicial_bitset(g, v, lower_bound) {
            simplicial_vertices.push(v);
        }
    }

    simplicial_vertices
}

fn is_simplicial_or_almost_simplicial(
    g: &adjlist::Graph,
    v: usize,
    lower_bound: usize,
) -> bool {
    let neighbors = g.neighbors_ref(v).unwrap();
    let num_neighbors = neighbors.len();

    let mut potential_excluded_neighbors = None;
    let mut excluded_neighbor = None;

    for i in 0..num_neighbors {
        let a = neighbors[i];

        if let Some(x) = excluded_neighbor && a == x {
            continue;
        }

        for j in (i + 1)..num_neighbors {
            let b = neighbors[j];

            if let Some(x) = excluded_neighbor && b == x {
                continue;
            }

            if g.has_edge(a, b) {
                continue;
            }

            if excluded_neighbor.is_some() {
                return false;
            }

            if g.degree(v) > lower_bound {
                return false;
            }

            if let Some((x, y)) = potential_excluded_neighbors {
                if a == x || a == y {
                    excluded_neighbor = Some(a);
                } else if b == x || b == y {
                    excluded_neighbor = Some(b);
                } else {
                    return false;
                }
            } else {
                potential_excluded_neighbors = Some((a, b));
            }
        }
    }

    true
}

fn is_simplicial_or_almost_simplicial_bitset(
    g: &newbitset::Graph,
    v: usize,
    lower_bound: usize,
) -> bool {
    let neighbors = g.neighbors_ref(v).unwrap();

    let mut potential_excluded_neighbors = None;
    let mut excluded_neighbor = None;

    for (a, b) in neighbors.iter().tuple_combinations() {
        if let Some(x) = excluded_neighbor && (a == x || b == x) {
            continue;
        }

        if g.has_edge(a, b) {
            continue;
        }

        if excluded_neighbor.is_some() {
            return false;
        }

        if g.degree(v) > lower_bound {
            return false;
        }

        if let Some((x, y)) = potential_excluded_neighbors {
            if a == x || a == y {
                excluded_neighbor = Some(a);
            } else if b == x || b == y {
                excluded_neighbor = Some(b);
            } else {
                return false;
            }
        } else {
            potential_excluded_neighbors = Some((a, b));
        }
    }

    true
}
