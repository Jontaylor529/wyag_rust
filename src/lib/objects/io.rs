use super::git_repository::{GitRepository,repo_file};
use super::git_object::{GitObject,ObjectType};
use super::super::error::object::ObjectParseError;
use std::path::{PathBuf,Path};
use flate2::read::ZlibDecoder;
use std::io::Read;

struct ObjectData {
    length: String,
    obj_type: String,
    content: String,
}

fn decompress_file_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let mut decoder = ZlibDecoder::new(&bytes[..]);
    let mut raw = Vec::<u8>::new();
    decoder.read_to_end(&mut raw)?;
    Ok(raw)
}

fn parse_object_bytes(bytes: &[u8]) -> Result<ObjectData, ObjectParseError> {
    let is_ascii_space = |b: &u8| *b == 0x20;
    let is_ascii_null = |b: &u8| *b == 0x00;

    if let Some(end_object_type) = bytes.iter().position(is_ascii_space) {
        if let Some(end_obj_size) = bytes[end_object_type ..].iter().position(is_ascii_null) {
            let end_obj_size = end_obj_size + end_object_type;
            let obj_type = &bytes[..end_object_type];
            println!("end type {}, end size {}",end_object_type,end_obj_size);
            let length = &bytes[(end_object_type + 1)..end_obj_size];
            let content = &bytes[(end_obj_size + 1)..];
            let obj_type = String::from_utf8_lossy(obj_type);
            let length = String::from_utf8_lossy(length);
            let content = String::from_utf8_lossy(content);
            Ok (
                ObjectData {
                length: (*length).to_owned(),
                obj_type: (*obj_type).to_owned(),
                content: (*content).to_owned(),
            }
        )
        } else {
            Err(ObjectParseError::SizeNotFound())
        }
    } else {
        Err(ObjectParseError::TypeNotFound())
    }
}

fn validate_object<'a>(repo: &'a GitRepository, data: ObjectData) -> Result<GitObject<'a>, ObjectParseError> {
    let obj_size: usize = data.length.parse()?;
    if obj_size == data.content.len() {
        if let Ok(obj_type) =
            data.obj_type.parse::<ObjectType>()
        {
            Ok(GitObject::new(
                obj_type,
                data.content,
                repo
            )
            )

        } else {
            Err(ObjectParseError::ObjectTypeNotRecognized(format!("{}",data.obj_type)))
        }
    } else {
        Err(ObjectParseError::ObjectWrongSize())
    }
}

pub fn read_object<'a>(repo: &'a GitRepository, sha: &str) -> Result<GitObject<'a>, ObjectParseError> {
    let rel_path: PathBuf = ["objects", &sha[..2], &sha[2..]].iter().collect();
    let path = repo_file(repo, &rel_path, false)?;
    let raw = decompress_file_to_bytes(&path)?;
    println!("raw size = {}",raw.len());
    let data = parse_object_bytes(&raw)?;
    validate_object(repo, data)
}

#[cfg(test)]
mod tests {
    use crate::lib::get_test_dir;
    use super::*;
    #[test]
    fn parse_bytes() {
        let bytes: [u8; 13] = [0x63, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x20, 0x34, 0x00, 0x74, 0x72, 0x65, 0x65];
        let data = parse_object_bytes(&bytes).expect("Byte parsing error: ");
        let obj_type: ObjectType = data.obj_type.parse().expect("Error parsing object type");
        assert!(obj_type == ObjectType::Commit);
        assert!(data.length.parse::<i32>().expect("Error parsing size") == 4, "data length {}", data.length);
        assert!(data.content == "tree");      
    }

    #[test]
    fn read_object_from_bytes() {
        let test_dir = get_test_dir("read_object_from_bytes");
        let test_repo = GitRepository::new(test_dir.clone(), test_dir.clone().join(".git"),configparser::ini::Ini::new());
        let file_dir = test_dir.join(r".git\objects\05");
        let src_path = std::env::current_dir()
        .unwrap()
        .join(r"src\test\05\f01ab76171493c8ab7dc46d0abdbc94ed85372");
        let mut src = std::fs::File::open(&src_path)
        .expect(format!("Error opening test file source at {}", src_path.to_string_lossy()).as_ref());
        let test_file = file_dir.join("f01ab76171493c8ab7dc46d0abdbc94ed85372");

        if !file_dir.exists() {
            std::fs::create_dir_all(file_dir).expect("Failed to create test_dir");
        }
        
        if !test_file.exists() {
            let mut target = std::fs::File::create(&test_file).expect("Failed to create test file");
            std::io::copy(&mut src, &mut target).expect("Failed to copy test file");
        }

        let res = read_object(&test_repo, "05f01ab76171493c8ab7dc46d0abdbc94ed85372").expect("Error reading object");
        assert!(true);
    }
}