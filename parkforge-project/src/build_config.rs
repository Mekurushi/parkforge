use std::fs;
use std::path::Path;

use parkforge_types::{BuildConfig, BuildConfigValue};
use serde::{Deserialize, Deserializer, de};

use crate::error::{Error, Result};

struct BuildConfigFile(BuildConfig);

impl<'de> Deserialize<'de> for BuildConfigFile {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values: std::collections::BTreeMap<String, BuildConfigValue> =
            std::collections::BTreeMap::<String, toml::Value>::deserialize(deserializer)?
            .into_iter()
            .map(|(name, value)| {
                let value = match value {
                    toml::Value::Integer(value) => {
                        let value = i32::try_from(value).map_err(|_| {
                            de::Error::custom(format_args!(
                                "configuration value {name:?} is {value}, but FSC integers must be between {} and {}",
                                i32::MIN,
                                i32::MAX
                            ))
                        })?;
                        BuildConfigValue::Integer(value)
                    }
                    toml::Value::Float(value) => {
                        BuildConfigValue::Float(to_fsc_float(&name, value).map_err(de::Error::custom)?)
                    }
                    toml::Value::Boolean(value) => BuildConfigValue::Boolean(value),
                    toml::Value::String(value) => BuildConfigValue::String(value),
                    _ => {
                        return Err(de::Error::custom(format!(
                            "configuration value {name:?} must be an integer, float, boolean, or string"
                        )));
                    }
                };
                Ok((name, value))
            })
            .collect::<std::result::Result<_, D::Error>>()?;
        Ok(Self(values.into_iter().collect()))
    }
}

pub fn read_build_config(path: &Path) -> Result<BuildConfig> {
    let contents = fs::read_to_string(path).map_err(|source| Error::ReadBuildConfig {
        path: path.to_path_buf(),
        source,
    })?;
    let file: BuildConfigFile =
        toml::from_str(&contents).map_err(|source| Error::ParseBuildConfig {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(file.0)
}

fn to_fsc_float(name: &str, value: f64) -> std::result::Result<f32, String> {
    #[allow(clippy::cast_possible_truncation)]
    let converted = value as f32;
    if converted.is_finite() {
        Ok(converted)
    } else {
        Err(format!(
            "configuration value {name:?} must be a finite FSC float"
        ))
    }
}
