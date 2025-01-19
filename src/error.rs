// External crates.
use core::fmt;

#[derive(Debug, PartialEq)]
pub enum DeepFinderError {
    ArgError(ArgError),
    SystemError(SystemError),
}

#[derive(Debug, PartialEq)]
pub enum ArgError {
    NoPathSpecified,
}

#[derive(Debug, PartialEq)]
pub enum SystemError {
    InvalidPath(String),
    InvalidFilename(String),
    UnableToCreateFile(String, String),
    #[cfg(target_family = "windows")]
    PathTooLong(String),
    ParentFolderDoesntExist(String),
    InvalidFolder(String),
    UnableToReadDir(String),
    UnableToGetCurrentDir(String),
}

impl fmt::Display for DeepFinderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeepFinderError::ArgError(e) => write!(f, "{}", e),
            DeepFinderError::SystemError(e) => write!(f, "{}", e),
        }
    }
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::NoPathSpecified => write!(f, "Error: no path specified.\nUsage: deefinder <path> [options]\nTry 'deefinder --help' for more information."),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SystemError::InvalidFolder(p) => write!(f, "Error: invalid folder specified '{}'.\nThis folder may not exist.\nUsage: deefinder <path> [options]\nTry 'deefinder --help' for more information.", p),
            SystemError::InvalidPath(p) => write!(f, "Error: invalid path '{}'", p),
            SystemError::InvalidFilename(file) => write!(f, "Error: invalid filename '{}'", file),
            SystemError::UnableToCreateFile(p, e) => write!(f, "Error: unable to create file '{}': {}", p, e),
            #[cfg(target_family = "windows")]
            SystemError::PathTooLong(p) => write!(f, "Error: path too long '{}'", p),
            SystemError::ParentFolderDoesntExist(p) => write!(f, "Error: parent folder doesn't exist '{}'", p),
            SystemError::UnableToReadDir(p) => write!(f, "Error: unable to read directory '{}'", p),
            SystemError::UnableToGetCurrentDir(e) => write!(f, "Error: unable to get current directory.\n{}", e),
        }
    }
}