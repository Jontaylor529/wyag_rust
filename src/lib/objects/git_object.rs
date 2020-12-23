use crate::lib::objects::git_repository::GitRepository;
use configparser::ini::Ini;
use std::path::PathBuf;
use std::str::FromStr;
use std::fmt::Formatter;

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

pub struct GitObject<'a> {
    kind: ObjectType,
    content: Vec<u8>,
    repo: &'a GitRepository,
}

impl<'a> GitObject<'a> {
    pub fn new(kind: ObjectType, content: Vec<u8>, repo: &'a GitRepository) -> GitObject {
        GitObject {
            kind,
            content,
            repo,
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

    pub fn repo(&self) -> &GitRepository {
        self.repo
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }
}
