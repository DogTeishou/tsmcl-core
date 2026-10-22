use crate::model::library::{Artifact, Download, Library, Os, Rule};
use crate::parse::Parse;

impl Parse<&str> for Library {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Library>(data)
    }
}

impl Parse<&str> for Download {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Download>(data)
    }
}

impl Parse<&str> for Rule {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Rule>(data)
    }
}

impl Parse<&str> for Os {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Os>(data)
    }
}

impl Parse<&str> for Artifact {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Artifact>(data)
    }
}