//! Weisfeiler-Lehman graph hash & subgraph hashes — value-exact port of
//! `networkx.weisfeiler_lehman_graph_hash` and
//! `networkx.weisfeiler_lehman_subgraph_hashes` for undirected graphs without
//! node or edge attributes (the plain-edge-list case).
//!
//! The initial label of each node is its degree as a decimal string. Each WL
//! iteration recomputes every node's label as `blake2b` of `own_label +
//! "".join(sorted(neighbour_labels))`. `iterations` is decremented by one to
//! account for the degree initialisation counting as the first round — the
//! post-v3.5 (issue #7806) accounting that networkx 3.6.1 uses.
//!
//! The graph hash is `blake2b` of the Python-`repr` of a tuple of
//! `(label, count)` pairs accumulated across the iterations; the string
//! serialisation is reproduced byte-for-byte so the final hex matches nx.
//!
//! Shervashidze, Schweitzer, van Leeuwen, Mehlhorn & Borgwardt,
//! "Weisfeiler-Lehman Graph Kernels", JMLR 12 (2011).

use std::collections::HashMap;

use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use serde::Serialize;

/// Undirected simple graph over interned integer node ids, retaining node names
/// in first-appearance order.
pub struct Graph {
    idx_to_node: Vec<String>,
    adj: Vec<Vec<usize>>,
}

impl Graph {
    fn intern(&mut self, name: &str, table: &mut HashMap<String, usize>) -> usize {
        if let Some(&idx) = table.get(name) {
            return idx;
        }
        let idx = self.idx_to_node.len();
        table.insert(name.to_owned(), idx);
        self.idx_to_node.push(name.to_owned());
        self.adj.push(Vec::new());
        idx
    }

    fn n(&self) -> usize {
        self.idx_to_node.len()
    }

    /// Degree as `nx.Graph.degree` reports it: a self-loop counts twice. The
    /// self-loop is stored once in `adj`, so add one for the second incidence.
    fn degree(&self, node: usize) -> usize {
        self.adj[node].len() + usize::from(self.adj[node].contains(&node))
    }

    /// Node names in first-appearance order (matches `nx.Graph.nodes()`).
    #[must_use]
    pub fn node_names(&self) -> &[String] {
        &self.idx_to_node
    }
}

/// Parse a whitespace-delimited `u v` edge list. Text from the first `#` to
/// end of line is a comment and is discarded before tokenising (matching
/// `nx.parse_edgelist`); blank/all-comment lines are skipped. Parallel edges
/// are deduplicated; self-loops are kept (as
/// `nx.Graph` keeps them), giving the undirected graph `nx.Graph` induces on
/// the same edges. A self-loop appears once in the node's neighbour list but
/// adds two to its degree. The node set is exactly the endpoints seen in the
/// edge list (no isolated nodes).
#[must_use]
pub fn parse_edge_list(input: &str) -> Graph {
    let mut g = Graph {
        idx_to_node: Vec::new(),
        adj: Vec::new(),
    };
    let mut table = HashMap::new();

    for line in input.lines() {
        // nx.parse_edgelist strips a '#' comment anywhere in the line before tokenising.
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(u), Some(v)) = (parts.next(), parts.next()) else {
            continue;
        };
        let ui = g.intern(u, &mut table);
        let vi = g.intern(v, &mut table);
        if !g.adj[ui].contains(&vi) {
            g.adj[ui].push(vi);
            if ui != vi {
                g.adj[vi].push(ui);
            }
        }
    }
    g
}

/// `blake2b(label.encode("ascii"), digest_size).hexdigest()`.
fn hash_label(label: &str, digest_size: usize) -> String {
    let mut h = Blake2bVar::new(digest_size).expect("digest_size in 1..=64");
    h.update(label.as_bytes());
    let mut out = vec![0u8; digest_size];
    h.finalize_variable(&mut out).expect("output size matches");
    hex::encode(out)
}

/// One WL step: new label of a node is `blake2b(own + sorted(neighbour labels))`.
fn wl_step(g: &Graph, labels: &[String], digest_size: usize) -> Vec<String> {
    let n = g.n();
    let mut new_labels = Vec::with_capacity(n);
    let mut nbr_labels: Vec<&str> = Vec::new();
    let mut agg = String::new();
    for node in 0..n {
        nbr_labels.clear();
        nbr_labels.extend(g.adj[node].iter().map(|&w| labels[w].as_str()));
        nbr_labels.sort_unstable();
        agg.clear();
        agg.push_str(&labels[node]);
        for l in &nbr_labels {
            agg.push_str(l);
        }
        new_labels.push(hash_label(&agg, digest_size));
    }
    new_labels
}

/// Initial labels: each node's degree as a decimal string.
fn init_labels(g: &Graph) -> Vec<String> {
    (0..g.n()).map(|i| g.degree(i).to_string()).collect()
}

