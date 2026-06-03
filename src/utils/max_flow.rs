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
        self.level.fill(-1);
        self.level[s] = 0;

        let mut q = Vec::with_capacity(self.g.len());
        q.push(s);

        let mut head = 0;

        while head < q.len() {
            let v = q[head];
            head += 1;

            for e in &self.g[v] {
                if e.cap > 0 && self.level[e.to] < 0 {
                    self.level[e.to] = self.level[v] + 1;
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
