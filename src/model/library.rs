use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{Allowed, rule::Rule};

pub type Natives = HashMap<String, String>;

#[derive(Deserialize, Serialize)]
pub struct Extract {
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Library {
    pub downloads: Download,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<Natives>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<Extract>
}

#[derive(Deserialize, Serialize)]
pub struct Download {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<DownloadItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, DownloadItem>>,
}

#[derive(Deserialize, Serialize)]
pub struct DownloadItem {
    #[serde(default, alias = "id")]
    pub path: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

impl Allowed for Library {
    fn allowed(&self) -> bool {
        let mut allowed = true;
        if let Some(rules) = &self.rules {
            for rule in rules {
                if !rule.allowed() {
                    allowed = false;
                    break;
                }
            }
        }

        let natives = self.get_natives();
        if let Some(native) = natives {
            if native.eq("windows") && !cfg!(target_os = "windows") {
                allowed = false;
            } else if native.eq("osx") && !cfg!(target_os = "macos") {
                allowed = false;
            } else if native.eq("linux") && !cfg!(target_os = "linux") {
                allowed = false;
            }
        }

        if self.name.contains("native") {
            if self.name.contains("x86") && !cfg!(target_arch = "x86") {
                return false;
            } else if self.name.contains("arm64") && !cfg!(target_arch = "aarch64") {
                return false;
            } else if !cfg!(target_arch = "x86_64") {
                return false;
            }
        }
        
        allowed
    }
}

impl Library {
    pub fn get_natives(&self) -> Option<String> {
        let mut native = Some(String::new());
        if let Some(natives) = &self.natives {
            for (platform, n) in natives {
                if platform.eq("windows") && cfg!(target_os = "windows") {
                    native = Some(n.clone());
                } else if platform.eq("linux") && cfg!(target_os = "linux") {
                    native = Some(n.clone());
                } else if platform.eq("osx") && cfg!(target_os = "macos") {
                    native = Some(n.clone());
                } else {
                    native = None;
                }
            }
        }
        native
    }
}