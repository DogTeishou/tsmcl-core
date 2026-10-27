use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{Allowed, rule::Rule};

pub type Natives = HashMap<String, String>;

#[derive(Deserialize, Serialize, Clone)]
pub struct Extract {
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
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

#[derive(Deserialize, Serialize, Clone)]
pub struct Download {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<DownloadItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, DownloadItem>>,
}

#[derive(Deserialize, Serialize, Clone)]
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
            allowed = rules.allowed();
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
        let natives = self.natives.as_ref()?;

        natives.iter().find_map(|(platform, native)| {
            let matches_current_os = (platform.contains("windows") && cfg!(target_os = "windows"))
                || (platform.contains("linux") && cfg!(target_os = "linux"))
                || (platform.contains("osx") && cfg!(target_os = "macos"));

            matches_current_os.then(|| native.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_natives_selects_current_platform_from_mixed_metadata() {
        let library = Library {
            downloads: Download {
                artifact: None,
                classifiers: None,
            },
            name: "com.mojang:text2speech:1.11.3".to_string(),
            rules: None,
            natives: Some(HashMap::from([
                ("windows".to_string(), "natives-windows".to_string()),
                ("linux".to_string(), "natives-linux".to_string()),
            ])),
            extract: None,
        };

        #[cfg(target_os = "windows")]
        assert_eq!(library.get_natives().as_deref(), Some("natives-windows"));

        #[cfg(target_os = "linux")]
        assert_eq!(library.get_natives().as_deref(), Some("natives-linux"));

        #[cfg(target_os = "macos")]
        assert_eq!(library.get_natives(), None);
    }

    #[test]
    fn get_natives_returns_none_when_library_has_no_native_metadata() {
        let library = Library {
            downloads: Download {
                artifact: None,
                classifiers: None,
            },
            name: "example:library:1.0".to_string(),
            rules: None,
            natives: None,
            extract: None,
        };

        assert_eq!(library.get_natives(), None);
    }

    #[test]
    fn allowed_applies_library_rules_in_order() {
        let library = Library {
            downloads: Download {
                artifact: None,
                classifiers: None,
            },
            name: "org.lwjgl:lwjgl:3.2.2".to_string(),
            rules: Some(vec![
                Rule {
                    action: "allow".to_string(),
                    os: None,
                    features: None,
                },
                Rule {
                    action: "disallow".to_string(),
                    os: Some(crate::model::rule::Os {
                        name: Some("osx".to_string()),
                        arch: None,
                        version: None,
                        version_range: None,
                    }),
                    features: None,
                },
            ]),
            natives: None,
            extract: None,
        };

        #[cfg(target_os = "macos")]
        assert!(!library.allowed());

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        assert!(library.allowed());
    }
}