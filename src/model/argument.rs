use serde::{Deserialize, Serialize};
use crate::model::Allowed;
use crate::model::rule::Rule;

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Conditional(ConditionalArgument),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConditionalArgument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    pub value: ArgumentValue,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

impl Argument {
    pub fn set_all_features_false(&mut self) {
        match self {
            Argument::Plain(_) => (),
            Argument::Conditional(conditional_argument) => {
                if let Some(rules) = &mut conditional_argument.rules {
                    for rule in rules {
                        if let Some(features) = &mut rule.features {
                            for (_, v) in features {
                                *v = false;
                            }
                        }
                    }
                }
            },
        };
    }
}

impl Allowed for ConditionalArgument {
    fn allowed(&self) -> bool {
        let mut allowed = false;
        if let Some(rules) = &self.rules {
            if rules.allowed() {
                allowed = true;
            }
        }
        allowed
    }
}