// Internal crates.
use crate::error::SystemError;

// External crates.
use blake2::{Blake2b512, Blake2s256};
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use std::{collections::HashMap, fs::{self, File}, io::{BufReader, Read}, path::{Path, PathBuf}};
use whirlpool::Whirlpool;

#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_family = "windows")]
use std::os::windows::fs::MetadataExt;

/// This struct represents a virtual file on the system.
/// It permits the program to store the file's name, size, full path and checksum properly.
///
#[derive(Debug)]
pub struct VirtualFile {
    pub name: String,
    pub size: u64,
    pub full_path: String,
    pub checksum: Option<HashMap<String, String>>,
}

/// This function is responsible for checking a path/filename.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path/filename to check.
///
/// # Returns
///
/// Ok(String) if the path/filename is valid, containing the full path, SystemError otherwise.
///
pub fn is_valid_file_path(path: &str) -> Result<String, SystemError> {
    let filename: String = match Path::new(path).file_name() {
        Some(f) => match f.to_str() {
            Some(f) => f.to_string(),
            None => return Err(SystemError::InvalidPath(path.to_string())),
        },
        None => return Err(SystemError::InvalidPath(path.to_string())),
    };

    let invalid_chars: &[char] = get_invalid_chars();
    if filename.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(SystemError::InvalidFilename(filename.to_string()));
    }

    let full_path: String = build_full_path(path)?;

    #[cfg(target_family = "windows")]
    if full_path.len() > 260 {
        return Err(SystemError::PathTooLong(path.to_string()));
    }

    if !check_if_parent_folder_exists(&full_path) {
        return Err(SystemError::ParentFolderDoesntExist(path.to_string()));
    }
    Ok(full_path)
}

/// This function is responsible for checking if the parent folder exists from a given file path.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the file path.
///
/// # Returns
///
/// True if the parent folder exists, false otherwise.
///
pub fn check_if_parent_folder_exists(file_path: &str) -> bool {
    match Path::new(file_path).parent() {
        Some(p) => p.exists(),
        None => false,
    }
}

/// This function is responsible for checking a folder path.
/// This function is used to check the directory where the duplicate files are supposed to be saved.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to check.
///
/// # Returns
///
/// Ok(String) if the path is valid, containing the full path, SystemError otherwise.
///
pub fn is_valid_folder_path(path: &str) -> Result<String, SystemError> {
    let full_path: String = build_full_path(path)?;
    if !Path::new(&full_path).exists() {
        return Err(SystemError::InvalidFolder(full_path));
    }

    Ok(full_path)
}

/// This function is responsible for managing file hashing.
/// It returns the checksum of the file, using the hash algorithm provided.
/// If the hash algorithm is not supported, it returns an error.
///
/// # Arguments
///
/// * `file` - The password to hash.
/// * `hash` - The hash algorithm to use.
///
/// # Returns
///
/// The hashed file, SystemError otherwise.
///
pub fn manage_hash(file: &str, hash: &str) -> Result<String, SystemError> {
    match hash {
        "md5" => Ok(hash_with_digest(Md5::new(), file)),
        "sha1" => Ok(hash_with_digest(Sha1::new(), file)),
        "sha224" => Ok(hash_with_digest(Sha224::new(), file)),
        "sha256" => Ok(hash_with_digest(Sha256::new(), file)),
        "sha384" => Ok(hash_with_digest(Sha384::new(), file)),
        "sha512" => Ok(hash_with_digest(Sha512::new(), file)),
        "sha3-224" => Ok(hash_with_digest(Sha3_224::new(), file)),
        "sha3-256" => Ok(hash_with_digest(Sha3_256::new(), file)),
        "sha3-384" => Ok(hash_with_digest(Sha3_384::new(), file)),
        "sha3-512" => Ok(hash_with_digest(Sha3_512::new(), file)),
        "blake2b-512" => Ok(hash_with_digest(Blake2b512::new(), file)),
        "blake2s-256" => Ok(hash_with_digest(Blake2s256::new(), file)),
        "whirlpool" => Ok(hash_with_digest(Whirlpool::new(), file)),
        _ => Err(SystemError::UnsupportedHashAlgorithm(hash.to_string())),
    }
}

/// This function is responsible for calculating file hash with a specified algorithm.
/// It returns the file hash.
///
/// # Arguments
///
/// * `hasher` - The hasher to use, it must implement the Digest trait.
/// * `file` - The file to hash.
///
/// # Returns
///
/// The hashed file.
///
fn hash_with_digest<D: Digest>(mut hasher: D, path: &str) -> String {
    let input = File::open(path).unwrap();
    let mut reader = BufReader::new(input);

    let digest = {
        let mut buffer = [0; 1024];
        loop {
            let count = reader.read(&mut buffer).unwrap();
            if count == 0 { break }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };

    hex::encode(digest)
}

/// This function sends the invalid chars for windows platforms.
///
/// # Returns
///
/// '<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n' chars.
///
#[cfg(target_family = "windows")]
fn get_invalid_chars() -> &'static [char] {
    &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n',]
}

