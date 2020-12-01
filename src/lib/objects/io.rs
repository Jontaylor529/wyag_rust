use super::git_repository::GitRepository;
use super::git_object::{GitObject,factory};
use super::super::error::object::ObjectParseError;
use std::path::{PathBuf,Path};
use flate2::read::ZlibDecoder;
use std::io::Read;


fn decompress_file_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let mut decoder = ZlibDecoder::new(&bytes[..]);
    let mut raw = Vec::<u8>::new();
    decoder.read(&mut raw)?;
    Ok(raw)
}

pub fn read_object(repo: &GitRepository, sha: &str) -> Result<Box<dyn GitObject>, ObjectParseError> {
    let rel_path: PathBuf = ["objects", &sha[..1], &sha[2..]].iter().collect();
    let path = repo.repo_file(&rel_path, false)?;
    let raw = decompress_file_to_bytes(&path)?;
    let is_ascii_space = |b: &u8| *b == 0x20;
    let is_ascii_null = |b: &u8| *b == 0x00;
    if let Some(end_object_type) = raw.iter().position(is_ascii_space) {
        if let Some(end_obj_size) = raw.iter().skip(end_object_type).position(is_ascii_null) {
            let obj_type = &raw[..end_object_type - 1];
            let obj_size = &raw[end_object_type + 1..end_obj_size - 1];
            let obj_content = &raw[end_obj_size + 1..];

            let obj_size: usize = String::from_utf8_lossy(&obj_size).parse()?;
            if obj_size == obj_content.len() {
                let obj_type = String::from_utf8_lossy(obj_type);
                let obj_content = String::from_utf8_lossy(obj_content);
                if let Some(git_object) =
                    factory(&(*obj_type), (*obj_content).to_owned())
                {
                    Ok(git_object)
                } else {
                    Err(ObjectParseError::ObjectTypeNotRecognized())
                }
            } else {
                Err(ObjectParseError::ObjectWrongSize())
            }
        } else {
            Err(ObjectParseError::SizeNotFound())
        }
    } else {
        Err(ObjectParseError::ObjectTypeNotRecognized())
    }
}