use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use rsomics_wl_hash::{graph_hash, parse_edge_list, subgraph_hashes};

const GNM: &str = include_str!("../tests/golden/gnm_1000_4000_s1.txt");

fn bench(c: &mut Criterion) {
    // Parse once; time only the WL compute path — mirrors the networkx oracle
    // comparison, which pre-builds the graph and times the hash function.
    let g = parse_edge_list(GNM);
    c.bench_function("wl_graph_hash_gnm_1000_4000", |b| {
        b.iter(|| black_box(graph_hash(black_box(&g), 3, 16)));
    });
    c.bench_function("wl_subgraph_hashes_gnm_1000_4000", |b| {
        b.iter(|| black_box(subgraph_hashes(black_box(&g), 3, 16)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
