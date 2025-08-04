// Internal crates.
use crate::{
    cli::FindingConfig,
    error::{DeepFinderError, SystemError},
    system::{self, VirtualFile, build_virtual_files},
};

// External crates.
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Write,
    thread::{self, JoinHandle},
};

#[derive(Eq, PartialEq)]
pub struct DuplicateFile {
    pub paths: HashSet<String>,
    pub name: String,
    pub checksums: Option<HashMap<String, String>>,
    pub nb_occurrences: usize,
    pub size: u64,
}

/// This function is the scheduler for the search engine.
///
/// # Arguments
///
/// * `config` - The FindingConfig struct with the user's configuration.
///
/// # Returns
///
/// The result of the search engine scheduler, DeepFinderError otherwise.
///
pub fn search_engine_scheduler(config: &FindingConfig) -> Result<(), DeepFinderError> {
    let file_paths: Vec<String> = search_files(&config.search_path, config.include_hidden_files).map_err(DeepFinderError::SystemError)?;
    let mut virtual_files: Vec<VirtualFile> = build_virtual_files(&file_paths);
    
    if let Some(hash_algorithms) = &config.hash {
        hash_handler(hash_algorithms, &mut virtual_files)?;
    }

    let duplicates: Vec<DuplicateFile> = search_eventual_duplicates(&virtual_files, config);

    // Save the results in a temp file (WIP).
    let mut file: File = File::create_new("deepfinder_results.txt").unwrap();
    for d in &duplicates {
        writeln!(file, "Name: {}, Size: {}, Paths: {:?}, Occurrences: {}", d.name, d.size, d.paths, d.nb_occurrences).unwrap();
        if let Some(checksums) = &d.checksums {
            writeln!(file, "Checksums: {checksums:?}").unwrap();
        }
    }

    println!("\n{} files", virtual_files.len());
    Ok(())
}

/// This function is responsible for searching files in a directory.
///
/// # Arguments
///
/// * `dir` - A string slice that holds the directory to search.
/// * `include_hidden_files` - A boolean that indicates if hidden files should be included in the search or not.
///
/// # Returns
///
/// A vector of strings with the files found in the directory, SystemError otherwise.
///
pub fn search_files(dir: &str, include_hidden_files: bool) -> Result<Vec<String>, SystemError> {
    let mut paths: Vec<_> = fs::read_dir(dir)
        .map_err(|e| SystemError::UnableToReadDir(e.to_string()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            if !include_hidden_files {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    return !name.starts_with('.');
                }
            }
            true
        })
        .collect();
    let mut files: Vec<String> = Vec::new();
    let mut index: usize = paths.len();

    paths.sort_by(|a, b| {
        let aname: &str = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let bname: &str = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        aname.cmp(bname)
    });

    for path in &paths {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        index -= 1;

        if index == 0 {
            if path.is_dir() {
                files.extend(search_files(&format!("{dir}/{name}"), include_hidden_files)?);
            }
        } else if path.is_dir() {
            files.extend(search_files(&format!("{dir}/{name}"), include_hidden_files)?);
        }
    }

    // Add the files to the vector.
    files.extend(paths.iter().filter_map(|p| {
        if p.is_file() {
            p.to_str().map(ToString::to_string)
        } else {
            None
        }
    }));

    Ok(files)
}

/// This function is responsible for scheduling the hashing of files and the different threads.
///
/// # Arguments
///
/// * `hash_algorithms` - A string slice that holds the directory to search.
/// * `virtual_files` - A boolean that indicates if hidden files should be included in the search or not.
///
/// # Returns
///
/// A vector of strings with the files found in the directory, SystemError otherwise.
///
fn hash_handler(hash_algorithms: &[String], virtual_files: &mut Vec<VirtualFile>) -> Result<(), DeepFinderError> {
    let num_cores: usize = num_cpus::get(); // Get the number of logical cores.
    let chunk_size: usize = virtual_files.len().div_ceil(num_cores);
    let mut updated_files: Vec<VirtualFile> = Vec::new();

    for hash_algorithm in hash_algorithms {
        let mut threads: Vec<JoinHandle<Vec<VirtualFile>>>= Vec::new();
        for i in 0..num_cores {
            let hash_algorithm: String = hash_algorithm.clone();
            let start: usize = i * chunk_size;
            let end: usize = ((i + 1) * chunk_size).min(virtual_files.len());

            // We can break the loop because there are no more files to process.
            if start >= end {
                break;
            }

            let mut chunk_files: Vec<VirtualFile> = virtual_files[start..end].to_vec();
            threads.push(thread::spawn(move || {
                for file in &mut chunk_files {
                    if let Some(hash) = system::manage_hash(&file.full_path, &hash_algorithm) {
                        file.update_checksum(&hash_algorithm, hash);
                    }
                }
                chunk_files // Return the processed chunk.
            }));
        }

        for thread in threads {
            let chunk: Vec<VirtualFile> = thread.join().map_err(|_| DeepFinderError::SystemError(SystemError::ThreadError))?;
            for file in chunk {
                if let Some(existing_file) = updated_files.iter_mut().find(|f| f.full_path == file.full_path) {
                    // If the file already exists, update its checksums.
                    if let Some(checksums) = &file.checksums {
                        existing_file.update_checksum(hash_algorithm, checksums.get(hash_algorithm.as_str()).cloned().unwrap_or_default());
                    }
                } else {
                    // Otherwise, add the new file to the updated files.
                    updated_files.push(file);
                }
            }
        }
    }

    *virtual_files = updated_files;
    Ok(())
}

/// This function is responsible for searching eventual duplicates in the virtual files.
///
/// # Arguments
///
/// * `virtual_files` - A slice of VirtualFile.
/// * `config` - A reference to the FindingConfig struct with the user's configuration.
///
/// # Returns
///
/// A vector of DuplicateFile containing the duplicates found.
///
fn search_eventual_duplicates(virtual_files: &[VirtualFile], config: &FindingConfig) -> Vec<DuplicateFile> {
    let mut map: HashMap<String, DuplicateFile> = HashMap::new();

    for file in virtual_files {
        let key: String = if config.enable_search_by_name {
            file.name.clone()
        } else if let Some(checksums) = &file.checksums {
            checksums.iter().map(|(k, v)| format!("{k}:{v}")).collect::<Vec<_>>().join("|")
        } else {
            continue;
        };

        let entry: &mut DuplicateFile = map.entry(key).or_insert_with(|| DuplicateFile {
            paths: HashSet::new(),
            name: file.name.clone(),
            checksums: file.checksums.clone(),
            nb_occurrences: 0,
            size: file.size,
        });

        entry.paths.insert(file.full_path.clone());
        entry.nb_occurrences += 1;
    }

    map.into_values().collect()
}