use std::fmt;

#[derive(Debug)]
pub enum ObjectParseError {
    SizeNotNumber(std::num::ParseIntError),
    ObjectWrongSize(),
    IOError(std::io::Error),
    ObjectTypeNotRecognized(String),
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
            ObjectParseError::ObjectTypeNotRecognized(s) => {
                write!(f, "Object type is not recognized/supported {}", s)
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
