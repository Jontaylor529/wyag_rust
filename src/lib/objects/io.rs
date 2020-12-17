use super::super::error::object::ObjectParseError;
use super::git_object::{GitObject, ObjectType};
use super::git_repository::{repo_file, GitRepository};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

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

fn compress_str(content: &str) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content.as_bytes());
    //safe to unwrap since no real io?
    encoder.finish().unwrap()
}

fn parse_object_bytes(bytes: &[u8]) -> Result<ObjectData, ObjectParseError> {
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

fn validate_object<'a>(
    repo: &'a GitRepository,
    data: ObjectData,
) -> Result<GitObject<'a>, ObjectParseError> {
    let obj_size: usize = data.length.parse()?;
    if obj_size == data.content.len() {
        if let Ok(obj_type) = data.obj_type.parse::<ObjectType>() {
            Ok(GitObject::new(obj_type, data.content, repo))
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

pub fn read_object<'a>(
    repo: &'a GitRepository,
    sha: &str,
) -> Result<GitObject<'a>, ObjectParseError> {
    let rel_path: PathBuf = ["objects", &sha[..2], &sha[2..]].iter().collect();
    let path = repo_file(repo, &rel_path, false)?;
    let raw = decompress_file_to_bytes(&path)?;
    println!("raw size = {}", raw.len());
    let data = parse_object_bytes(&raw)?;
    validate_object(repo, data)
}

fn format_object(object: &GitObject) -> Vec<u8> {
    let data = object.serialize();
    let type_string = object.kind().to_string();
    let type_bytes = type_string.as_bytes();
    let len_string = data.len().to_string();
    let len_bytes = len_string.as_bytes();
    let null: u32 = 0x00;
    let null = null.to_be_bytes();
    let result = [type_bytes, b" ", len_bytes, &null, &data].concat();
    result
}

fn hash_object(object: &GitObject) -> Vec<u8> {
    let result = format_object(object);
    let mut hasher = Sha1::new();
    hasher.update(&result);
    hasher.finalize().as_slice().to_owned()
}

fn write_object(object: &GitObject) -> Result<(), std::io::Error> {
    let hash = hash_object(object);
    let hash_iter = hash.iter().map(|v| format!("{:x}", v));
    let mut hash_str = "".to_owned();
    for val in hash_iter {
        hash_str.push_str(val.as_ref());
    }
    let path = PathBuf::new()
        .join("objects")
        .join(&hash_str[..2])
        .join(&hash_str[2..]);
    let file = repo_file(object.repo(), path, true)?;
    std::fs::write(file, format_object(object))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::get_test_dir;
    #[test]
    fn parse_bytes() {
        let bytes: [u8; 13] = [
            0x63, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x20, 0x34, 0x00, 0x74, 0x72, 0x65, 0x65,
        ];
        let data = parse_object_bytes(&bytes).expect("Byte parsing error: ");
        let obj_type: ObjectType = data.obj_type.parse().expect("Error parsing object type");
        assert!(obj_type == ObjectType::Commit);
        assert!(
            data.length.parse::<i32>().expect("Error parsing size") == 4,
            "data length {}",
            data.length
        );
        assert!(data.content == "tree");
    }

    #[test]
    fn read_object_from_bytes() {
        let test_dir = get_test_dir("read_object_from_bytes");
        let test_repo = GitRepository::new(
            test_dir.clone(),
            test_dir.clone().join(".git"),
            configparser::ini::Ini::new(),
        );
        let file_dir = test_dir.join(r".git\objects\05");
        let src_path = std::env::current_dir()
            .unwrap()
            .join(r"src\test\05\f01ab76171493c8ab7dc46d0abdbc94ed85372");
        let mut src = std::fs::File::open(&src_path).expect(
            format!(
                "Error opening test file source at {}",
                src_path.to_string_lossy()
            )
            .as_ref(),
        );
        let test_file = file_dir.join("f01ab76171493c8ab7dc46d0abdbc94ed85372");

        if !file_dir.exists() {
            std::fs::create_dir_all(file_dir).expect("Failed to create test_dir");
        }

        if !test_file.exists() {
            let mut target = std::fs::File::create(&test_file).expect("Failed to create test file");
            std::io::copy(&mut src, &mut target).expect("Failed to copy test file");
        }

        let res = read_object(&test_repo, "05f01ab76171493c8ab7dc46d0abdbc94ed85372")
            .expect("Error reading object");
        assert!(true);
    }

    #[test]
    fn hash_an_object() {
        let test_repo = GitRepository::new(
            PathBuf::new(),
            PathBuf::new(),
            configparser::ini::Ini::new(),
        );
        let test_obj = GitObject::new(ObjectType::Blob, "Not real content".to_owned(), &test_repo);
        let hash = hash_object(&test_obj);
        let hash_str: String = format!("{:x?}", &hash);
        assert!(hash_str == "[cc, d4, e8, f9, 5c, f9, 1b, 38, da, 8b, 25, ba, 2d, 77, bc, 74, c2, a8, 81, d9]", "Hash was {}", hash_str);
    }

    #[test]
    fn write_an_object() {
        let test_dir = get_test_dir("write_an_object");
        let test_file = test_dir
            .join(".git")
            .join("objects")
            .join("cc")
            .join("d4e8f95cf91b38da8b25ba2d77bc74c2a881d9");
        let test_repo = GitRepository::new(
            PathBuf::new(),
            test_dir.join(".git"),
            configparser::ini::Ini::new(),
        );
        let test_obj = GitObject::new(ObjectType::Blob, "Not real content".to_owned(), &test_repo);

        if !test_dir.exists() {
            std::fs::create_dir_all(&test_dir).expect("Unable to create test directory");
        }

        if test_file.exists() {
            std::fs::remove_file(&test_file).expect("Failed to remove test file");
        }

        write_object(&test_obj).expect("Error writing hashed object");

        assert!(test_file.exists());
    }
}
