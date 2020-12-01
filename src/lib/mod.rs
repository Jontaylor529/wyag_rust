use flate2::read::ZlibDecoder;
use std::io::{Read};
use std::path::{Path, PathBuf};

pub mod error;
pub mod objects;
pub mod commands;

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
    [
        "C:\\", "users", "gameo", "appdata", "local", "temp", "testing", sub_dir,
    ]
    .iter()
    .collect::<PathBuf>()
}
