//! Runs treewidth computations in child processes so CLI commands can enforce timeouts. The child
//! process output is parsed from JSON and converted back into runtime and memory statistics.

use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use serde::Deserialize;
use thiserror::Error;

use crate::cli::approximate_treewidth::ApproxAlgorithmArg;
use crate::cli::benchmark::AlgorithmArg;
use crate::cli::compute_treewidth::ExactAlgorithmArg;
use crate::cli::heuristic_treewidth::HeuristicAlgorithmArg;

/// Errors that can occur while running a treewidth computation in a child process.
#[derive(Debug, Error)]
pub enum TreewidthProcessError {
    /// The child process exceeded the configured timeout.
    #[error("treewidth computation timed out after {timeout:?}")]
    Timeout {
        timeout: Duration,
    },

    /// The child process exited unsuccessfully.
    #[error("treewidth child process failed with status {status}; stderr: {stderr}")]
    ChildFailed {
        status: ExitStatus,
        stderr: String,
    },

    /// Any other error while managing the child process, such as spawning the process or parsing its output.
    #[error("treewidth process error: {message}")]
    Other {
        message: String,
    },
}

impl TreewidthProcessError {
    /// Creates a generic error with a message derived from error's Display implementation.
    pub fn other(error: impl std::fmt::Display) -> Self {
        Self::Other {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TreewidthProcessOutput {
    treewidth: usize,
    duration_ns: u64,
    #[serde(default)]
    allocated_bytes: Option<u64>,
    #[serde(default)]
    peak_bytes: Option<u64>,
}

/// Optional memory statistics reported as `(allocated_bytes, peak_bytes)`.
pub type MemoryStats = Option<(u64, u64)>;

/// Runs an exact, approximation, or heuristic treewidth computation in a child process with a
/// timeout. The bitset-based graph representation is used when `with_bitset` is true.
pub fn compute_or_approximate_treewidth(
    g6: &str,
    algorithm: AlgorithmArg,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration, MemoryStats), TreewidthProcessError> {
    match algorithm {
        AlgorithmArg::Exact(exact_alg) => compute_treewidth(g6, exact_alg, with_bitset, timeout),
        AlgorithmArg::Approx(approx_alg) => {
            approximate_treewidth(g6, approx_alg, with_bitset, timeout)
        }
        AlgorithmArg::Heuristic(heuristic_alg) => {
            heuristic_treewidth(g6, heuristic_alg, with_bitset, timeout)
        }
    }
}

/// Runs exact treewidth computation in a child process with a timeout.
pub fn compute_treewidth(
    g6: &str,
    algorithm: ExactAlgorithmArg,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration, MemoryStats), TreewidthProcessError> {
    run_command_timeout(
        "compute-treewidth",
        g6,
        &algorithm.to_string(),
        with_bitset,
        timeout,
    )
}

/// Runs approximate treewidth computation in a child process with a timeout.
pub fn approximate_treewidth(
    g6: &str,
    algorithm: ApproxAlgorithmArg,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration, MemoryStats), TreewidthProcessError> {
    run_command_timeout(
        "approximate-treewidth",
        g6,
        &algorithm.to_string(),
        with_bitset,
        timeout,
    )
}

/// Runs heuristic treewidth computation in a child process with a timeout.
pub fn heuristic_treewidth(
    g6: &str,
    algorithm: HeuristicAlgorithmArg,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration, MemoryStats), TreewidthProcessError> {
    run_command_timeout(
        "heuristic-treewidth",
        g6,
        &algorithm.to_string(),
        with_bitset,
        timeout,
    )
}

// Helper function to run a treewidth computation in a child process with a timeout and parse the
// JSON output.
fn run_command_timeout(
    command_name: &str,
    g6: &str,
    algorithm: &str,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration, MemoryStats), TreewidthProcessError> {
    let executable = std::env::current_exe().map_err(TreewidthProcessError::other)?;

    let mut command = Command::new(executable);

    command
        .arg(command_name)
        .arg("--json")
        .arg("--graph")
        .arg(g6)
        .arg("--algorithm")
        .arg(algorithm)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if with_bitset {
        command.arg("--bitset");
    }

    let mut child = command.spawn().map_err(TreewidthProcessError::other)?;

    let start = Instant::now();

    // Tries to wait for the child process to finish, checking periodically if the timeout has been
    // exceeded.
    loop {
        match child.try_wait().map_err(TreewidthProcessError::other)? {
            Some(status) => {
                let output = child
                    .wait_with_output()
                    .map_err(TreewidthProcessError::other)?;

                // If the child process exited with a non-zero status, it indicates an error.
                if !status.success() {
                    return Err(TreewidthProcessError::ChildFailed {
                        status,
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    });
                }

                // Parses the JSON output from the child process to extract the treewidth,
                // duration, and optional memory statistics.
                let stdout =
                    String::from_utf8(output.stdout).map_err(TreewidthProcessError::other)?;

                let parsed: TreewidthProcessOutput =
                    serde_json::from_str(&stdout).map_err(TreewidthProcessError::other)?;

                let memory_stats = match (parsed.allocated_bytes, parsed.peak_bytes) {
                    (Some(allocated_bytes), Some(peak_bytes)) => {
                        Some((allocated_bytes, peak_bytes))
                    }
                    _ => None,
                };

                return Ok((
                    parsed.treewidth,
                    Duration::from_nanos(parsed.duration_ns),
                    memory_stats,
                ));
            }

            None => {
                // If the child process is still running, checks if the timeout has been exceeded.
                // If so, it kills the child process and returns a timeout error. Otherwise, it
                // sleeps for a short duration before checking again.
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();

                    return Err(TreewidthProcessError::Timeout { timeout });
                }

                let remaining = timeout
                    .checked_sub(start.elapsed())
                    .unwrap_or(Duration::ZERO);

                std::thread::sleep(remaining.min(Duration::from_millis(50)));
            }
        }
    }
}
