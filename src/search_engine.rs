// External crates.
use std::{fs, io};

pub struct Counter {
    pub dirs: i32,
    pub files: i32,
}

pub fn walk(dir: &str, counts: &mut Counter) -> io::Result<()> {
    let mut paths: Vec<_> = fs::read_dir(dir)?
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut index: usize = paths.len();

    paths.sort_by(|a, b| {
        let aname: &str = a.file_name().unwrap().to_str().unwrap();
        let bname: &str = b.file_name().unwrap().to_str().unwrap();
        aname.cmp(bname)
    });

    for path in paths.iter() {
        let name: &str = path.file_name().unwrap().to_str().unwrap();
        index -= 1;

        // Skip hidden files and directories, for a future option.
        // if name.starts_with(".") {
        //     continue;
        // }

        if path.is_dir() {
            counts.dirs += 1;
        } else {
            counts.files += 1;
        }

        if index == 0 {
            if path.is_dir() {
                walk(&format!("{}/{}", dir, name), counts)?;
            }
        } else {
            if path.is_dir() {
                walk(&format!("{}/{}", dir, name), counts)?;
            }
        }
    }
    println!("{:?}", paths);
    Ok(())
}