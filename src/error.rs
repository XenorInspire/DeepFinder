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
    SyntaxError,
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
    UnsupportedHashAlgorithm(String),
}

impl fmt::Display for DeepFinderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ArgError(e) => write!(f, "{}", e),
            Self::SystemError(e) => write!(f, "{}", e),
        }
    }
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoPathSpecified => write!(f, "Error: no path specified.\nUsage: deefinder <path> [options]\nTry 'deefinder --help' for more information."),
            Self::SyntaxError => write!(f, "Error: syntax error, please check the command line arguments.\nUsage: deefinder <path> [options]\nTry 'deefinder --help' for more information."),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidFolder(p) => write!(f, "Error: invalid folder specified '{}'.\nThis folder may not exist.\nUsage: deefinder <path> [options]\nTry 'deefinder --help' for more information.", p),
            Self::InvalidPath(p) => write!(f, "Error: invalid path '{}'", p),
            Self::InvalidFilename(file) => write!(f, "Error: invalid filename '{}'", file),
            Self::UnableToCreateFile(p, e) => write!(f, "Error: unable to create file '{}': {}", p, e),
            #[cfg(target_family = "windows")]
            Self::PathTooLong(p) => write!(f, "Error: path too long '{}'", p),
            Self::ParentFolderDoesntExist(p) => write!(f, "Error: parent folder doesn't exist '{}'", p),
            Self::UnableToReadDir(p) => write!(f, "Error: unable to read directory '{}'", p),
            Self::UnableToGetCurrentDir(e) => write!(f, "Error: unable to get current directory.\n{}", e),
            Self::UnsupportedHashAlgorithm(a) => write!(f, "Error: unsupported hash algorithm '{}'", a),
        }
    }
}