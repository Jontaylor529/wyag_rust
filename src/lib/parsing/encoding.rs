use crate::lib::objects::git_object::{GitObject};
use sha1::{Digest, Sha1};

///Pack object info into the git object style
pub fn object_file_format(object: &GitObject) -> Vec<u8> {
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

///Return the SHA1 hash as a string of hex characters
pub fn hash_bytes_as_string(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&content);
    let result = hasher.finalize();
    byte_array_to_string(result.as_slice())
}

///Turn an array of bytes into a string of hex characters
fn byte_array_to_string(arr: &[u8]) -> String {
    let hash_iter = arr.iter().map(|v| format!("{:x}", v));
    let mut hash_str = "".to_owned();
    for val in hash_iter {
        hash_str.push_str(val.as_ref());
    }
    hash_str
}