use std::path::PathBuf;
use crate::lib::objects::git_repository::GitRepository;
use configparser::ini::Ini;
use std::str::FromStr;

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

impl FromStr for ObjectType {
    type Err = InvalidObject;

    fn from_str(s: &str) -> Result<ObjectType, InvalidObject> {
        match s.to_lowercase().as_ref() {
            "commit" => Ok(ObjectType::Commit),
            "tree" => Ok(ObjectType::Tree),
            "blob" => Ok(ObjectType::Blob),
            "tag" => Ok(ObjectType::Tag),
            _ => Err(InvalidObject{name: s.to_owned()}),
        }
    }
}

pub struct GitObject<'a> {
    kind: ObjectType,
    content: String,
    repo: &'a GitRepository,
}

impl<'a> GitObject<'a> {
    pub fn new(kind: ObjectType, content: String, repo: &'a GitRepository) -> GitObject {
        GitObject {
            kind,
            content,
            repo,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        Vec::<u8>::new()
    }

    pub fn deserialize() -> () {

    }

    pub fn kind(&self) -> &ObjectType {
        &self.kind
    }

    pub fn repo(&self) -> &GitRepository {
        self.repo
    }

}

