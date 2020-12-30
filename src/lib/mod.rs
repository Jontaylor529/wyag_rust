
use std::path::{PathBuf};

pub mod commands;
pub(crate) mod objects;
pub(crate) mod parsing;

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


