pub mod version_manifest;
pub mod library;
pub mod asset;
pub mod version;

pub trait Parse<T>: Sized {
    type Error;
    fn parse(data: &str) -> Result<Self, Self::Error>;
}