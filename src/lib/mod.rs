use flate2::read::ZlibDecoder;
use std::io::Read;
use std::path::{Path, PathBuf};

pub mod commands;
pub mod error;
pub mod objects;

fn clean_unc(path: PathBuf) -> PathBuf {
    let str_path = path.to_string_lossy();
    if str_path.starts_with(r"\\?\") {
        //Unwrap safe because of check???
        PathBuf::from(str_path.strip_prefix(r"\\?\").unwrap())
    } else {
        path
    }
}

fn get_test_dir(sub_dir: &str) -> PathBuf {
    std::env::temp_dir().join("testing").join(sub_dir)
}

fn hash_array_to_string(arr: &[u8]) -> String {
    let hash_iter = arr.iter().map(|v| format!("{:x}", v));
    let mut hash_str = "".to_owned();
    for val in hash_iter {
        hash_str.push_str(val.as_ref());
    }
    hash_str
}
