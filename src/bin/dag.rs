// dag_matcher.rs — DAG isomorphism (single sink, connected) with extra predicate `f`
// -----------------------------------------------------------------------------
// Optimisations adopted:
// 1. Graphs are connected and each has a unique sink (out‑deg == 0) ⇒ no top‑
//    level matching: just start recursion at the two sinks.
// 2. Every node has a cheap `kind: u8`; kinds must be equal.  `kind` is folded
//    into the structural fingerprint, so non‑matching kinds never enter the
//    search.
// 3. Caller still supplies an arbitrary predicate `f(&Node,&Node)->bool` that
//    must be satisfied **in addition** to kind equality.  `f` is only evaluated
//    on candidates inside the same fingerprint bucket.
// 4. Search = MRV back‑tracking with edge‑consistency pruning.  No Hopcroft–Karp.
//
// Build:  cargo run --release
// Rust 2021.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use itertools::Itertools;

// ------------------------------------------------------------
// Node payload
// ------------------------------------------------------------
#[derive(Clone, Debug)]
struct Node {
    kind: u8,           // lightweight categorical tag
    name: &'static str, // just for display / debugging
}

// ------------------------------------------------------------
// Minimal DAG container (acyclic, connected, single sink assumed)
// ------------------------------------------------------------
#[derive(Debug)]
struct Dag {
    nodes: Vec<Node>,
    adj: Vec<Vec<usize>>, // u -> children
    rev: Vec<Vec<usize>>, // u -> parents (pre‑computed)
}

impl Dag {
    fn new(nodes: Vec<Node>, edges: &[(usize, usize)]) -> Self {
        let n = nodes.len();
        let mut adj = vec![vec![]; n];
        let mut rev = vec![vec![]; n];
        for &(u, v) in edges {
            adj[u].push(v);
            rev[v].push(u);
        }
        Self { nodes, adj, rev }
    }
    fn n(&self) -> usize {
        self.nodes.len()
    }
    fn in_deg(&self, u: usize) -> usize {
        self.rev[u].len()
    }
    fn out_deg(&self, u: usize) -> usize {
        self.adj[u].len()
    }
    fn children(&self, u: usize) -> &[usize] {
        &self.adj[u]
    }
    fn parents(&self, u: usize) -> &[usize] {
        &self.rev[u]
    }

    /// Topological order (Kahn).
    fn topo(&self) -> Vec<usize> {
        let mut indeg: Vec<usize> = self.rev.iter().map(Vec::len).collect();
        let mut q: VecDeque<usize> = (0..self.n()).filter(|&u| indeg[u] == 0).collect();
        let mut order = Vec::with_capacity(self.n());
        while let Some(u) = q.pop_front() {
            order.push(u);
            for &v in &self.adj[u] {
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    q.push_back(v);
                }
            }
        }
        order
    }
}

// ------------------------------------------------------------
// Utility: stable 64‑bit hash
// ------------------------------------------------------------
fn hash_u64<T: Hash>(x: &T) -> u64 {
    let mut h = DefaultHasher::new();
    x.hash(&mut h);
    h.finish()
}

// ------------------------------------------------------------
// Fingerprint = (kind, inDeg, outDeg, multiset(childFP))
// ------------------------------------------------------------
fn fingerprints(g: &Dag) -> Vec<u64> {
    let order = g.topo();
    let mut fp = vec![0u64; g.n()];
    for &u in order.iter().rev() {
        let mut child_fp: Vec<u64> = g.children(u).iter().map(|&c| fp[c]).collect();
        child_fp.sort_unstable();
        fp[u] = hash_u64(&(g.nodes[u].kind, g.in_deg(u), g.out_deg(u), child_fp));
    }
    fp
}

