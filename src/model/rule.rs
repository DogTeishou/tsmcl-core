use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::model::Allowed;

pub type Features = HashMap<String, bool>;
pub type VersionRange = HashMap<String, String>;
pub type Rules = Vec<Rule>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Rule {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Os>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Features>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Os {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(alias = "versionRange", skip_serializing_if = "Option::is_none")]
    pub version_range: Option<VersionRange>
}

impl Allowed for Rules {
    fn allowed(&self) -> bool {
        let mut allowed = false;
        for rule in self {
            if rule.matches() {
                allowed = rule.action.eq("allow");
            }
        }
        allowed
    }
}

impl Allowed for Rule {
    fn allowed(&self) -> bool {
        self.matches() && self.action.eq("allow")
    }
}

impl Rule {
    fn matches(&self) -> bool {
        if let Some(os) = &self.os {
            if !os.allowed() {
                return false;
            }
        }
        if let Some(features) = &self.features {
            if !features.allowed() {
                return false;
            }
        }
        true
    }
}

impl Allowed for Features {
    fn allowed(&self) -> bool {
        let mut allowed = true;
        for (_, v) in self {
            if !v {
                allowed = false;
                break;
            }
        }
        allowed
    }
} 

impl Allowed for Os {
    fn allowed(&self) -> bool {
        let mut allowed = true;

        if let Some(name) = &self.name {
            #[cfg(target_os = "windows")]
            if !name.eq("windows") {
                allowed = false;
            }
            #[cfg(target_os = "macos")]
            if !name.eq("osx") {
                allowed = false;
            }
            #[cfg(target_os = "linux")]
            if !name.eq("linux") {
                allowed = false;
            }
        }

        if let Some(arch) = &self.arch {
            #[cfg(target_arch = "x86_64")]
            if !arch.eq("x86_64") {
                allowed = false;
            }
            #[cfg(target_arch = "x86")]
            if !arch.eq("x86") {
                allowed = false;
            }
            #[cfg(target_arch = "aarch64")]
            if !arch.eq("arm64") {
                allowed = false;
            }
        }

        if let Some(version) = &self.version {
            let current_version = get_os_version();
            let regex = Regex::new(version).map_err(|e| eprintln!("{e}")).unwrap();
            if !regex.is_match(&current_version) {
                allowed = false;
            }
        }

        if let Some(version_range) = &self.version_range {
            if !version_range.allowed() {
                allowed = false;
            }
        }

        allowed
    }
}

impl Allowed for VersionRange {
    fn allowed(&self) -> bool {
        let mut allowed = true;
        let current_version = version_to_u32(&get_os_version());

        for (k, v) in self {
            let expect_version = version_to_u32(v);
            if k.eq("min") {
                if current_version.0 < expect_version.0 {
                    allowed = false;
                } else if current_version.1 < expect_version.1 {
                    allowed = false;
                } else if current_version.2 < expect_version.2 {
                    allowed = false;
                }
            } else if k.eq("max") {
                if current_version.0 > expect_version.0  {
                    allowed = false;
                } else if current_version.1 > expect_version.1{
                    allowed = false;
                } else if current_version.2 > expect_version.2 {
                    allowed = false;
                }
            }
        }
        allowed
    }
}

fn version_to_u32(version: &String) -> (u32, u32, u32) {
    let splits: Vec<&str> = version.split(".").collect();
    let first = splits[0].parse::<u32>().map_err(|e| println!("{e}")).unwrap();
    let second = splits[1].parse::<u32>().map_err(|e| println!("{e}")).unwrap();
    let third = splits[2].parse::<u32>().map_err(|e| println!("{e}")).unwrap();
    (first, second, third)
}

fn get_os_version() -> String {
    os_info::get().version().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_apply_matching_actions_in_order() {
        let rules = vec![
            Rule {
                action: "allow".to_string(),
                os: None,
                features: None,
            },
            Rule {
                action: "disallow".to_string(),
                os: Some(Os {
                    name: Some("osx".to_string()),
                    arch: None,
                    version: None,
                    version_range: None,
                }),
                features: None,
            },
        ];

        #[cfg(target_os = "macos")]
        assert!(!rules.allowed());

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        assert!(rules.allowed());
    }
}