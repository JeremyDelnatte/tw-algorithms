use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use thiserror::Error;
use serde::Deserialize;
use tw_algorithms::treewidth::Algorithm;

use crate::cli::approximate_treewidth::ApproxAlgorithmArg;
use crate::cli::benchmark::AlgorithmArg;
use crate::cli::compute_treewidth::ExactAlgorithmArg;

#[derive(Debug, Error)]
pub enum TreewidthProcessError {
    #[error("treewidth computation timed out after {timeout:?}")]
    Timeout {
        timeout: Duration,
    },

    #[error("treewidth child process failed with status {status}; stderr: {stderr}")]
    ChildFailed {
        status: ExitStatus,
        stderr: String,
    },

    #[error("treewidth process error: {message}")]
    Other {
        message: String,
    },
}

impl TreewidthProcessError {
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
}

pub fn compute_or_approximate_treewidth(
    g6: &str,
    algorithm: AlgorithmArg,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration), TreewidthProcessError> {
    match algorithm {
        AlgorithmArg::Exact(exact_alg) => compute_treewidth(g6, exact_alg, with_bitset, timeout),
        AlgorithmArg::Approx(approx_alg) => approximate_treewidth(g6, approx_alg, with_bitset, timeout),
    }
}

pub fn compute_treewidth(
    g6: &str,
    algorithm: ExactAlgorithmArg,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration), TreewidthProcessError> {
    run_command_timeout("compute-treewidth", g6, &algorithm.to_string(), with_bitset, timeout)
}

pub fn approximate_treewidth(
    g6: &str,
    algorithm: ApproxAlgorithmArg,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration), TreewidthProcessError> {
    run_command_timeout("approximate-treewidth", g6, &algorithm.to_string(), with_bitset, timeout)
}

fn run_command_timeout(
    command_name: &str,
    g6: &str,
    algorithm: &str,
    with_bitset: bool,
    timeout: Duration,
) -> Result<(usize, Duration), TreewidthProcessError> {
    let executable = std::env::current_exe()
        .map_err(TreewidthProcessError::other)?;

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

    let mut child = command
        .spawn()
        .map_err(TreewidthProcessError::other)?;

    let start = Instant::now();

    loop {
        match child.try_wait().map_err(TreewidthProcessError::other)? {
            Some(status) => {
                let output = child
                    .wait_with_output()
                    .map_err(TreewidthProcessError::other)?;

                if !status.success() {
                    return Err(TreewidthProcessError::ChildFailed {
                        status,
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    });
                }

                let stdout = String::from_utf8(output.stdout)
                    .map_err(TreewidthProcessError::other)?;

                let parsed: TreewidthProcessOutput = serde_json::from_str(&stdout)
                    .map_err(TreewidthProcessError::other)?;

                return Ok((
                    parsed.treewidth,
                    Duration::from_nanos(parsed.duration_ns),
                ));
            }

            None => {
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
