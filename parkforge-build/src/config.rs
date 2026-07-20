use std::collections::HashSet;

use globset::Glob;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(rename = "rule", default)]
    pub rules: Vec<BuildRule>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            rules: vec![BuildRule {
                name: "fsc".to_owned(),
                source: "**/*.fsc".to_owned(),
                compiler: "fsc".to_owned(),
                output_extension: Some("fsb".to_owned()),
            }],
        }
    }
}

impl BuildConfig {
    pub fn validate(&self) -> Result<()> {
        let mut names = HashSet::new();
        for rule in &self.rules {
            if rule.name.is_empty() {
                return Err(Error::InvalidBuildRule {
                    rule: rule.name.clone(),
                    message: "name must not be empty".to_owned(),
                });
            }
            if !names.insert(&rule.name) {
                return Err(Error::InvalidBuildRule {
                    rule: rule.name.clone(),
                    message: "rule names must be unique".to_owned(),
                });
            }
            Glob::new(&rule.source).map_err(|error| Error::InvalidBuildRule {
                rule: rule.name.clone(),
                message: format!("invalid source pattern: {error}"),
            })?;
            if rule.compiler.is_empty() {
                return Err(Error::InvalidBuildRule {
                    rule: rule.name.clone(),
                    message: "compiler must not be empty".to_owned(),
                });
            }
            if rule.output_extension.as_ref().is_some_and(|extension| {
                extension.is_empty() || extension.contains(['/', '\\', '.'])
            }) {
                return Err(Error::InvalidBuildRule {
                    rule: rule.name.clone(),
                    message: "output_extension must contain an extension without a leading dot"
                        .to_owned(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRule {
    pub name: String,
    pub source: String,
    pub compiler: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_extension: Option<String>,
}
