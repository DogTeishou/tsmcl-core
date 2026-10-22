use crate::model::version_manifest::{Latest, VersionManifest, Version};
use crate::parse::Parse;

impl Parse<&str> for VersionManifest {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<VersionManifest>(data)
    }
}

impl Parse<&str> for Latest {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Latest>(data)
    }
}

impl Parse<&str> for Version {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Version>(data)
    }
}