/// This function sends the invalid chars for unix platforms.
///
/// # Returns
///
/// '/', '\0', '\r', '\n' chars.
///
#[cfg(target_family = "unix")]
fn get_invalid_chars() -> &'static [char] {
    &['/', '\0', '\r', '\n']
}

/// This function is reponsible for building the entire path of a file/folder.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to build.
///
/// Returns
///
/// A string containing the full path. DeepFinderError if the path is invalid or it can't get the current directory.
///
fn build_full_path(path: &str) -> Result<String, SystemError> {
    let full_path: String = if !Path::new(path).is_absolute() {
        let current_dir: String = match std::env::current_dir() {
            Ok(c) => match c.to_str() {
                Some(s) => s.to_string(),
                None => return Err(SystemError::InvalidPath(path.to_string())),
            },
            Err(e) => {
                return Err(SystemError::UnableToGetCurrentDir(e.to_string()));
            }
        };
        current_dir + "/" + path.trim_start_matches("./")
    } else {
        path.to_string()
    };

    Ok(full_path)
}

/// This function is responsible for building virtual files from a list of file paths.
///
/// Arguments
///
/// * `file_paths` - A vector of strings that holds the file paths.
///
/// Returns
///
/// A vector of VirtualFile structs. Empty if no paths are provided.
///
pub fn build_virtual_files(file_paths: Vec<String>) -> Vec<VirtualFile> {
    let mut virtual_files: Vec<VirtualFile> = Vec::new();
    for path in file_paths.iter() {
        let file: VirtualFile = VirtualFile {
            name: match Path::new(path).file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            },
            size: get_file_size(path),
            full_path: path.to_string(),
            checksum: None,
        };
        virtual_files.push(file);
    }
    virtual_files
}

/// This function is responsible for getting the size of a file.
/// It uses the metadata from the file to get the size in bytes.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the file path.
///
/// # Returns
///
/// The size of the file in bytes. 0 if the file doesn't exist or can't get the metadata.
///
fn get_file_size(file_path: &str) -> u64 {
    let metadata = match fs::metadata(file_path) {
        Ok(m) => m,
        Err(_) => return 0,
    };

    #[cfg(target_family = "windows")]
    return metadata.file_size();

    #[cfg(target_family = "unix")]
    return metadata.size();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, path::PathBuf};

    #[test]
    fn test_is_valid_file_path() {
        let relative_path: &str = "./test.txt";
        let invalid_path: &str = "test.txt\0";

        #[cfg(target_family = "windows")]
        let absolute_path: &str = "C:/Users/test.txt";

        #[cfg(target_family = "unix")]
        let absolute_path: &str = "/home/test.txt";

        assert!(is_valid_file_path(relative_path).is_ok());
        assert!(is_valid_file_path(absolute_path).is_ok());
        assert!(is_valid_file_path(invalid_path).is_err());
    }

    #[test]
    fn test_is_valid_folder() {
        let relative_folder_path: &str = "./";
        let invalid_folder_path: &str = "./test1/test2/";

        #[cfg(target_family = "windows")]
        let absolute_folder_path: &str = "C:/Users/";

        #[cfg(target_family = "unix")]
        let absolute_folder_path: &str = "/home/";

        assert!(is_valid_folder_path(relative_folder_path).is_ok());
        assert!(is_valid_folder_path(absolute_folder_path).is_ok());
        assert!(is_valid_folder_path(invalid_folder_path).is_err());
    }

    #[test]
    fn test_check_if_folder_exists() {
        let valid_relative_file_path: &str = "./test.txt";
        let invalid_file_path: &str = "./test/test.txt";

        #[cfg(target_family = "windows")]
        let valid_absolute_file_path: &str = "C:/Users/test.txt";

        #[cfg(target_family = "unix")]
        let valid_absolute_file_path: &str = "/home/test.txt";

        assert!(check_if_parent_folder_exists(valid_relative_file_path));
        assert!(!check_if_parent_folder_exists(invalid_file_path));
        assert!(check_if_parent_folder_exists(valid_absolute_file_path));
    }

    #[test]
    fn test_build_full_path() {
        let binding: PathBuf = env::current_dir().unwrap();
        let current_path: &str = binding.to_str().unwrap();

        assert_eq!(build_full_path("./"), Ok(current_path.to_string() + "/"));
        assert_eq!(build_full_path("./test.txt"), Ok(current_path.to_string() + "/test.txt"));
    }

    #[test]
    fn test_build_virtual_files() {
        let file_paths: Vec<String> = vec![
            "/test1/test1.txt".to_string(),
            "/test2/test2.txt".to_string(),
        ];
        let virtual_files: Vec<VirtualFile> = build_virtual_files(file_paths);

        assert_eq!(virtual_files.len(), 2);
        assert_eq!(virtual_files[0].name, "test1.txt");
        assert_eq!(virtual_files[1].name, "test2.txt");
        assert_eq!(virtual_files[0].checksum, None);
        assert_eq!(virtual_files[1].checksum, None);
        assert_eq!(virtual_files[0].size, 0);
        assert_eq!(virtual_files[1].size, 0);
        assert_eq!(virtual_files[0].full_path, "/test1/test1.txt");
        assert_eq!(virtual_files[1].full_path, "/test2/test2.txt");
    }
}
