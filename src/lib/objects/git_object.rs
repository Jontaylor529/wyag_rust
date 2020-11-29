use std::path::PathBuf;
use crate::lib::objects::git_repository::GitRepository;
use configparser::ini::Ini;
pub trait GitObject {
    fn repo(&self) -> &GitRepository;
    fn serialize(&self) -> ();
    fn deserialize(&self) -> ();
}

pub struct Commit {
    repo: GitRepository,
}

impl Commit {
    pub fn new() -> Commit {
        Commit {
            repo: GitRepository::new(PathBuf::new(),PathBuf::new(),Ini::new()),
        }
    }
}

impl GitObject for Commit {
    fn repo(&self) -> &GitRepository {
        &self.repo
    }

    fn serialize(&self) -> () {}

    fn deserialize(&self) -> () {}
}

pub struct Tree {}

impl Tree {
    pub fn new() -> Tree {
        Tree {}
    }
}

pub struct Blob {}

impl Blob {
    pub fn new() -> Blob {
        Blob {}
    }
}
pub struct Tag {}

impl Tag {
    pub fn new() -> Tag {
        Tag {}
    }
}

pub fn factory(object_name: &str, _content: String) -> Option<Box<dyn GitObject>> {
    match object_name {
        "Commit" => Some(Box::new(Commit::new())),
        _ => None,
    }
}
