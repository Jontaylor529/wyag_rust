
use crate::lib::objects::git_object::{GitObject,ObjectType};
use crate::lib::objects::git_repository::{GitRepository};
use flate2::read::ZlibDecoder;
use std::io::Read;
use std::path::Path;
use std::fmt;
use std::rc::Rc;

///Parse decompressed object file bytes into an object
pub fn bytes_to_object(bytes: &[u8], repo: &Rc<GitRepository>) -> Result<GitObject, ObjectParseError> {
    let data = parse_bytes(bytes)?;
    validate_object(repo, data)
}

///Reads the given file and returns the decompressed bytes
pub fn read_repo_file <P: AsRef<Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let mut decoder = ZlibDecoder::new(&bytes[..]);
    let mut raw = Vec::<u8>::new();
    decoder.read_to_end(&mut raw)?;
    Ok(raw)
}

pub enum GitNameFormat {
    Placeholder,
}

///Finds an object using the given name format and returns the full sha name
pub(crate) fn find_object<'a>(repo: &GitRepository, name: &str, fmt: &GitNameFormat, follow: bool) -> Result<String,std::io::Error> {
    Ok(String::default())
}

struct ObjectData {
    length: String,
    obj_type: String,
    content: String,
}

fn parse_bytes (bytes: &[u8]) -> Result<ObjectData, ObjectParseError> {
    let is_ascii_space = |b: &u8| *b == 0x20;
    let is_ascii_null = |b: &u8| *b == 0x00;

    if let Some(end_object_type) = bytes.iter().position(is_ascii_space) {
        if let Some(end_obj_size) = bytes[end_object_type..].iter().position(is_ascii_null) {
            let end_obj_size = end_obj_size + end_object_type;
            let obj_type = &bytes[..end_object_type];
            println!("end type {}, end size {}", end_object_type, end_obj_size);
            let length = &bytes[(end_object_type + 1)..end_obj_size];
            let content = &bytes[(end_obj_size + 1)..];
            let obj_type = String::from_utf8_lossy(obj_type);
            let length = String::from_utf8_lossy(length);
            let content = String::from_utf8_lossy(content);
            Ok(ObjectData {
                length: (*length).to_owned(),
                obj_type: (*obj_type).to_owned(),
                content: (*content).to_owned(),
            })
        } else {
            Err(ObjectParseError::SizeNotFound())
        }
    } else {
        Err(ObjectParseError::TypeNotFound())
    }
}

fn validate_object(
    repo: & Rc<GitRepository>,
    data: ObjectData,
) -> Result<GitObject, ObjectParseError> {
    let obj_size: usize = data.length.parse()?;
    if obj_size == data.content.len() {
        if let Ok(obj_type) = data.obj_type.parse::<ObjectType>() {
            Ok(GitObject::new(obj_type, data.content.as_bytes().to_owned(), repo))
        } else {
            Err(ObjectParseError::ObjectTypeNotRecognized(format!(
                "{}",
                data.obj_type
            )))
        }
    } else {
        Err(ObjectParseError::ObjectWrongSize())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::objects::git_object::ObjectType;
    #[test]
    fn parse_data_as_bytes() {
        let bytes: [u8; 13] = [
            0x63, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x20, 0x34, 0x00, 0x74, 0x72, 0x65, 0x65,
        ];
        let data = parse_bytes(&bytes).expect("Byte parsing error: ");
        let obj_type: ObjectType = data.obj_type.parse().expect("Error parsing object type");
        assert!(obj_type == ObjectType::Commit);
        assert!(
            data.length.parse::<i32>().expect("Error parsing size") == 4,
            "data length {}",
            data.length
        );
        assert!(data.content == "tree");
    }
}
