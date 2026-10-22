use crate::model::version::{Downloads, Libraries, MainJar, Version};
use crate::parse::Parse;

impl Parse<&str> for Libraries {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Libraries>(data)
    }
}

impl Parse<&str> for Version {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Version>(data)
    }
}

impl Parse<&str> for Downloads {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Downloads>(data)
    }
}

impl Parse<&str> for MainJar {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<MainJar>(data)
    }
}