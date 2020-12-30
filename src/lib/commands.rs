use configparser::ini::Ini;
use std::path::PathBuf;
use crate::lib::objects::git_repository::*;
use crate::lib::objects::git_object::{GitObject,ObjectError};
use crate::lib::parsing::decoding::{GitNameFormat};
use std::rc::Rc;

#[derive(Debug)]
pub enum CommandError {
    Init(String),
    Repo(RepositoryError),
    Object(ObjectError),
}

impl From<RepositoryError> for CommandError {
    fn from(repo_err: RepositoryError) -> Self {
        CommandError::Repo(repo_err)
    }
}

impl From<ObjectError> for CommandError {
    fn from(obj_err: ObjectError) -> Self {
        CommandError::Object(obj_err)
    } 
}

fn default_config() -> Ini {
    let mut config = Ini::new();
    config.set("core", "repositoryformatversion", Some("0".to_owned()));
    config.set("core", "filemode", Some("false".to_owned()));
    config.set("core", "bare", Some("false".to_owned()));
    config
}

pub fn init<P: Into<PathBuf>>(path: P) -> Result<(), CommandError> {
    let path: PathBuf = path.into();
    let git_dir = path.join(".git");
    if git_dir.exists() {
        Err(CommandError::Init("Already a git directory".to_owned()))
    } else {
        std::fs::create_dir_all(&git_dir).or(Err(CommandError::Init("Cannot create .git dirctory".to_owned())));
        let config = default_config();
        config.write(git_dir.join("config").to_str().unwrap());
        Ok(())
    }
}

pub fn cat_file<P: Into<PathBuf>>(git_dir_path: P, type_str: &str, target: &str) -> Result<(), CommandError> {
    let repo = GitRepository::along_path(git_dir_path.into(), false)?;
    let repo = Rc::new(repo);
    let object = GitObject::from_internal_name(&repo, target, &GitNameFormat::Placeholder,true)?;
    println!("{}",String::from_utf8_lossy(object.serialize()));
    Ok(())
}

///Creates hash for the given file and possibly adds it to a repo
pub fn hash_object(object_type: &str, file: &str, repo: &Rc<GitRepository>, write:bool) -> Result<(), CommandError> {
    let blob = GitObject::from_external_file(file, repo)?;
    if write {
        blob.write_to_repo()?;
    }
    println!("{}", blob.get_hash());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{hash_object, init};
    use crate::lib::get_test_dir;
    use crate::lib::objects::git_repository::GitRepository;
    use std::rc::Rc;
    #[test]
    fn create_default_repo() {
        let test_dir = get_test_dir("create_default_repo");
        if test_dir.join(".git").exists() {
            std::fs::remove_dir_all(&test_dir.join(".git")).expect("Error cleaning directory");
        }
        match init(test_dir.to_str().unwrap()) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "Error initializing repo: {:?}", err),
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
        let repo = Rc::new(GitRepository::at_path( &test_dir, false).expect("Error opening repo"));
        match hash_object("blob", test_file.to_str().unwrap(), &repo,false) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "Error hashing object: {:?}",err),
        }
    }
}
