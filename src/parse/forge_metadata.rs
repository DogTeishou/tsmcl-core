use crate::model::forge::Metadata;
use crate::parse::Parse;

impl Parse<&str> for Metadata {
    type Error = quick_xml::de::DeError;

    fn parse(data: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str::<Metadata>(data)
    }
}