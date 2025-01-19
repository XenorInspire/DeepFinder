// Internal crates.
use crate::{
    cli::FindingConfig,
    error::{DeepFinderError, SystemError},
};

// External crates.
use std::fs;

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
    let files: Vec<String> = search_files(&config.path, config.include_hidden_files).map_err(DeepFinderError::SystemError)?;
    println!("\n{:?}, {} files", files, files.len());
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

    for path in paths.iter() {
        let name: &str = path.file_name().unwrap().to_str().unwrap();
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
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_string())
        } else {
            None
        }
    }));

    Ok(files)
}
