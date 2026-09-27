use std::path::Path;

use fsc_compiler::{ConfigRequirement, ConfigType, ConfigValue, ConfigValues};
use parkforge_types::{BuildConfig, BuildConfigValue};

use super::error::{Error, Result};

pub(super) struct FscConfigMapper {
    requirements: Vec<ConfigRequirement>,
}

impl FscConfigMapper {
    pub(super) fn new(requirements: Vec<ConfigRequirement>) -> Self {
        Self { requirements }
    }

    pub(super) fn validate(&self, source: &Path, config: &BuildConfig) -> Result<()> {
        for requirement in &self.requirements {
            let _value = Self::value(source, config, requirement)?;
        }
        Ok(())
    }

    pub(super) fn map(&self, source: &Path, config: &BuildConfig) -> Result<ConfigValues> {
        let mut resolved = ConfigValues::new();
        for requirement in &self.requirements {
            let value = Self::value(source, config, requirement)?;
            let value = match (requirement.ty, value) {
                (ConfigType::Int, BuildConfigValue::Integer(value)) => ConfigValue::Int(*value),
                (ConfigType::Float, BuildConfigValue::Float(value)) => ConfigValue::Float(*value),
                (ConfigType::Bool, BuildConfigValue::Boolean(value)) => ConfigValue::Bool(*value),
                (ConfigType::String, BuildConfigValue::String(value)) => {
                    ConfigValue::String(value.clone())
                }
                _ => unreachable!("configuration value was validated before mapping"),
            };
            drop(resolved.insert(requirement.name.clone(), value));
        }
        Ok(resolved)
    }

    fn value<'a>(
        source: &Path,
        config: &'a BuildConfig,
        requirement: &ConfigRequirement,
    ) -> Result<&'a BuildConfigValue> {
        let value = config.values().get(&requirement.name).ok_or_else(|| {
            Error::missing_config(
                source.to_path_buf(),
                requirement.name.clone(),
                requirement.ty,
            )
        })?;
        if matches!(
            (requirement.ty, value),
            (ConfigType::Int, BuildConfigValue::Integer(_))
                | (ConfigType::Float, BuildConfigValue::Float(_))
                | (ConfigType::Bool, BuildConfigValue::Boolean(_))
                | (ConfigType::String, BuildConfigValue::String(_))
        ) {
            Ok(value)
        } else {
            Err(Error::invalid_config_type(
                source.to_path_buf(),
                requirement.name.clone(),
                requirement.ty,
                value,
            ))
        }
    }
}
