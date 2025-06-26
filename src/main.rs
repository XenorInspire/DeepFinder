// Prevents the use of unsafe code
#![forbid(unsafe_code)]

// Internal crates.
use cli::FindingConfig;
use error::DeepFinderError;
mod cli;
mod error;
mod search_engine;
mod system;

// External crates.
use std::process;

/// This function is the "entry point" of the program.
///
fn main() {
    if let Err(e) = run_search() {
        eprintln!("{e}");
        process::exit(1);
    }

    process::exit(0);
}

/// This function runs the search engine and returns the result.
/// It permits the program to return an error if the search engine fails.
/// 
fn run_search() -> Result<(), DeepFinderError> {
    let config: FindingConfig = cli::run()?;
    search_engine::search_engine_scheduler(&config)?;
    Ok(())
}
