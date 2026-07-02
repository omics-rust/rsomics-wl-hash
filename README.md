# rsomics-wl-hash

Weisfeiler-Lehman (WL) graph fingerprint and per-node subgraph hashes for
undirected graphs — a value-exact Rust port of
`networkx.weisfeiler_lehman_graph_hash` and
`networkx.weisfeiler_lehman_subgraph_hashes` (networkx 3.6.1). Isomorphic graphs
produce identical hashes; the WL kernel is a standard graph-similarity feature
used in graph2vec-style embeddings and network comparison.

The output is the **exact same hex string** networkx produces — the hash is
deterministic (blake2b over a canonical serialisation), so there is no tolerance:
either the string matches or it does not.

## Install

```
cargo install rsomics-wl-hash
```

## Usage

Input is an undirected edge list on stdin (or a file argument): one `u v` pair
per line, arbitrary string node names, `#` comments and blank lines ignored.

```
# whole-graph hash (default: --mode graph, --iterations 3, --digest-size 16)
rsomics-wl-hash < edges.txt
6239a89f4422dc9abfb870b4dcb9f843

# per-node subgraph hash sequences (sorted by node name)
rsomics-wl-hash --mode subgraph --iterations 3 < edges.txt
0	183262a1ae6456f5...	ee7135842d783a11...	0018537aa1d3ffe5...
1	c71c3e57d858543b...	3f2d3b630da6ac67...	4683a186b8c487c0...
...

# JSON envelope
rsomics-wl-hash --json < edges.txt
```

Flags: `--mode graph|subgraph`, `--iterations N` (default 3), `--digest-size N`
bytes (default 16; hex string length is `2 * digest_size`).

### Input / node-set contract

The graph is the simple undirected graph induced by the edge list, matching
`nx.Graph`:

- Parallel (duplicate) edges are deduplicated.
- Self-loops (`u u`) are kept, as `nx.Graph` keeps them: a self-loop appears
  once in the node's neighbour list but adds two to its degree (the default WL
  initial label), so it changes the hash.
- The **node set is exactly the endpoints seen in the edge list** — there is no
  way to introduce an isolated (degree-0) node through an edge list, and none is
  added. If you need isolated nodes in the hash you must supply them via a
  different input path (not supported here, matching how a bare edge list is
  read).
- Node names are opaque strings; only the graph topology (via node degrees)
  enters the hash. Edge and node attributes are **not** used (the plain
  edge-list case).

### A note on the v3.5 hashing change

networkx changed the hash values for undirected graphs without node/edge
attributes in **v3.5** (bugfix, [issue #7806](https://github.com/networkx/networkx/issues/7806)):
the degree initialisation now counts as the first WL iteration, so an internal
`iterations -= 1` is applied. This crate reproduces the **post-v3.5 / 3.6.1**
behaviour. To recover the pre-v3.5 hash of an unlabelled undirected graph,
increase `--iterations` by one.

## Origin

This crate is an independent Rust reimplementation of NetworkX's
Weisfeiler-Lehman graph hashing based on:

- The NetworkX 3.6.1 reference implementation
  (`networkx.algorithms.graph_hashing`), which is BSD-3-Clause licensed and was
  read and cited directly.
- The published method: Shervashidze, Schweitzer, van Leeuwen, Mehlhorn &
  Borgwardt, "Weisfeiler-Lehman Graph Kernels", *Journal of Machine Learning
  Research* 12 (2011).
  <http://www.jmlr.org/papers/volume12/shervashidze11a/shervashidze11a.pdf>

The hash uses `blake2b` node-label digests and reproduces NetworkX's exact
string serialisation at every step (initial degree labels, sorted-neighbour
aggregation, and the `str(tuple(...))` histogram), so the final hex digest is
byte-identical to NetworkX. Test fixtures are Zachary's karate club and
seeded `gnm_random_graph` instances; expected hashes are captured from NetworkX
3.6.1 and committed as constants.

License: MIT OR Apache-2.0.
Upstream credit: NetworkX <https://networkx.org/> (BSD-3-Clause).
