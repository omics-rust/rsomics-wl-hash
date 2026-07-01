//! Value-exact compatibility tests against networkx 3.6.1.
//!
//! Every expected hex string was captured from
//! `nx.weisfeiler_lehman_graph_hash` / `weisfeiler_lehman_subgraph_hashes`
//! (networkx 3.6.1) at build time and hardcoded here. No Python or subprocess
//! runs at test time.

use rsomics_wl_hash::{graph_hash_from_edge_list, subgraph_hashes_from_edge_list, NodeHashes};

const TRIANGLE: &str = "a b\nb c\nc a\n";
const PATH: &str = "1 2\n2 3\n3 4\n";
const STAR: &str = "0 1\n0 2\n0 3\n0 4\n";
const K5: &str = "1 2\n1 3\n1 4\n1 5\n2 3\n2 4\n2 5\n3 4\n3 5\n4 5\n";

const KARATE: &str = include_str!("golden/karate.txt");
const GNM_30_60: &str = include_str!("golden/gnm_30_60_s42.txt");
const GNM_50_120: &str = include_str!("golden/gnm_50_120_s7.txt");
const GNM_1000_4000: &str = include_str!("golden/gnm_1000_4000_s1.txt");

fn gh(input: &str, it: usize, ds: usize) -> String {
    graph_hash_from_edge_list(input, it, ds)
}

#[test]
fn graph_triangle() {
    assert_eq!(gh(TRIANGLE, 3, 16), "d41316288e6b8bab957f3b304df0b032");
    assert_eq!(gh(TRIANGLE, 5, 8), "6524b97d7794a8af");
}

#[test]
fn graph_path() {
    assert_eq!(gh(PATH, 3, 16), "f8186735edd856041244adc66492b285");
    assert_eq!(gh(PATH, 5, 8), "9cb1d808370f5f4c");
}

#[test]
fn graph_star() {
    assert_eq!(gh(STAR, 3, 16), "1be856886b76a78263f7d55beb5cdb33");
    assert_eq!(gh(STAR, 3, 8), "03188077603651a1");
}

#[test]
fn graph_k5() {
    assert_eq!(gh(K5, 3, 16), "b203899d1848b1553527d4bf5ccd8913");
}

// iterations=1 → zero WL rounds → hashed empty tuple `()`; identical for every
// non-trivial graph. This pins the empty-serialisation edge case.
#[test]
fn graph_iterations_one_is_empty_tuple() {
    let empty = "de75f5edfabdb0477e652512e4287161";
    assert_eq!(gh(TRIANGLE, 1, 16), empty);
    assert_eq!(gh(PATH, 1, 16), empty);
    assert_eq!(gh(K5, 1, 16), empty);
    assert_eq!(gh(KARATE, 1, 16), empty);
    assert_eq!(gh(GNM_30_60, 1, 16), empty);
}

#[test]
fn graph_karate() {
    assert_eq!(gh(KARATE, 3, 16), "6239a89f4422dc9abfb870b4dcb9f843");
    assert_eq!(gh(KARATE, 5, 16), "8488a543b913a20dc2784e0c85f2cdf7");
    assert_eq!(gh(KARATE, 3, 8), "4a798434ce1beac7");
    assert_eq!(gh(KARATE, 5, 8), "17bebe7c8b27de2c");
}

#[test]
fn graph_gnm_30_60() {
    assert_eq!(gh(GNM_30_60, 3, 16), "1d39b72ea1c97efe676996585ee110a6");
    assert_eq!(gh(GNM_30_60, 5, 16), "449fbd20c2f357a2e3474b7b7c4264d9");
    assert_eq!(gh(GNM_30_60, 5, 8), "518d5899c570cea4");
    assert_eq!(gh(GNM_30_60, 3, 8), "759c369e9e04dc4c");
}

#[test]
fn graph_gnm_50_120() {
    assert_eq!(gh(GNM_50_120, 3, 16), "0a4e4106ad623c5b0183de4d5c7cacce");
    assert_eq!(gh(GNM_50_120, 5, 16), "6e9392de40125b4527253c88a8206c4b");
    assert_eq!(gh(GNM_50_120, 3, 8), "f2591b4362bf74b3");
}

#[test]
fn graph_gnm_1000_4000() {
    assert_eq!(gh(GNM_1000_4000, 3, 16), "980eadd4456f66b5f51df1d3b090dad0");
    assert_eq!(gh(GNM_1000_4000, 5, 16), "8ee26375be27c276eef3406b9599a072");
    assert_eq!(gh(GNM_1000_4000, 3, 8), "d5256422fe0c6df1");
}

