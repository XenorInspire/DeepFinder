// Internal crates.
use crate::{
    cli::FindingConfig,
    error::{DeepFinderError, SystemError},
    system::{self, build_virtual_files, VirtualFile},
};

// External crates.
use std::{fs, thread::JoinHandle};

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
    let file_paths: Vec<String> = search_files(&config.path, config.include_hidden_files).map_err(DeepFinderError::SystemError)?;
    let mut virtual_files: Vec<VirtualFile> = build_virtual_files(&file_paths);
    println!("\n{:?} - {} files", virtual_files[0], virtual_files.len());

    if let Some(hash_algorithms) = &config.hash {
        hash_handler(hash_algorithms, &mut virtual_files)?;
    }

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
                files.extend(search_files(&format!("{}/{}", dir, name), include_hidden_files)?);
            }
        } else if path.is_dir() {
            files.extend(search_files(&format!("{}/{}", dir, name), include_hidden_files)?);
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

/// <!> ---------- Fix this function by using ArcMutex slice of virtual_files array. ---------- <!>
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
fn hash_handler(hash_algorithms: &[String], virtual_files: &mut [VirtualFile]) -> Result<(), DeepFinderError> {
    let num_cores: usize = num_cpus::get(); // Get the number of logical cores.
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    let nb_of_files_per_thread: usize = virtual_files.len() / num_cores;
    let nb_of_passwd_last_thread: usize = nb_of_files_per_thread + virtual_files.len() % num_cores;

    for hash_algorithm in hash_algorithms {
        for i in 0..num_cores {
            let idx_start: usize = i * nb_of_files_per_thread;
            let idx_end: usize = if i == num_cores - 1 {
                idx_start + nb_of_passwd_last_thread
            } else {
                idx_start + nb_of_files_per_thread
            };

            let mut files_to_hash: Vec<VirtualFile> = virtual_files.to_vec();
            let hash_algorithm: String = hash_algorithm.clone();

            threads.push(std::thread::spawn(move || {
                calculate_hash(&hash_algorithm, &mut files_to_hash, idx_start, idx_end);
            }));
        }
    }

    for thread in threads {
        thread.join().map_err(|_| DeepFinderError::SystemError(SystemError::ThreadError))?;
    }

    Ok(())
}

/// <!> ---------- Delete index to parse the entire slice ---------- <!>
/// This function is responsible for calculating the hash of files.
///
/// # Arguments
/// 
/// * `hash_algorithm` - A string slice that holds the hash algorithm to use.
/// * `files_to_hash` - A mutable reference to a slice of VirtualFile.
/// * `idx_start` - The starting index of the files to hash.
/// * `idx_end` - The ending index of the files to hash.
/// 
/// # Returns
/// 
/// A vector of strings with the files found in the directory, SystemError otherwise.
/// 
fn calculate_hash(hash_algorithm: &str, files_to_hash: &mut [VirtualFile], idx_start: usize, idx_end: usize) {
    for file in files_to_hash.iter_mut().take(idx_end + 1).skip(idx_start) {
        if let Some(hash) = system::manage_hash(&file.full_path, hash_algorithm) {
            println!("Hashing {} with {}...", hash, hash_algorithm);
            file.update_checksum(hash_algorithm, hash);
        }
    }
}