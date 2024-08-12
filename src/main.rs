// Prevents the use of unsafe code
#![forbid(unsafe_code)]

// External crates.
use std::{env, fs, io};

struct Counts {
    dirs: i32,
    files: i32,
}

fn walk(dir: &str, prefix: &str, counts: &mut Counts) -> io::Result<()> {
    let mut paths: Vec<_> = fs::read_dir(dir)?
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut index: usize = paths.len();

    paths.sort_by(|a, b| {
        let aname: &str = a.file_name().unwrap().to_str().unwrap();
        let bname: &str = b.file_name().unwrap().to_str().unwrap();
        aname.cmp(bname)
    });

    for path in paths {
        let name: &str = path.file_name().unwrap().to_str().unwrap();
        println!("{}", path.display());
        index -= 1;

        if name.starts_with(".") {
            continue;
        }

        if path.is_dir() {
            counts.dirs += 1;
        } else {
            counts.files += 1;
        }

        if index == 0 {
            // println!("{}└── {}", prefix, name);
            if path.is_dir() {
                walk(
                    &format!("{}/{}", dir, name),
                    &format!("{}    ", prefix),
                    counts,
                )?;
            }
        } else {
            // println!("{}├── {}", prefix, name);
            if path.is_dir() {
                walk(
                    &format!("{}/{}", dir, name),
                    &format!("{}│   ", prefix),
                    counts,
                )?;
            }
        }
    }

    Ok(())
}

/// This function is the "entry point" of the program.
///
fn main() -> io::Result<()> {
    let dir: String = env::args().nth(1).unwrap_or(".".to_string());
    println!("{}", dir);

    let mut counts: Counts = Counts { dirs: 0, files: 0 };
    walk(&dir, "", &mut counts)?;

    println!("\n{} directories, {} files", counts.dirs, counts.files);

    Ok(())
}
