use core::fmt;

#[derive(Debug)]
pub enum DeepFinderError {
    ArgError(ArgError),
    SystemError(SystemError),
}

#[derive(Debug)]
pub enum ArgError {
    NoArgument,
    MissingConfiguration,
}

#[derive(Debug)]
pub enum SystemError {
    InvalidPath(String),
    InvalidFilename(String),
    UnableToCreateFile(String, String),
    PathTooLong(String),
    ParentFolderDoesntExist(String),
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
            ArgError::NoArgument => write!(f, "Error: no argument specified\nUsage: worgenX <command> [options]\nTry 'worgenX --help' for more information."),
            ArgError::MissingConfiguration => write!(f, "Error: no configuration given for argument.\nPlease specify the mandatory parameters and at least one type of characters.\nUsage: worgenX <command> [options]\nTry 'worgenX --help' for more information."),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SystemError::InvalidPath(p) => write!(f, "Error: invalid path '{}'", p),
            SystemError::InvalidFilename(file) => write!(f, "Error: invalid filename '{}'", file),
            SystemError::UnableToCreateFile(p, e) => write!(f, "Error: unable to create file '{}': {}", p, e),
            SystemError::PathTooLong(p) => write!(f, "Error: path too long '{}'", p),
            SystemError::ParentFolderDoesntExist(p) => write!(f, "Error: parent folder doesn't exist '{}'", p),
        }
    }
}