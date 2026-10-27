use crate::model::vanilla_version::{Downloads, Libraries, MainJar, VanillaVersion};
use crate::parse::Parse;

impl Parse<&str> for Libraries {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Libraries>(data)
    }
}

impl Parse<&str> for VanillaVersion {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<VanillaVersion>(data)
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