pub mod manifest;
pub mod library;
pub mod asset;
pub mod version;
pub mod forge_metadata;

pub trait Parse<T>: Sized {
    type Error;
    fn parse(data: &str) -> Result<Self, Self::Error>;
}