// ------------------------------------------------------------
// Isomorphism with extra predicate `f`
// ------------------------------------------------------------
fn dag_isomorphic<F>(g1: &Dag, g2: &Dag, mut f: F) -> bool
where
    F: FnMut(&Node, &Node) -> bool,
{
    let n = g1.n();
    if n != g2.n() {
        return false;
    }

    // Quick degree multiset check
    let mut mult1 = HashMap::new();
    let mut mult2 = HashMap::new();
    for u in 0..n {
        *mult1.entry((g1.in_deg(u), g1.out_deg(u))).or_insert(0) += 1;
    }
    for v in 0..n {
        *mult2.entry((g2.in_deg(v), g2.out_deg(v))).or_insert(0) += 1;
    }
    if mult1 != mult2 {
        return false;
    }

    // Unique sinks
    let sink1 = (0..n)
        .find(|&u| g1.out_deg(u) == 0)
        .expect("G1 missing sink");
    let sink2 = (0..n)
        .find(|&v| g2.out_deg(v) == 0)
        .expect("G2 missing sink");

    // Fingerprints → buckets (kinds already inside fp)
    let fp1 = fingerprints(g1);
    let fp2 = fingerprints(g2);
    let mut bucket1: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut bucket2: HashMap<u64, Vec<usize>> = HashMap::new();
    for u in 0..n {
        bucket1.entry(fp1[u]).or_default().push(u);
    }
    for v in 0..n {
        bucket2.entry(fp2[v]).or_default().push(v);
    }
    if bucket1.len() != bucket2.len() {
        return false;
    }
    for (sig, b1) in &bucket1 {
        if b1.len() != bucket2.get(sig).map_or(0, |b| b.len()) {
            return false;
        }
    }

    // Candidate sets: same fingerprint & predicate f true
    let mut cand: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for (&sig, list1) in &bucket1 {
        let list2 = &bucket2[&sig];
        for &u in list1 {
            cand[u] = list2
                .iter()
                .copied()
                .filter(|&v| f(&g1.nodes[u], &g2.nodes[v]))
                .collect();
            if cand[u].is_empty() {
                return false;
            }
        }
    }

    // MRV back‑tracking search
    fn dfs(
        g1: &Dag,
        g2: &Dag,
        cand: &mut Vec<HashSet<usize>>,
        used_v: &mut HashSet<usize>,
        mapping: &mut HashMap<usize, usize>,
    ) -> bool {
        if mapping.len() == g1.n() {
            return true;
        }

        // choose node with fewest remaining candidates
        let (u, _) = cand
            .iter()
            .enumerate()
            .filter(|(u, _)| !mapping.contains_key(u))
            .min_by_key(|(_, s)| s.len())
            .unwrap();

        let snapshot = cand.to_vec();
        for &v in cand[u].clone().iter() {
            if used_v.contains(&v) {
                continue;
            }

            // Edge consistency
            let ok_parents = g1.parents(u).iter().all(|&p| {
                mapping
                    .get(&p)
                    .map_or(true, |&vp| g2.parents(v).contains(&vp))
            });
            let ok_children = g1.children(u).iter().all(|&c| {
                mapping
                    .get(&c)
                    .map_or(true, |&vc| g2.children(v).contains(&vc))
            });
            if !(ok_parents && ok_children) {
                continue;
            }

            // Extend mapping
            mapping.insert(u, v);
            used_v.insert(v);
            let mut dead = false;
            for (x, s) in cand.iter_mut().enumerate() {
                if !mapping.contains_key(&x) {
                    s.remove(&v);
                    if s.is_empty() {
                        dead = true;
                        break;
                    }
                }
            }
            if !dead && dfs(g1, g2, cand, used_v, mapping) {
                return true;
            }

            // backtrack
            *cand = snapshot.clone();
            used_v.remove(&v);
            mapping.remove(&u);
        }
        false
    }

    // initial mapping: sink1 → sink2 (must satisfy f too)
    if !f(&g1.nodes[sink1], &g2.nodes[sink2]) {
        return false;
    }
    let mut used_v = HashSet::from([sink2]);
    let mut mapping = HashMap::from([(sink1, sink2)]);

    let success = dfs(g1, g2, &mut cand, &mut used_v, &mut mapping);
    if success {
        println!("Mapping:");
        for (u, v) in mapping.into_iter().sorted() {
            println!("  {} → {}", g1.nodes[u].name, g2.nodes[v].name);
        }
    }
    success
}

// ------------------------------------------------------------
// Demo (diamond DAG)
// ------------------------------------------------------------
fn main() {
    /*
           A                W
          / \              / \
         B   C    ↔      X   Y
          \ /              \ /
           D                Z
    */

    let g1 = Dag::new(
        vec![
            Node { kind: 1, name: "A" }, // 0
            Node { kind: 2, name: "B" }, // 1
            Node { kind: 2, name: "C" }, // 2
            Node { kind: 3, name: "D" }, // 3 (sink)
        ],
        &[(0, 1), (0, 2), (1, 3), (2, 3)],
    );

    let g2 = Dag::new(
        vec![
            Node { kind: 1, name: "W" }, // 0
            Node { kind: 2, name: "X" }, // 1
            Node { kind: 2, name: "Y" }, // 2
            Node { kind: 3, name: "Z" }, // 3 (sink)
        ],
        &[(0, 1), (0, 2), (1, 3), (2, 3)],
    );

    // Break an edge to get non‑isomorphic variant
    // let g3 = Dag::new(
    //     vec![
    //         Node { kind: 1, name: "W" },
    //         Node { kind: 2, name: "X" },
    //         Node { kind: 2, name: "Y" },
    //         Node { kind: 3, name: "Z" },
    //     ],
    //     &[(0, 1), (0, 2), (1, 3) /* (2,3) missing */],
    // );

    // predicate f: kinds already equal; extra rule: names' first letters must differ
    let pred = |a: &Node, b: &Node| a.name.chars().next() != b.name.chars().next();

    println!("g1 vs g2 = {}", dag_isomorphic(&g1, &g2, pred)); // true
    // println!("g1 vs g3 = {}", dag_isomorphic(&g1, &g3, pred)); // false
}
