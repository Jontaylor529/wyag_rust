use configparser::ini::Ini;
use flate2::read::ZlibDecoder;
use std::fs::*;
use std::io::{Error, ErrorKind, Read};
use std::path::{Path, PathBuf};

pub mod error;
pub mod objects;



//TODO this should live somehwere else
fn clean_unc(path: PathBuf) -> PathBuf {
    let str_path = path.to_string_lossy();
    if str_path.starts_with(r"\\?\") {
        //Unwrap safe because of check???
        PathBuf::from(str_path.strip_prefix(r"\\?\").unwrap())
    } else {
        path
    }
}

fn decompress_file_to_bytes(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let mut decoder = ZlibDecoder::new(&bytes[..]);
    let mut raw = Vec::<u8>::new();
    decoder.read(&mut raw);
    Ok(raw)
}

