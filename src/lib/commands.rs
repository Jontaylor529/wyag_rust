use crate::lib::error::git::GitError;
use configparser::ini::Ini;
use std::path::PathBuf;
use crate::lib::objects::git_repository::*;
use crate::lib::objects::io::{read_object,find_object,GitNameFormat,hash_file};

use super::{hash_array_to_string, objects::git_object::ObjectType};

fn default_config() -> Ini {
    let mut config = Ini::new();
    config.set("core", "repositoryformatversion", Some("0".to_owned()));
    config.set("core", "filemode", Some("false".to_owned()));
    config.set("core", "bare", Some("false".to_owned()));
    config
}

pub fn init<P: Into<PathBuf>>(path: P) -> Result<(), GitError> {
    let path: PathBuf = path.into();
    let git_dir = path.join(".git");
    if git_dir.exists() {
        Err(GitError::AlreadyGitDirectory())
    } else {
        std::fs::create_dir_all(&git_dir)?;
        let config = default_config();
        config.write(git_dir.join("config").to_str().unwrap())?;
        Ok(())
    }
}

pub fn cat_file<P: Into<PathBuf>>(git_dir_path: P, type_str: &str, target: &str) -> Result<(), GitError> {
    let git_dir_path = find_repo_dir(git_dir_path)?;
    let repo = GitRepository::at_path(git_dir_path, false)?;
    let hash = find_object(&repo,target,&GitNameFormat::Placeholder,true)?;
    let object = read_object(&repo, &hash).or(Err(GitError::Parse()))?;
    println!("{}",String::from_utf8_lossy(object.serialize()));
    Ok(())
}

pub fn hash_object_cmd(object_type: &str, file: &str, write: bool) -> Result<(),GitError> {
    let repo: Option<GitRepository> = None;
    if write {
        let repo = Some(
            GitRepository::at_path(
                find_repo_dir(
                    std::env::current_dir()?
                )?, false)?
            );
    }
    hash_object(object_type, file, repo)
}

///Creates hash for the given file and adds it to a repo, if one is given
pub fn hash_object(object_type: &str, file: &str, repo: Option<GitRepository>) -> Result<(), GitError> {
    let hash = hash_file(file, object_type, repo)?;
    println!("{}", hash_array_to_string(&hash));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{hash_object, init};
    use crate::lib::get_test_dir;
    use crate::lib::objects::git_repository::GitRepository;
    #[test]
    fn create_default_repo() {
        let test_dir = get_test_dir("create_default_repo");
        if test_dir.join(".git").exists() {
            std::fs::remove_dir_all(&test_dir.join(".git")).expect("Error cleaning directory");
        }
        match init(test_dir.to_str().unwrap()) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "Error initializing repo: {}", err),
        }
    }

    #[test]
    fn hash_a_file() {
        let test_dir = get_test_dir("hash_a_file");
        if test_dir.join(".git").exists() {
            std::fs::remove_dir_all(&test_dir.join(".git")).expect("Error cleaning directory");
        }
        let test_file = std::env::current_dir()
            .expect("Unable to find test file")
            .join(r"src\test\blob_test.txt");

        init(&test_dir).expect("unable to create git dir at test dir");
        let repo = GitRepository::at_path( &test_dir, false).expect("Error opening repo");
        match hash_object("blob", test_file.to_str().unwrap(), Some(repo)) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "Error hashing object: {}",err),
        }
    }
}
