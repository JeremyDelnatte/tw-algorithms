//! Utilities for displaying benchmark progress bars. The progress bars track graph-level progress
//! and optional per-graph iteration progress.

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

/// Progress bars used while running benchmark scenarios.
pub struct BenchmarkProgressBars {
    graph_pb: ProgressBar,
    iteration_pb: Option<ProgressBar>,
}

/// Creates progress bars for graph-level progress and optional per-graph iterations.
pub fn create_benchmark_progress_bars(
    show_progress: bool,
    num_graphs: usize,
    num_iterations: Option<usize>,
) -> Result<BenchmarkProgressBars, Box<dyn std::error::Error>> {
    let multi = MultiProgress::new();
    multi.set_draw_target(ProgressDrawTarget::stdout());

    if !show_progress {
        return Ok(BenchmarkProgressBars {
            graph_pb: ProgressBar::hidden(),
            iteration_pb: None,
        });
    }

    let graph_pb = multi.add(ProgressBar::new(num_graphs as u64));
    graph_pb.set_prefix("Graphs");
    graph_pb.set_style(
        ProgressStyle::with_template(
            "{prefix:.green}\n\
             {spinner:.green} {msg:20} {bar:32.red/black} {percent:>3}% • {pos}/{len} • {eta_precise}",
        )?
        .progress_chars("━╾ "),
    );

    let iteration_pb = if let Some(num_iterations) = num_iterations {
        let pb = multi.add(ProgressBar::new(num_iterations as u64));
        pb.set_prefix("Iterations");
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.cyan} {prefix:20} {bar:32.red/black} {percent:>3}% • {pos}/{len} • {eta_precise}",
            )?
            .progress_chars("━╾ "),
        );

        Some(pb)
    } else {
        None
    };

    Ok(BenchmarkProgressBars {
        graph_pb,
        iteration_pb,
    })
}

/// Starts progress reporting for a graph.
pub fn start_graph_progress(
    progress: &BenchmarkProgressBars,
    graph_label: &str,
    num_iterations: Option<usize>,
) {
    let mut chars = graph_label.chars();

    let truncated: String = chars.by_ref().take(17).collect();

    let graph_label = if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    };

    progress.graph_pb.set_message(graph_label);

    if let (Some(iteration_pb), Some(num_iterations)) = (&progress.iteration_pb, num_iterations) {
        iteration_pb.reset();
        iteration_pb.set_length(num_iterations as u64);
    }
}

/// Increments the per-graph iteration progress bar.
pub fn inc_iteration_progress(progress: &BenchmarkProgressBars) {
    if let Some(iteration_pb) = &progress.iteration_pb {
        iteration_pb.inc(1);
    }
}

/// Marks the current graph as finished.
pub fn finish_graph_progress(progress: &BenchmarkProgressBars) {
    progress.graph_pb.inc(1);
}

/// Finishes all benchmark progress bars.
pub fn finish_benchmark_progress(progress: &BenchmarkProgressBars) {
    progress.graph_pb.finish_with_message("done");

    if let Some(iteration_pb) = &progress.iteration_pb {
        iteration_pb.finish_and_clear();
    }
}
