//! Command-line entry point.

use clap::Parser;

mod cli;
mod timeout;

// The measure-memory feature needs to use a dedicated allocator to track memory usage.
// NOTE: This has an impact on performance, so it should only be enabled when needed.
#[cfg(feature = "measure-memory")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::Cli::parse().run()
}
