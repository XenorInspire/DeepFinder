// Prevents the use of unsafe code
#![forbid(unsafe_code)]

// External crates.
use std::{env, io};

// Internal crates.
mod cli;
mod search_engine;
use search_engine::Counter;

/// This function is the "entry point" of the program.
///
fn main() -> io::Result<()> {
    match cli::run() {
        Ok(_) => {
            let dir: String = env::args().nth(1).unwrap_or(".".to_string());
            let mut counts: Counter = Counter { dirs: 0, files: 0 };
            search_engine::walk(&dir, &mut counts)?;
            println!("\n{} directories, {} files", counts.dirs, counts.files);
            std::process::exit(0);
        }
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    }
}
