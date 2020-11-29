use std::fmt;

pub enum GitError {
    Io(std::io::Error),
    Parse(),
    NotAGitDirectory(),
    MissingConfig(),
    UnsupportedVersion(),
    AlreadyGitDirectory(),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitError::Io(io_err) => write!(f, "Io error {}", io_err),
            GitError::Parse() => write!(f, "Unable to parse target file"),
            GitError::NotAGitDirectory() => write!(f, "Directory is not a git directory"),
            GitError::MissingConfig() => write!(f, "Config file not found"),
            GitError::UnsupportedVersion() => write!(f, "Unsupported repository version"),
            GitError::AlreadyGitDirectory() => {
                write!(f, "Path already contains a git directory")
            }
        }
    }
}

impl std::convert::From<std::io::Error> for GitError {
    fn from(io_error: std::io::Error) -> Self {
        GitError::Io(io_error)
    }
}
