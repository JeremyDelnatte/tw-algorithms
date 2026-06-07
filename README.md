# Treewidth Algorithms

This repository contains the Rust implementation for a master's thesis on
exact, approximation, and heuristic algorithms for computing treewidth.

<!-- The project exposes both a library crate and a command-line tool named -->
<!-- `tw_algorithms`. The CLI accepts graphs in graph6 format and can run individual -->
<!-- treewidth computations or benchmark algorithms on random and preset instances. -->

## Repository Layout

- `src/treewidth/exact`: exact treewidth algorithms.
- `src/treewidth/approx`: separator-based approximation algorithms.
- `src/treewidth/heuristic`: linear-ordering-based heuristics.
- `src/graph`: adjacency-list and bitset graph representations.
- `src/utils`: graph6 parsing, bitsets, max-flow, and helper utilities.
- `src/cli`: command-line interface and benchmarking commands.
- `src/bin/convert_dimacs.rs`: utility binary for converting DIMACS `.col`
  instances to graph6.
- `instances`: graph instances used for experiments.

## Requirements

Install a recent Rust toolchain with Cargo.

```sh
cargo build
```

In order to improve the performance of the algorithms, the project should be built in release mode:

```sh
cargo build --release
```

To run the tests

```sh
cargo test
```

## Input Format

Single-graph commands expect a graph6 string:

```sh
cargo run -r -- compute-treewidth --graph 'FJ\~w'
```

File-based computation commands expect one graph6 string per line:

```sh
cargo run -r -- compute-treewidth --file instances/house_of_graphs/graphs_n8.g6
```

Benchmarking preset graphs accepts either one graph6 string per line or a
two-column `name graph6` format. This is useful for files such as
DIMACS instances that have names.

## CLI Usage

Print the top-level help:

```sh
cargo run -- --help
```

Global options:

- `--bitset` or `-b`: use the bitset-based graph representation for algorithms
  that support it.
- `--timeout <duration>`: stop a computation after a duration such as `30s`,
  `5m`, `1h`, `250ms`, or `100us`. This option adds an overhead for creating child processes, but does not affect the time measurements of the algorithms themselves.

### Exact Treewidth

```sh
cargo run -r -- compute-treewidth --algorithm branch-bound --graph 'FJ\~w'
```

Alias for the `compute-treewidth` command: `tw`.

Available algorithms:

- `dynamic-prog` (`dp`)
- `recursive` (`rec`)
- `improved-rec` (`imprec`)
- `branch-bound` (`bb`)

Useful options:

- `--file <path>` (`-f`): process all graph6 graphs in a file.
- `--graph <graph6>` (`-g`): compute treewidth for a single graph.
- `--treewidth <value>` (`-t`): assert the expected exact treewidth.
- `--json`: print machine-readable JSON output.

Example:

```sh
cargo run -r -- --bitset --timeout 30s compute-treewidth \
  --algorithm branch-bound \
  --file instances/house_of_graphs/graphs_n10.g6
```

### Approximate Treewidth

```sh
cargo run -r -- approximate-treewidth --algorithm four-half-approx --graph 'FJ\~w'
```

Aliases for the `approximate-treewidth` command: `atw`, `approx`, `a`, `apx`.

Available algorithms:

- `four-approx` (`4apx`)
- `four-half-approx` (`4.5apx`)

Useful options:

- `--file <path>` (`-f`): process all graph6 graphs in a file.
- `--graph <graph6>` (`-g`): compute treewidth for a single graph.
- `--treewidth <value>` (`-t`): provide the optimal treewidth for validation against
  the approximation guarantee.
- `--json`: print machine-readable JSON output.

### Heuristic Treewidth Upper Bounds

```sh
cargo run -r -- heuristic-treewidth --algorithm min-degree-plus-fill --graph 'FJ\~w'
```

Aliases for the `heuristic-treewidth` command: `htw`, `heuristic`, `h`, `heu`.

Available algorithms:

- `min-fill` (`mf`)
- `min-degree` (`md`)
- `min-degree-plus-fill` (`mdpf`)
- `min-sparsest-subgraph` (`mss`)
- `min-fill-degree` (`mfd`)
- `min-degree-fill` (`mdf`)

Useful options:

- `--file <path>` (`-f`): process all graph6 graphs in a file.
- `--graph <graph6>` (`-g`): compute treewidth for a single graph.
- `--treewidth <value>` (`-t`): provide the optimal treewidth and assert that the
  heuristic upper bound is not below it.
- `--json`: print machine-readable JSON output.

## Benchmarking

The benchmark command writes CSV files under `benchmarks/`. It skips an existing
valid result file unless `--force` is passed.

Benchmark random graphs:

```sh
cargo run -r -- benchmark \
  --exact-algorithm branch-bound \
  random-graphs \
  --num-vertices 20 \
  --num-edges 40 \
  --num-iterations 100 \
  --seed 42
```

Benchmark preset graphs:

```sh
cargo run -r -- benchmark \
  --heuristic-algorithm min-degree-plus-fill \
  preset-graphs \
  --graph-file instances/house_of_graphs/graphs_n10.g6 \
  --num-iterations 10
```

Benchmark options:

- `--exact-algorithm <algorithm>`: one of the exact algorithms for treewidth.
- `--approximate-algorithm <algorithm>`: one of the approximation algorithms for treewidth.
- `--heuristic-algorithm <algorithm>`: one of the heuristic algorithms for treewidth.
- `--num-vertices <n>`: number of vertices for random graph generation.
- `--num-edges <m>`: number of edges for random graph generation.
- `--seed <s>`: random seed for random graph generation.
- `--num-iterations <n>`: Number of random graphs to generate or number of iterations to run on each preset graph.
- `--progress-bar` or `-p`: show a progress bar during benchmarking.
- `--force`: overwrite existing benchmark results.

## Memory Measurements

Memory instrumentation is optional and uses the `measure-memory` feature:

```sh
cargo run -r --features measure-memory -- compute-treewidth \
  --algorithm branch-bound \
  --graph 'FJ\~w'
```

When this feature is enabled, JSON and text outputs include allocated bytes and
peak bytes when available. The feature changes allocation behavior and can affect
runtime, so use it for measurement runs rather than normal benchmarking.

## Algorithm References

- `dynamic-prog`, `recursive`, and `improved-rec`: Hans L. Bodlaender, Fedor V. Fomin, Arie M. C. A. Koster,
  Dieter Kratsch, and Dimitrios M. Thilikos, "On Exact Algorithms for
  Treewidth", ESA 2006.
- `branch-bound`: Vibhav Gogate and Rina Dechter, "A Complete Anytime Algorithm
  for Treewidth", 2012.
- `four-approx` and `four-half-approx`: Eyal Amir, "Approximation Algorithms for Treewidth",
  Algorithmica, 2010.
- `min-fill`, `min-degree`, `min-degree-plus-fill`, `min-sparsest-subgraph`, `min-fill-degree`, and `min-degree-fill`: Hans L. Bodlaender and Arie M. C. A. Koster, "Treewidth
  computations I. Upper bounds", Information and Computation, 2010.
