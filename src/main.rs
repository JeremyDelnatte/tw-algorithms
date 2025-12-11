use std::io::BufRead;

mod graph;
mod utils;
mod treewidth;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // // let g = graph::Graph::from_g6("IF~~~~~~w")?;
    // let g = graph::Graph::from_g6("I")?;
    // // let g = graph::Graph::from_g6("E@lw")?;
    // // let mut g = graph::Graph::new(3);
    // // g.add_edge(0, 1);
    // // g.add_edge(1, 2);
    // // g.add_edge(0, 2);
    //
    // println!("{:?}", g);
    //
    // let treewidth = treewidth::dynamic_prog::treewidth(&g);
    // println!("Treewidth: {}", treewidth);
    // Ok(())


    // let mut g = Graph::new(4);
    // g.add_edge(0, 1);
    // g.add_edge(1, 2);
    // g.add_edge(0, 2);
    // g.add_edge(2, 3);
    //
    // let mut g2 = Graph::new(4);
    // g2.add_edge(0, 1);
    // g2.add_edge(1, 2);
    // g2.add_edge(0, 2);
    // g2.add_edge(1, 3);
    // g.eq(&g2);

    let mut count = 0;
    let file = std::fs::File::open("graphs-tw4.txt")?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let g6 = line?;
        let g = graph::Graph::from_g6(&g6)?;
        let treewidth = treewidth::rec::treewidth(&g);
        count += 1;
        println!("Processed {} graphs", count);

        if treewidth != 4 {
            panic!("Expected treewidth 4, got {}", treewidth);
        }

        if count >= 100 {
            break;
        }
    }


    Ok(())
}

// use viuer::{Config, print_from_file};
// use tempfile::NamedTempFile;
// use std::io::Write;
// use std::process::Command;
//
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // 1. Create a simple DOT graph
//     let dot = r#"
//         graph G {
//           graph [dpi=300];
//           node [shape=circle];
//           A -- B;
//           A -- C;
//           B -- C;
//           B -- D;
//           C -- E;
//           // give Graphviz a little hint for layout
//           { rank=same; A; C; }
//         }
//     "#;
//
//     // 2. Create a temporary file for the PNG
//     let tmp_file = NamedTempFile::new()?.into_temp_path();
//     let png_path = tmp_file.with_extension("png");
//
//     // 3. Render the DOT graph to PNG using Graphviz
//     let mut dot_file = NamedTempFile::new()?;
//     write!(dot_file, "{}", dot)?;
//     dot_file.flush()?;
//
//     let output = Command::new("dot")
//         .args(&["-Tpng", dot_file.path().to_str().unwrap(), "-o", png_path.to_str().unwrap()])
//         .output()?;
//
//     if !output.status.success() {
//         eprintln!("Graphviz error: {}", String::from_utf8_lossy(&output.stderr));
//         return Ok(());
//     }
//
//     // 4. Show the PNG in terminal using viuer
//     let config = Config {
//         transparent: true,
//         ..Default::default()
//     };
//     print_from_file(png_path.to_str().unwrap(), &config)?;
//
//     Ok(())
// }
