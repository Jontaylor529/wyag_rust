use crate::lib::GitRepository;
pub trait GitObject {
    fn repo(&self) ->  GitRepository;
    fn serialize(&self) -> ();
    fn deserialize(&self) -> ();
}