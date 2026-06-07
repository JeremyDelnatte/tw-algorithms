//! Utility binary for converting DIMACS `.col` graph instances into graph6 format.

use std::io::Write;
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
};

use tw_algorithms::graph::adjlist::Graph;

fn graph_from_dimacs_file(file_path: &str) -> io::Result<Graph> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            dbg!(file_path);
            return Err(e);
        }
    };

    let mut reader = BufReader::new(file).lines();

    let n = loop {
        let line = reader
            .next()
            .expect("Could not load graph, because the file is empty")?;

        if line.starts_with('c') {
            continue;
        } else if line.starts_with('p') {
            break line.split_whitespace().nth(2)
                .expect("Could not load graph, because the line starting with 'p' does not contain the number of vertices in the graph")
                .parse::<usize>()
                .expect("Could not load graph, because the line starting with 'p' does not contain a valid number of vertices in the graph");
        } else if line.starts_with('e') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Could not load graph, because the file must contain a 'p' line before any 'e' line",
            ));
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Could not load graph, because the file does not contain a line starting with 'p'",
            ));
        }
    };

    let mut graph = Graph::new(n);
    let err_msg = "Could not load graph, because one line in the file is not in the correct format";

    while let Some(line) = reader.next() {
        let line = line?;
        if line.starts_with('c') {
            continue;
        } else if line.starts_with('e') {
            let mut line_iter = line.split_whitespace();

            let u = line_iter
                .nth(1)
                .expect(err_msg)
                .parse::<usize>()
                .expect(err_msg)
                - 1;

            let v = line_iter
                .next()
                .expect(err_msg)
                .parse::<usize>()
                .expect(err_msg)
                - 1;

            graph.add_edge(u, v);
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, err_msg));
        }
    }

    Ok(graph)
}

fn main() -> io::Result<()> {
    let dir_path = "instances/dimacs";
    let output_path = "instances/dimacs/dimacs.g6";

    let file = File::create(output_path)?;

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("col") {
            let graph = graph_from_dimacs_file(path.to_str().unwrap())?;
            let g6 = graph.to_g6();

            writeln!(
                &file,
                "{} {}",
                path.file_stem().unwrap().to_string_lossy(),
                g6
            )?;
        }
    }

    Ok(())
}
