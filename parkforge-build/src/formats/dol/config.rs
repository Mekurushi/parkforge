use std::collections::BTreeMap;

use parkforge_types::{BuildConfig, BuildConfigValue};
use serde::Deserialize;

use super::error::{Error, Result, config_value_type};
use super::symbols::CustomSymbols;

pub(super) struct ConfigMappings(BTreeMap<String, ConfigMapping>);

pub(super) struct ConfigWrite {
    pub(super) address: u32,
    pub(super) data: Vec<u8>,
}

impl ConfigMappings {
    pub(super) fn parse(text: &str) -> Result<Self> {
        toml::from_str(text).map_err(|source| Error::ParseConfigs { source })
    }

    pub(super) fn resolve(
        &self,
        config: &BuildConfig,
        symbols: &CustomSymbols,
    ) -> Result<Vec<ConfigWrite>> {
        self.0
            .iter()
            .map(|(name, mapping)| {
                let value = config
                    .values()
                    .get(name)
                    .ok_or_else(|| Error::MissingConfig { name: name.clone() })?;
                let address = symbols
                    .get(name)
                    .ok_or_else(|| Error::MissingCustomSymbol { name: name.clone() })?;
                let data = mapping.encode(name, value)?;
                Ok(ConfigWrite { address, data })
            })
            .collect()
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
struct ConfigMappingsFile(BTreeMap<String, ConfigMapping>);

impl<'de> Deserialize<'de> for ConfigMappings {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        ConfigMappingsFile::deserialize(deserializer).map(|file| Self(file.0))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigMapping {
    encoding: Encoding,
    size: Option<usize>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Encoding {
    U32,
    Bool,
    String,
}

impl ConfigMapping {
    fn encode(&self, name: &str, value: &BuildConfigValue) -> Result<Vec<u8>> {
        match self.encoding {
            Encoding::U32 => {
                if self.size.is_some() {
                    return Err(Error::UnexpectedConfigSize {
                        name: name.to_owned(),
                        encoding: "u32",
                    });
                }
                let BuildConfigValue::Integer(value) = value else {
                    return Err(Error::InvalidConfigType {
                        name: name.to_owned(),
                        expected: "integer",
                        found: config_value_type(value),
                    });
                };
                let value = u32::try_from(*value).map_err(|_error| Error::NegativeU32 {
                    name: name.to_owned(),
                    value: *value,
                })?;
                Ok(value.to_be_bytes().to_vec())
            }
            Encoding::Bool => {
                if self.size.is_some() {
                    return Err(Error::UnexpectedConfigSize {
                        name: name.to_owned(),
                        encoding: "bool",
                    });
                }
                let BuildConfigValue::Boolean(value) = value else {
                    return Err(Error::InvalidConfigType {
                        name: name.to_owned(),
                        expected: "boolean",
                        found: config_value_type(value),
                    });
                };
                Ok(vec![u8::from(*value)])
            }
            Encoding::String => {
                let size = self.size.ok_or_else(|| Error::MissingStringSize {
                    name: name.to_owned(),
                })?;
                let BuildConfigValue::String(value) = value else {
                    return Err(Error::InvalidConfigType {
                        name: name.to_owned(),
                        expected: "string",
                        found: config_value_type(value),
                    });
                };
                let bytes = value.as_bytes();
                if bytes.len() > size {
                    return Err(Error::StringTooLong {
                        name: name.to_owned(),
                        length: bytes.len(),
                        size,
                    });
                }
                let mut encoded = vec![0; size];
                encoded[..bytes.len()].copy_from_slice(bytes);
                Ok(encoded)
            }
        }
    }
}