fn node_seq<'a>(rows: &'a [NodeHashes], name: &str) -> &'a [String] {
    &rows
        .iter()
        .find(|r| r.node == name)
        .expect("node present")
        .hashes
}

#[test]
fn subgraph_triangle() {
    let rows = subgraph_hashes_from_edge_list(TRIANGLE, 3, 16);
    let expected = [
        "4129e2a8044a57ce7635fd6023661cd6",
        "0ea5aaaa75acdcc10dfc8f72ac0d4373",
        "fc00910618fc4d778563d53213514b80",
    ];
    for n in ["a", "b", "c"] {
        assert_eq!(node_seq(&rows, n), expected);
    }
    // rows sorted by node name
    let names: Vec<&str> = rows.iter().map(|r| r.node.as_str()).collect();
    assert_eq!(names, ["a", "b", "c"]);
}

#[test]
fn subgraph_path() {
    let rows = subgraph_hashes_from_edge_list(PATH, 3, 16);
    assert_eq!(
        node_seq(&rows, "1"),
        [
            "cea3878a334b240469d159ff840b6434",
            "68f978fe50aca8ee91e0e1f94618c62f",
            "b2f41f60299bb84fe53a8b732dbab232",
        ]
    );
    assert_eq!(
        node_seq(&rows, "2"),
        [
            "4129e2a8044a57ce7635fd6023661cd6",
            "b51b1dced9065019a9c212167446badf",
            "22b3d97b0ad69557c0061f22b6ddea2b",
        ]
    );
    // endpoints identical, inner nodes identical (path symmetry)
    assert_eq!(node_seq(&rows, "4"), node_seq(&rows, "1"));
    assert_eq!(node_seq(&rows, "3"), node_seq(&rows, "2"));
}

#[test]
fn subgraph_star() {
    let rows = subgraph_hashes_from_edge_list(STAR, 2, 8);
    assert_eq!(
        node_seq(&rows, "0"),
        ["711d7b067f3018b6", "0dda8c511cc4b1e8"]
    );
    for leaf in ["1", "2", "3", "4"] {
        assert_eq!(
            node_seq(&rows, leaf),
            ["f6fc42039fba3776", "5110adc459f44b27"]
        );
    }
}

#[test]
fn subgraph_k5() {
    let rows = subgraph_hashes_from_edge_list(K5, 3, 16);
    let expected = [
        "c33cedce35493fc82dfad8ee5206e23e",
        "f1675c7828f405795e71b9f866b0d1e1",
        "523df14852d5348d10ab8aee0d7c02b4",
    ];
    for n in ["1", "2", "3", "4", "5"] {
        assert_eq!(node_seq(&rows, n), expected);
    }
}

#[test]
fn subgraph_karate_nodes() {
    let rows = subgraph_hashes_from_edge_list(KARATE, 3, 16);
    assert_eq!(
        node_seq(&rows, "0"),
        [
            "183262a1ae6456f5e586b93d795fefed",
            "ee7135842d783a112c63461a5ee39aae",
            "0018537aa1d3ffe529beadf0a8c60b35",
        ]
    );
    assert_eq!(
        node_seq(&rows, "33"),
        [
            "1f749db43a13a74afd40d9b0a4fd3fef",
            "9bf47c424742931b5edb709ac903e1ec",
            "48037a7f9f992f045b76f19a257489b7",
        ]
    );
    assert_eq!(
        node_seq(&rows, "1"),
        [
            "c71c3e57d858543b15064b65d8c7e8c8",
            "3f2d3b630da6ac6d760dc5771eabf258",
            "4683a186b8c487c09a82b983ba42e052",
        ]
    );
    assert_eq!(
        node_seq(&rows, "14"),
        [
            "4129e2a8044a57ce7635fd6023661cd6",
            "57fe625e018e910d994db88483014a2f",
            "1d5239dc62f669bb7c926a0f249d8d2b",
        ]
    );
    // one hash per iteration, all 34 nodes present
    assert_eq!(rows.len(), 34);
    assert!(rows.iter().all(|r| r.hashes.len() == 3));
}

#[test]
fn parse_dedups_and_drops_self_loops() {
    // triangle with a duplicate edge, a self-loop, a comment and blank lines —
    // must reduce to the plain triangle hash.
    let noisy = "# header\n\na b\nb a\nb c\n\nc a\nc c\n# trailing\n";
    assert_eq!(gh(noisy, 3, 16), gh(TRIANGLE, 3, 16));
}
