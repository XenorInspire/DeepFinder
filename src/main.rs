// Prevents the use of unsafe code
#![forbid(unsafe_code)]

// External crates.
use std::{io, process::exit};

// Internal crates.
mod cli;
mod error;
mod search_engine;
mod system;

/// This function is the "entry point" of the program.
///
fn main() -> io::Result<()> {
    match cli::run() {
        Ok(config) => {
            search_engine::search_engine_scheduler(config).unwrap();
            exit(0);
        }
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    }
}
