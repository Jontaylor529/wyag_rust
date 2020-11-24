use std::io::Error;
use std::fmt;
pub enum InternalError {
    Io(std::io::Error),
    Parse(),
    NotAGitDirectory(),
    MissingConfig(),
    UnsupportedVersion(),
    AlreadyGitDirectory(),
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InternalError::Io(io_err) => 
                write!(f,"Io error {}", io_err),
            InternalError::Parse() => 
                write!(f,"Unable to parse target file"),
            InternalError::NotAGitDirectory() => 
                write!(f, "Directory is not a git directory"),
            InternalError::MissingConfig() => 
                write!(f, "Config file not found"),
            InternalError::UnsupportedVersion() =>
                write!(f, "Unsupported repository version"),
            InternalError::AlreadyGitDirectory() =>
                write!(f, "Path already contains a git directory"),
        }
    }
} 

impl std::convert::From<std::io::Error> for InternalError {
    fn from(io_error: std::io::Error) -> Self {
        InternalError::Io(io_error)
    }
}