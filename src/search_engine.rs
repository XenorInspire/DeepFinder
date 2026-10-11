// Internal crates.
use crate::{
    cli::FindingConfig,
    error::{DeepFinderError, SystemError},
    export::export_findings_scheduler,
    system::{self, VirtualFile, build_virtual_files},
};

// External crates.
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
    thread::{self, JoinHandle},
};

#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct DuplicateFile {
    pub paths: HashSet<String>,
    pub name: String,
    pub checksums: Option<HashMap<String, String>>,
    pub size: u64,
}

#[derive(Eq, Hash, PartialEq)]
enum DuplicateKey<'a> {
    Name(&'a str),
    Checksums(Vec<&'a str>),
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
        if !config.enable_search_by_name {
            retain_hash_candidates_by_size(&mut virtual_files);
        }
        hash_handler(hash_algorithms, &mut virtual_files)?;
    }

    let duplicates: Vec<DuplicateFile> = search_eventual_duplicates(&virtual_files, config);
    export_findings_scheduler(&duplicates, config)

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
            if !include_hidden_files
                && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    return !name.starts_with('.');
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
/// * `hash_algorithms` - The algorithms to use when hashing each file.
/// * `virtual_files` - The files to hash, updated in place with their checksums.
///
/// # Returns
///
/// A vector of strings with the files found in the directory, SystemError otherwise.
///
fn hash_handler(hash_algorithms: &[String], virtual_files: &mut Vec<VirtualFile>) -> Result<(), DeepFinderError> {
    if virtual_files.is_empty() {
        return Ok(());
    }

    let file_count: usize = virtual_files.len();
    let worker_count: usize = num_cpus::get().min(file_count);
    let chunk_size: usize = file_count.div_ceil(worker_count);
    let hash_algorithms: Arc<Vec<String>> = Arc::new(hash_algorithms.to_vec());
    let mut threads: Vec<JoinHandle<Vec<VirtualFile>>> = Vec::new();
    let mut files = std::mem::take(virtual_files).into_iter();

    loop {
        let mut chunk_files: Vec<VirtualFile> = files.by_ref().take(chunk_size).collect();
        if chunk_files.is_empty() {
            break;
        }
        let hash_algorithms: Arc<Vec<String>> = Arc::clone(&hash_algorithms);
        threads.push(thread::spawn(move || {
            for file in &mut chunk_files {
                for hash_algorithm in hash_algorithms.iter() {
                    if let Some(hash) = system::manage_hash(&file.full_path, hash_algorithm) {
                        file.update_checksum(hash_algorithm, hash);
                    }
                }
            }
            chunk_files
        }));
    }

    let mut updated_files: Vec<VirtualFile> = Vec::with_capacity(file_count);
    for thread in threads {
        updated_files.extend(
            thread
                .join()
                .map_err(|_| DeepFinderError::SystemError(SystemError::ThreadError))?,
        );
    }

    *virtual_files = updated_files;
    Ok(())
}

/// This function is responsible for filtering the virtual files by size, keeping only those with more than one occurrence.
/// It's been used especially when name-based search is disabled, to reduce the number of files to hash and compare.
///
/// # Arguments
///
/// * `virtual_files` - A mutable reference to the vector containing the virtual files.
///
fn retain_hash_candidates_by_size(virtual_files: &mut Vec<VirtualFile>) {
    let mut size_counts: HashMap<u64, usize> = HashMap::with_capacity(virtual_files.len());
    for file in virtual_files.iter() {
        *size_counts.entry(file.size).or_default() += 1;
    }
    virtual_files.retain(|file| size_counts.get(&file.size).is_some_and(|count| *count > 1));
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
    let mut map: HashMap<DuplicateKey<'_>, DuplicateFile> = HashMap::with_capacity(virtual_files.len());

    for file in virtual_files {
        let key = if config.enable_search_by_name {
            DuplicateKey::Name(&file.name)
        } else if let (Some(hash_algorithms), Some(checksums)) =
            (&config.hash, &file.checksums)
        {
            let Some(ordered_checksums) = hash_algorithms
                .iter()
                .map(|algorithm| checksums.get(algorithm).map(String::as_str))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            DuplicateKey::Checksums(ordered_checksums)
        } else {
            continue;
        };

        let entry: &mut DuplicateFile = map.entry(key).or_insert_with(|| DuplicateFile {
            paths: HashSet::new(),
            name: file.name.clone(),
            checksums: file.checksums.clone(),
            size: file.size,
        });

        entry.paths.insert(file.full_path.clone());
    }

    // Filter out entries with only one occurrence.
    map.retain(|_, v| v.paths.len() > 1);
    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::cli::CliOutput;

    #[test]
    fn test_search_eventual_duplicates_by_name() {
        let files = vec![
            VirtualFile {
                name: "file1.txt".to_string(),
                full_path: "/tmp/file1.txt".to_string(),
                size: 123,
                checksums: None,
            },
            VirtualFile {
                name: "file1.txt".to_string(),
                full_path: "/tmp/copy_file1.txt".to_string(),
                size: 123,
                checksums: None,
            },
            VirtualFile {
                name: "file2.txt".to_string(),
                full_path: "/tmp/file2.txt".to_string(),
                size: 456,
                checksums: None,
            },
        ];

        let config: FindingConfig = FindingConfig {
            search_path: "/tmp".to_string(),
            enable_search_by_name: true,
            include_hidden_files: false,
            include_hashes_in_output: false,
            hash: None,
            output: CliOutput::Standard,
        };

        let duplicates: Vec<DuplicateFile> = search_eventual_duplicates(&files, &config);
        assert_eq!(duplicates.len(), 1);
        
        let duplicate: &DuplicateFile = &duplicates[0];
        assert_eq!(duplicate.name, "file1.txt");
        assert_eq!(duplicate.paths.len(), 2);
        assert!(duplicate.paths.contains("/tmp/file1.txt"));
        assert!(duplicate.paths.contains("/tmp/copy_file1.txt"));
    }

    #[test]
    fn test_search_eventual_duplicates_by_checksum() {
        let mut files: Vec<VirtualFile> = Vec::new();
        let mut checksums: HashMap<String, String> = HashMap::new();
        checksums.insert("md5".to_string(), "abc123".to_string());
        checksums.insert("sha256".to_string(), "def456".to_string());

        files.push(VirtualFile {
            name: "fileA.txt".to_string(),
            full_path: "/tmp/fileA.txt".to_string(),
            size: 100,
            checksums: Some(checksums.clone()),
        });
        let mut reordered_checksums: HashMap<String, String> = HashMap::new();
        reordered_checksums.insert("sha256".to_string(), "def456".to_string());
        reordered_checksums.insert("md5".to_string(), "abc123".to_string());
        files.push(VirtualFile {
            name: "fileB.txt".to_string(),
            full_path: "/tmp/fileB.txt".to_string(),
            size: 100,
            checksums: Some(reordered_checksums),
        });

        let config: FindingConfig = FindingConfig {
            search_path: "/tmp".to_string(),
            enable_search_by_name: false,
            include_hidden_files: false,
            include_hashes_in_output: false,
            hash: Some(vec!["md5".to_string(), "sha256".to_string()]),
            output: CliOutput::Standard,
        };

        let duplicates: Vec<DuplicateFile> = search_eventual_duplicates(&files, &config);
        assert_eq!(duplicates.len(), 1);
        
        let duplicate: &DuplicateFile = &duplicates[0];
        assert_eq!(duplicate.paths.len(), 2);
        assert!(duplicate.paths.contains("/tmp/fileA.txt"));
        assert!(duplicate.paths.contains("/tmp/fileB.txt"));
    }

    #[test]
    fn test_retain_hash_candidates_by_size() {
        let mut files = vec![
            VirtualFile {
                name: "unique.txt".to_string(),
                full_path: "/tmp/unique.txt".to_string(),
                size: 1,
                checksums: None,
            },
            VirtualFile {
                name: "first.txt".to_string(),
                full_path: "/tmp/first.txt".to_string(),
                size: 2,
                checksums: None,
            },
            VirtualFile {
                name: "second.txt".to_string(),
                full_path: "/tmp/second.txt".to_string(),
                size: 2,
                checksums: None,
            },
        ];

        retain_hash_candidates_by_size(&mut files);

        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|file| file.size == 2));
    }
}