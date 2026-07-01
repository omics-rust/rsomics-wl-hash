use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use rsomics_common::{run, CommonFlags, RsomicsError, ToolMeta};
use serde::Serialize;

use rsomics_wl_hash::{graph_hash_from_edge_list, subgraph_hashes_from_edge_list, NodeHashes};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Single WL graph fingerprint (`weisfeiler_lehman_graph_hash`).
    Graph,
    /// Per-node subgraph hash sequences (`weisfeiler_lehman_subgraph_hashes`).
    Subgraph,
}

/// Weisfeiler-Lehman graph hash / subgraph hashes of an undirected graph
/// (`networkx.weisfeiler_lehman_graph_hash` / `..._subgraph_hashes`).
///
/// Reads an edge list (`u v` per line; `#` comments and blank lines skipped;
/// string node names; parallel edges deduplicated and self-loops dropped as in
/// a simple `nx.Graph`; the node set is the edge endpoints). `graph` mode prints
/// one hex hash; `subgraph` mode prints `node hash1 hash2 ...` per node, sorted
/// by node name.
#[derive(Parser, Debug)]
#[command(name = "rsomics-wl-hash", version, about, long_about = None)]
pub struct Cli {
    /// Edge list; `-` or omitted reads stdin.
    #[arg(value_name = "EDGES")]
    pub edges: Option<PathBuf>,

    /// Output the whole-graph hash or per-node subgraph hashes.
    #[arg(long, value_enum, default_value_t = Mode::Graph)]
    pub mode: Mode,

    /// Number of neighbour-aggregation rounds.
    #[arg(long, default_value_t = 3)]
    pub iterations: usize,

    /// blake2b digest size in bytes (hex string length is twice this).
    #[arg(long, default_value_t = 16)]
    pub digest_size: usize,

    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Output {
    Graph { hash: String },
    Subgraph { nodes: Vec<NodeHashes> },
}

impl Cli {
    pub fn run(self) -> ExitCode {
        let common = self.common.clone();
        run(&common, META, || {
            let mut input = String::new();
            match &self.edges {
                Some(p) if p.as_os_str() != "-" => {
                    File::open(p)
                        .map_err(RsomicsError::Io)?
                        .read_to_string(&mut input)
                        .map_err(RsomicsError::Io)?;
                }
                _ => {
                    io::stdin()
                        .lock()
                        .read_to_string(&mut input)
                        .map_err(RsomicsError::Io)?;
                }
            }

            if self.iterations == 0 {
                return Err(RsomicsError::InvalidInput(
                    "the WL algorithm requires that --iterations be positive".to_string(),
                ));
            }
            if self.digest_size == 0 || self.digest_size > 64 {
                return Err(RsomicsError::InvalidInput(
                    "--digest-size must be in 1..=64 (blake2b limit)".to_string(),
                ));
            }

            let output = match self.mode {
                Mode::Graph => Output::Graph {
                    hash: graph_hash_from_edge_list(&input, self.iterations, self.digest_size),
                },
                Mode::Subgraph => Output::Subgraph {
                    nodes: subgraph_hashes_from_edge_list(
                        &input,
                        self.iterations,
                        self.digest_size,
                    ),
                },
            };

            if !common.json {
                match &output {
                    Output::Graph { hash } => println!("{hash}"),
                    Output::Subgraph { nodes } => {
                        for nh in nodes {
                            println!("{}\t{}", nh.node, nh.hashes.join("\t"));
                        }
                    }
                }
            }
            Ok(output)
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        super::Cli::command().debug_assert();
    }
}
