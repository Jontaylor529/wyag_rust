use std::fmt;
use std::io::Error;
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
            InternalError::Io(io_err) => write!(f, "Io error {}", io_err),
            InternalError::Parse() => write!(f, "Unable to parse target file"),
            InternalError::NotAGitDirectory() => write!(f, "Directory is not a git directory"),
            InternalError::MissingConfig() => write!(f, "Config file not found"),
            InternalError::UnsupportedVersion() => write!(f, "Unsupported repository version"),
            InternalError::AlreadyGitDirectory() => {
                write!(f, "Path already contains a git directory")
            }
        }
    }
}

impl std::convert::From<std::io::Error> for InternalError {
    fn from(io_error: std::io::Error) -> Self {
        InternalError::Io(io_error)
    }
}

pub enum ObjectParseError {
    SizeNotNumber(std::num::ParseIntError),
    ObjectWrongSize(),
    IOError(std::io::Error),
    ObjectTypeNotRecognized(),
    SizeNotFound(),
    TypeNotFound(),
}

impl std::fmt::Display for ObjectParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectParseError::SizeNotNumber(int_error) => {
                write!(f, "Could not parse object size: {}", int_error)
            }
            ObjectParseError::ObjectWrongSize() => {
                write!(f, "Object was not the size indicated in file")
            }
            ObjectParseError::IOError(io_error) => {
                write!(f, "Could not open file for reading: {}", io_error)
            }
            ObjectParseError::ObjectTypeNotRecognized() => {
                write!(f, "Object type is not recognized/supported")
            }
            ObjectParseError::SizeNotFound() => {
                write!(f, "Ascii null not found at the end of object size")
            }
            ObjectParseError::TypeNotFound() => {
                write!(f, "Ascii space not found at the end of object type")
            }
        }
    }
}

impl std::convert::From<std::io::Error> for ObjectParseError {
    fn from(io_error: std::io::Error) -> Self {
        ObjectParseError::IOError(io_error)
    }
}

impl std::convert::From<std::num::ParseIntError> for ObjectParseError {
    fn from(int_error: std::num::ParseIntError) -> Self {
        ObjectParseError::SizeNotNumber(int_error)
    }
}
