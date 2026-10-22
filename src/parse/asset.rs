use crate::model::asset::{AssetIndex, Index, Object};
use crate::parse::Parse;

impl Parse<&str> for AssetIndex {
    type Error = serde_json::Error;
    
    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<AssetIndex>(data)
    }
}

impl Parse<&str> for Index {
    type Error = serde_json::Error;
    
    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Index>(data)
    }   
}

impl Parse<&str> for Object {
    type Error = serde_json::Error;
    
    fn parse(data: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Object>(data)
    }  
}