use std::fmt;
use crate::lib::objects::git_object::InvalidObject;

#[derive(Debug)]
pub enum GitError {
    Io(std::io::Error),
    Parse(),
    NotAGitDirectory(),
    MissingConfig(),
    UnsupportedVersion(),
    AlreadyGitDirectory(),
    InvalidType(InvalidObject),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitError::Io(io_err) => write!(f, "Io error {}", io_err),
            GitError::Parse() => write!(f, "Unable to parse target file"),
            GitError::NotAGitDirectory() => write!(f, "Directory is not a git directory"),
            GitError::MissingConfig() => write!(f, "Config file not found"),
            GitError::UnsupportedVersion() => write!(f, "Unsupported repository version"),
            GitError::AlreadyGitDirectory() => write!(f, "Path already contains a git directory"),
            GitError::InvalidType(name) => write!(f, "Invalid type name {}", name),
        }
    }
}

impl std::convert::From<std::io::Error> for GitError {
    fn from(io_error: std::io::Error) -> Self {
        GitError::Io(io_error)
    }
}

impl From<InvalidObject> for GitError {
    fn from(obj_error: InvalidObject) -> Self {
        GitError::InvalidType(obj_error)
    }
}