/// Append the Python `repr` of a `(label, count)` pair to `out`: `('<label>', <count>)`.
/// The label is a hex string (no quotes/backslashes to escape) so a single-quoted
/// wrap reproduces CPython's tuple element repr exactly.
fn push_pair_repr(out: &mut String, label: &str, count: usize) {
    out.push_str("('");
    out.push_str(label);
    out.push_str("', ");
    out.push_str(&count.to_string());
    out.push(')');
}

/// Weisfeiler-Lehman graph hash. Returns a `2 * digest_size` hex string
/// identical to `nx.weisfeiler_lehman_graph_hash(G, iterations, digest_size)`.
///
/// # Panics
/// If `iterations == 0` (networkx raises `ValueError` for non-positive iterations).
#[must_use]
pub fn graph_hash(g: &Graph, iterations: usize, digest_size: usize) -> String {
    assert!(
        iterations > 0,
        "the WL algorithm requires that `iterations` be positive"
    );
    let rounds = iterations - 1;

    let mut labels = init_labels(g);

    // Python: str(tuple(subgraph_hash_counts)) where subgraph_hash_counts is a
    // list of (label, count) pairs, sorted per round by label. Reproduce the
    // outer tuple repr — including the empty `()` and single-element trailing
    // comma — so the hashed string is byte-identical.
    let mut serialized = String::from("(");
    let mut n_pairs = 0usize;
    for _ in 0..rounds {
        labels = wl_step(g, &labels, digest_size);
        let mut counter: HashMap<&str, usize> = HashMap::new();
        for l in &labels {
            *counter.entry(l.as_str()).or_insert(0) += 1;
        }
        let mut items: Vec<(&str, usize)> = counter.into_iter().collect();
        items.sort_unstable_by(|a, b| a.0.cmp(b.0));
        for (label, count) in items {
            if n_pairs > 0 {
                serialized.push_str(", ");
            }
            push_pair_repr(&mut serialized, label, count);
            n_pairs += 1;
        }
    }
    if n_pairs == 1 {
        serialized.push(',');
    }
    serialized.push(')');

    hash_label(&serialized, digest_size)
}

/// Per-node subgraph hashes for one node, in increasing order of neighbourhood depth.
#[derive(Debug, Clone, Serialize)]
pub struct NodeHashes {
    pub node: String,
    pub hashes: Vec<String>,
}

/// Weisfeiler-Lehman subgraph hashes. Each node gets `iterations` hashes (the
/// first is the hashed degree label; each subsequent one is a further WL round),
/// matching `nx.weisfeiler_lehman_subgraph_hashes(G, iterations, digest_size)`.
/// Output rows are sorted by node name.
///
/// # Panics
/// If `iterations == 0`.
#[must_use]
pub fn subgraph_hashes(g: &Graph, iterations: usize, digest_size: usize) -> Vec<NodeHashes> {
    assert!(
        iterations > 0,
        "the WL algorithm requires that `iterations` be positive"
    );
    let rounds = iterations - 1;
    let n = g.n();

    let mut labels = init_labels(g);
    let mut per_node: Vec<Vec<String>> = (0..n)
        .map(|i| vec![hash_label(&labels[i], digest_size)])
        .collect();

    for _ in 0..rounds {
        labels = wl_step(g, &labels, digest_size);
        for (i, l) in labels.iter().enumerate() {
            per_node[i].push(l.clone());
        }
    }

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_unstable_by(|&a, &b| g.idx_to_node[a].cmp(&g.idx_to_node[b]));
    order
        .into_iter()
        .map(|i| NodeHashes {
            node: g.idx_to_node[i].clone(),
            hashes: std::mem::take(&mut per_node[i]),
        })
        .collect()
}

/// Parse an edge list and return the WL graph hash in one call.
#[must_use]
pub fn graph_hash_from_edge_list(input: &str, iterations: usize, digest_size: usize) -> String {
    graph_hash(&parse_edge_list(input), iterations, digest_size)
}

/// Parse an edge list and return the per-node WL subgraph hashes in one call.
#[must_use]
pub fn subgraph_hashes_from_edge_list(
    input: &str,
    iterations: usize,
    digest_size: usize,
) -> Vec<NodeHashes> {
    subgraph_hashes(&parse_edge_list(input), iterations, digest_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_hash_comment_matches_clean_graph() {
        // A '#' anywhere truncates the line: "1 2#note" is edge (1,2), and
        // "0 #x" is the single token "0" so the line is skipped entirely.
        let with_comments = "0 1\n1 2#note\n2 3\n0 #x\n# full line\n";
        let clean = "0 1\n1 2\n2 3\n";

        assert_eq!(
            graph_hash_from_edge_list(with_comments, 3, 16),
            graph_hash_from_edge_list(clean, 3, 16),
        );
    }
}
