use crate::model::library::{DownloadItem, Download, Library};
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

impl Parse<&str> for DownloadItem {
    type Error = serde_json::Error;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<DownloadItem>(data)
    }
}