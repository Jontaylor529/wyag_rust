use crate::lib::objects::git_repository::GitRepository;
use crate::lib::parsing::encoding::{object_file_format,hash_bytes_as_string};
use crate::lib::parsing::decoding::{bytes_to_object,ObjectParseError,read_repo_file,GitNameFormat, find_object};
use std::{fmt::{Display,Formatter}, path::Path, str::FromStr, rc::Rc};

#[derive(PartialEq)]
pub enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl ToString for ObjectType {
    fn to_string(&self) -> String {
        match self {
            ObjectType::Commit => "Commit".to_owned(),
            ObjectType::Tag => "Tag".to_owned(),
            ObjectType::Tree => "Tree".to_owned(),
            ObjectType::Blob => "Blob".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct InvalidObject {
    name: String,
}

impl std::fmt::Display for InvalidObject {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f,"Invalid type name: {}",self.name)
    }
}

impl FromStr for ObjectType {
    type Err = InvalidObject;

    fn from_str(s: &str) -> Result<ObjectType, InvalidObject> {
        match s.to_lowercase().as_ref() {
            "commit" => Ok(ObjectType::Commit),
            "tree" => Ok(ObjectType::Tree),
            "blob" => Ok(ObjectType::Blob),
            "tag" => Ok(ObjectType::Tag),
            _ => Err(InvalidObject { name: s.to_owned() }),
        }
    }
}

pub struct GitObject {
    kind: ObjectType,
    content: Vec<u8>,
    repo: Rc<GitRepository>,
}

impl GitObject {
    pub fn new(kind: ObjectType, content: Vec<u8>, repo: &Rc<GitRepository>) -> GitObject {
        GitObject {
            kind,
            content,
            repo: Rc::clone(repo),
        }
    }

    pub fn serialize(&self) -> &[u8] {
        match self.kind {
            ObjectType::Blob => {
                &self.content },
            _ => {
                panic!("Placeholder until I have all implementations done")
            }
        }
        
    }

    pub fn deserialize(&mut self, data: Vec<u8>) -> () {
        match self.kind {
            ObjectType::Blob => {
                self.content = data;
            },
            _ => {
                panic!("Placeholder until I have all implementations done")
            }
        }
    }

    pub fn kind(&self) -> &ObjectType {
        &self.kind
    }

    pub fn repo(&self) -> &Rc<GitRepository> {
        &self.repo
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }

    ///Write this object to the repo
    pub fn write_to_repo(&self) -> Result<(), ObjectError> {
        
        let formatted_content = object_file_format(&self);
        let hash = hash_bytes_as_string(&formatted_content);
        let target = object_file_location(&self.repo, &hash);
        std::fs::write(target, formatted_content)?;
        Ok(())  
    }

    ///Create blob object from unarchived file
    pub fn from_external_file<P: AsRef<Path>>(file: P, repo: &Rc<GitRepository>) -> Result<GitObject, ObjectError> {
        let contents = std::fs::read(file.as_ref())?;
        Ok(
            GitObject{
            repo: Rc::clone(repo),
            content: contents,
            kind: ObjectType::Blob
        }
    )
    }

    ///Create reference to a file inside the repo objects folder 
    pub fn from_internal_file(sha: &str, repo: &Rc<GitRepository>) -> Result<GitObject, ObjectError> {
        let target = object_file_location(repo, sha);
        let contents = read_repo_file(target)?;
        let object = bytes_to_object(&contents, &repo)?;
        Ok(object)
    }

    pub fn from_internal_name(repo: &Rc<GitRepository>, name: &str, fmt: &GitNameFormat, follow: bool) -> Result<GitObject,ObjectError> {
        let sha = find_object(repo, name, fmt, follow)?;
        GitObject::from_internal_file(&sha, repo)
    }

    pub fn get_hash(&self) -> String {
        let formatted_content = object_file_format(&self);
        hash_bytes_as_string(&formatted_content)
    }
    
}

fn object_file_location(repo: &GitRepository, sha: &str) -> std::path::PathBuf {
    repo.gitdir().join("objects").join(&sha[..2]).join(&sha[2..])
}

#[derive(Debug)]
pub enum ObjectError {
    FileIo(std::io::Error),
    FileParse(ObjectParseError),
}

impl Display for ObjectError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ObjectError::FileIo(io) => {write!(f, "Failed to access file: {}", io)},
            ObjectError::FileParse(parse_err) => {write!(f, "Unable to parse file: {}",parse_err)},
        }
    }
}

impl From<std::io::Error> for ObjectError {
    fn from(io_err: std::io::Error) -> Self {
        ObjectError::FileIo(io_err)
    }
}

impl From<ObjectParseError> for ObjectError {
    fn from(parse_err: ObjectParseError) -> Self{ 
        ObjectError::FileParse(parse_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::get_test_dir;
    use crate::lib::objects::git_repository::GitRepository;
    use std::rc::Rc;
    use std::path::PathBuf;

    use super::GitObject;
    #[test]
    fn read_object_from_bytes() {
        let test_dir = get_test_dir("read_object_from_bytes");
        let test_repo = Rc::new(
            GitRepository::new(
            test_dir.clone(),
            test_dir.clone().join(".git"),
            configparser::ini::Ini::new(),
        ));
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

        let res = GitObject::from_internal_file("05f01ab76171493c8ab7dc46d0abdbc94ed85372", &test_repo)
        .expect("Error reading object");
        assert!(true);
    }

    #[test]
    fn write_an_object() {
        let test_dir = get_test_dir("write_an_object");
        let test_file = test_dir
            .join(".git")
            .join("objects")
            .join("44")
            .join("94953d947add7f87c652ad6bdf7243cc945041");
        let test_repo = Rc::new(
            GitRepository::new(
            PathBuf::new(),
            test_dir.join(".git"),
            configparser::ini::Ini::new(),
        ));
        let test_obj = GitObject::new(ObjectType::Blob, "Not real content".as_bytes().to_owned(), &test_repo);

        if !test_dir.exists() {
            std::fs::create_dir_all(&test_dir).expect("Unable to create test directory");
        }

        if test_file.exists() {
            std::fs::remove_file(&test_file).expect("Failed to remove test file");
        }

        test_obj.write_to_repo().expect("Error writing hashed object");

        assert!(test_file.exists());
    }

    #[test]
    fn hash_an_object() {
        let test_repo = Rc::new(
            GitRepository::new(
            PathBuf::new(),
            PathBuf::new(),
            configparser::ini::Ini::new(),
        ));
        let test_obj = GitObject::new(ObjectType::Blob, "Not real content".as_bytes().to_owned(), &test_repo);
        let file_content = object_file_format(&test_obj);
        let hash_str = hash_bytes_as_string(&file_content);
        println!("object content is {}", String::from_utf8_lossy(test_obj.content()));
        assert!(hash_str == "4494953d947add7f87c652ad6bdf7243cc945041", "Hash was {}", hash_str);
    }
}

