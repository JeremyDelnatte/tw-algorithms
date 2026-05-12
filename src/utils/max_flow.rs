#[derive(Clone, Debug)]
struct Edge {
    to: usize,
    rev: usize,
    cap: usize,
}

#[derive(Clone, Debug)]
struct Dinic {
    g: Vec<Vec<Edge>>,
    level: Vec<i32>,
    it: Vec<usize>,
}

impl Dinic {
    fn new(n: usize) -> Self {
        Self {
            g: vec![Vec::new(); n],
            level: vec![0; n],
            it: vec![0; n],
        }
    }

    fn with_degrees(deg: &[usize]) -> Self {
        let n = deg.len();

        let mut g = Vec::with_capacity(n);
        for &d in deg {
            g.push(Vec::with_capacity(d));
        }

        Self {
            g,
            level: vec![0; n],
            it: vec![0; n],
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, cap: usize) {
        let rev_to = self.g[to].len();
        let rev_from = self.g[from].len();

        self.g[from].push(Edge { to, rev: rev_to, cap });
        self.g[to].push(Edge { to: from, rev: rev_from, cap: 0 });
    }

    fn bfs(&mut self, s: usize, t: usize) -> bool {
        // use std::collections::VecDeque;

        self.level.fill(-1);
        self.level[s] = 0;

        // let mut q = VecDeque::new();
        // q.push_back(s);

        let mut q = Vec::with_capacity(self.g.len());
        q.push(s);

        let mut head = 0;

        // while let Some(v) = q.pop_front() {
        while head < q.len() {
            let v = q[head];
            head += 1;

            for e in &self.g[v] {
                if e.cap > 0 && self.level[e.to] < 0 {
                    self.level[e.to] = self.level[v] + 1;
                    // q.push_back(e.to);
                    q.push(e.to);
                }
            }
        }

        self.level[t] >= 0
    }

    fn dfs(&mut self, v: usize, t: usize, f: usize) -> usize {
        if v == t || f == 0 {
            return f;
        }

        while self.it[v] < self.g[v].len() {
            let i = self.it[v];
            let to = self.g[v][i].to;

            if self.g[v][i].cap > 0 && self.level[v] + 1 == self.level[to] {
                let pushed = self.dfs(to, t, f.min(self.g[v][i].cap));

                if pushed > 0 {
                    let rev = self.g[v][i].rev;

                    self.g[v][i].cap -= pushed;
                    self.g[to][rev].cap += pushed;

                    return pushed;
                }
            }

            self.it[v] += 1;
        }

        0
    }

    fn max_flow(&mut self, s: usize, t: usize, limit: usize) -> bool {
        let mut flow = 0;

        while flow < limit && self.bfs(s, t) {
            self.it.fill(0);

            loop {
                let pushed = self.dfs(s, t, limit - flow);
                if pushed == 0 {
                    break;
                }

                flow += pushed;

                if flow >= limit {
                    return false;
                }
            }
        }

        true
    }

    fn reachable_from(&self, s: usize) -> Vec<bool> {
        let mut seen = vec![false; self.g.len()];
        let mut stack = vec![s];
        seen[s] = true;

        while let Some(v) = stack.pop() {
            for e in &self.g[v] {
                if e.cap > 0 && !seen[e.to] {
                    seen[e.to] = true;
                    stack.push(e.to);
                }
            }
        }

        seen
    }
}

// pub fn max_flow_reachable(n: usize, edges: &[(usize, usize)], capacities: &[usize], s: usize, t: usize, limit: usize) -> Option<HashSet<usize>> {
//     let mut dinic = Dinic::new(n);
//
//     for (&(u, v), &cap) in edges.iter().zip(capacities.iter()) {
//         dinic.add_edge(u, v, cap);
//     }
//
//     if !dinic.max_flow(s, t, limit) {
//         return None;
//     }
//
//     Some(dinic.reachable_from(s))
// }

pub fn max_flow_reachable(
    n: usize,
    edges: &[(usize, usize)],
    capacities: &[usize],
    s: usize,
    t: usize,
    limit: usize,
) -> Option<Vec<bool>> {
    let mut deg = vec![0usize; n];

    for &(u, v) in edges {
        deg[u] += 1; // forward edge
        deg[v] += 1; // reverse edge
    }

    let mut dinic = Dinic::with_degrees(&deg);

    for (&(u, v), &cap) in edges.iter().zip(capacities.iter()) {
        dinic.add_edge(u, v, cap);
    }

    if !dinic.max_flow(s, t, limit) {
        return None;
    }

    Some(dinic.reachable_from(s))
}

// #[cfg(test)]
// mod tests {
//     use super::max_flow_reachable;
//     use std::collections::HashSet;
//
//     fn set(xs: Vec<usize>) -> HashSet<usize> {
//         xs.into_iter().collect()
//     }
//
//     #[test]
//     fn single_path_capacity_5_below_limit() {
//         let edges = vec![(0, 1)];
//         let caps = vec![5];
//
//         // max-flow = 5 <= limit, so Some(reachable)
//         let reachable = max_flow_reachable(2, &edges, &caps, 0, 1, 10).unwrap();
//
//         // After max-flow, sink is not reachable in residual graph.
//         assert_eq!(reachable, set(vec![0]));
//     }
//
//     #[test]
//     fn single_path_exceeds_limit() {
//         let edges = vec![(0, 1)];
//         let caps = vec![5];
//
//         // flow reaches limit = 3, so reject.
//         assert!(max_flow_reachable(2, &edges, &caps, 0, 1, 3).is_none());
//     }
//
//     #[test]
//     fn two_parallel_paths_below_limit() {
//         let edges = vec![
//             (0, 1), (1, 3),
//             (0, 2), (2, 3),
//         ];
//         let caps = vec![1, 1, 1, 1];
//
//         let reachable = max_flow_reachable(4, &edges, &caps, 0, 3, 3).unwrap();
//
//         // Both outgoing edges from source are saturated.
//         assert_eq!(reachable, set(vec![0]));
//     }
//
//     #[test]
//     fn two_parallel_paths_reaches_limit() {
//         let edges = vec![
//             (0, 1), (1, 3),
//             (0, 2), (2, 3),
//         ];
//         let caps = vec![1, 1, 1, 1];
//
//         assert!(max_flow_reachable(4, &edges, &caps, 0, 3, 2).is_none());
//     }
//
//     #[test]
//     fn disconnected_graph() {
//         let edges = vec![(0, 1), (2, 3)];
//         let caps = vec![5, 5];
//
//         assert_eq!(max_flow_reachable(4, &edges, &caps, 0, 3, 1), Some(set(vec![0, 1])));
//     }
//
//     #[test]
//     fn bottleneck_middle_edge() {
//         let edges = vec![
//             (0, 1),
//             (1, 2),
//             (2, 3),
//         ];
//         let caps = vec![10, 3, 10];
//
//         let reachable = max_flow_reachable(4, &edges, &caps, 0, 3, 4).unwrap();
//
//         // Edge 1 -> 2 is saturated, so reachable side is {0,1}.
//         assert_eq!(reachable, set(vec![0, 1]));
//     }
//
//     #[test]
//     fn bottleneck_reaches_limit() {
//         let edges = vec![
//             (0, 1),
//             (1, 2),
//             (2, 3),
//         ];
//         let caps = vec![10, 3, 10];
//
//         assert!(max_flow_reachable(4, &edges, &caps, 0, 3, 3).is_none());
//     }
//
//     #[test]
//     fn requires_reverse_edges() {
//         let edges = vec![
//             (0, 1),
//             (0, 2),
//             (1, 2),
//             (1, 3),
//             (2, 3),
//         ];
//         let caps = vec![1, 1, 1, 1, 1];
//
//         let reachable = max_flow_reachable(4, &edges, &caps, 0, 3, 3).unwrap();
//
//         // Max-flow is 2. After max-flow, source side should not contain sink.
//         assert!(reachable.contains(&0));
//         assert!(!reachable.contains(&3));
//     }
// }
