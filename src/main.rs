use clap::Parser;

mod cli;
mod timeout;

#[cfg(feature = "measure-memory")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::Cli::parse().run()
}
