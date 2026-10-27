use serde::{Deserialize, Serialize};
use crate::model::argument::Argument;
use crate::model::library::{DownloadItem, Library};
use crate::model::asset::AssetIndex;
use crate::model::GameVersion;

pub type Libraries = Vec<Library>;

#[derive(Deserialize, Serialize)]
pub struct VanillaVersion {
    #[serde(alias = "minecraftArguments", skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
    #[serde(alias = "assetIndex", skip_serializing_if = "Option::is_none")]
    pub asset_index: Option<AssetIndex>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<String>,
    #[serde(alias = "complianceLevel", skip_serializing_if = "Option::is_none")]
    pub compliance_level: Option<u32>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub libraries: Option<Libraries>,
    #[serde(alias = "mainClass", skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(alias = "releaseTime")]
    pub release_time: String,
    pub time: String,
    #[serde(alias = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    #[serde(alias = "minimumLauncherVersion", skip_serializing_if = "Option::is_none")]
    pub minimum_launcher_version: Option<u32>,
    #[serde(alias = "clientVersion", skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
    #[serde(alias = "javaVersion", skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,
}

#[derive(Deserialize, Serialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(alias = "majorVersion")]
    pub major_version: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Downloads {
    pub client: MainJar,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<MainJar>,
}

#[derive(Deserialize, Serialize)]
pub struct MainJar {
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Arguments{
    #[serde(alias = "default-user-jvm", skip_serializing_if = "Option::is_none")]
    pub default_user_jvm: Option<Vec<Argument>>,
    pub jvm: Vec<Argument>,
    pub game: Vec<Argument>,
}

#[derive(Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    #[serde(alias = "type")]
    pub type_: String,
    pub file: DownloadItem,
}

impl GameVersion for VanillaVersion {
    fn id(&self) -> String {
        self.id.clone()
    }
}