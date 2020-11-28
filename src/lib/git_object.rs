use crate::lib::GitRepository;
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
            repo: GitRepository::new(),
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

pub fn factory(object_name: &str, content: String) -> Option<Box<dyn GitObject>> {
    match object_name {
        "Commit" => Some(Box::new(Commit::new())),
        _ => None,
    }
}
