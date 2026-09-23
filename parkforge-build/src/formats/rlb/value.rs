use std::path::Path;

use parkforge_types::{BuildConfig, BuildConfigValue};
use rlb_domain::Value;
use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum SourceValue {
    Value(ValueAssignment),
    Config(ConfigAssignment),
    Null(NullAssignment),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValueAssignment {
    value: Literal,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConfigAssignment {
    config: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NullAssignment {
    null: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Literal {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

pub(super) enum ResolveError {
    MissingConfig(String),
    InvalidNull,
    InvalidInteger(i64),
    InvalidFloat(f64),
}

impl SourceValue {
    pub(super) fn resolve(&self, config: &BuildConfig) -> Result<Value, ResolveError> {
        match self {
            Self::Value(assignment) => assignment.value.resolve(),
            Self::Config(assignment) => {
                let value = config
                    .values()
                    .get(&assignment.config)
                    .ok_or_else(|| ResolveError::MissingConfig(assignment.config.clone()))?;
                resolve_config(value)
            }
            Self::Null(NullAssignment { null: true }) => Ok(Value::String(None)),
            Self::Null(NullAssignment { null: false }) => Err(ResolveError::InvalidNull),
        }
    }
}

impl Literal {
    fn resolve(&self) -> Result<Value, ResolveError> {
        match self {
            Self::Integer(value) => u32::try_from(*value)
                .map(Value::Integer)
                .map_err(|_error| ResolveError::InvalidInteger(*value)),
            Self::Float(value) => {
                #[allow(clippy::cast_possible_truncation)]
                let converted = *value as f32;
                if converted.is_finite() {
                    Ok(Value::Float(converted))
                } else {
                    Err(ResolveError::InvalidFloat(*value))
                }
            }
            Self::Boolean(value) => Ok(Value::Boolean(*value)),
            Self::String(value) => Ok(Value::String(Some(value.clone()))),
        }
    }
}

//TODO: cleanup with error restructure
impl ResolveError {
    pub(super) fn into_build_error(
        self,
        path: &Path,
        table: &str,
        row: usize,
        field: &str,
    ) -> Error {
        match self {
            Self::MissingConfig(name) => Error::MissingRlbConfig {
                path: path.to_path_buf(),
                table: table.to_owned(),
                row,
                field: field.to_owned(),
                name,
            },
            Self::InvalidNull => Error::InvalidRlbAssignment {
                path: path.to_path_buf(),
                table: table.to_owned(),
                row,
                field: field.to_owned(),
                message: "null must be true",
            },
            Self::InvalidInteger(value) => Error::InvalidRlbInteger {
                path: path.to_path_buf(),
                table: table.to_owned(),
                row,
                field: field.to_owned(),
                value,
            },
            Self::InvalidFloat(value) => Error::InvalidRlbFloat {
                path: path.to_path_buf(),
                table: table.to_owned(),
                row,
                field: field.to_owned(),
                value,
            },
        }
    }
}

fn resolve_config(value: &BuildConfigValue) -> Result<Value, ResolveError> {
    match value {
        BuildConfigValue::Integer(value) => u32::try_from(*value)
            .map(Value::Integer)
            .map_err(|_error| ResolveError::InvalidInteger(i64::from(*value))),
        BuildConfigValue::Float(value) => Ok(Value::Float(*value)),
        BuildConfigValue::Boolean(value) => Ok(Value::Boolean(*value)),
        BuildConfigValue::String(value) => Ok(Value::String(Some(value.clone()))),
    }
}
