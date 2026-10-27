pub mod manifest;
pub mod library;
pub mod vanilla_version;
pub mod asset;
pub mod argument;
pub mod rule;
pub mod forge;

pub trait Allowed {
    fn allowed(&self) -> bool;
}

pub trait GameVersion {
    fn id(&self) -> String;
}