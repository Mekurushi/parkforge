use std::path::PathBuf;

use fsc_compiler::ConfigType;
use parkforge_types::BuildConfigValue;
use thiserror::Error;

pub(super) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported source format: {0:?}")]
    UnsupportedSourceFormat(PathBuf),

    #[error("failed to read FSB {path:?}: {source}")]
    ReadFsb {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read symbols {path:?}: {source}")]
    ReadSymbols {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse symbols {path:?}: {source}")]
    ParseSymbols {
        path: PathBuf,
        #[source]
        source: fsc_patcher::SymbolTableParseError,
    },

    #[error("failed to apply FSC patch {path:?} to FSB {target:?}: {source}")]
    PatchFsb {
        path: PathBuf,
        target: PathBuf,
        #[source]
        source: fsc_patcher::PatchFailure,
    },

    #[error("FSC source {0:?} must have a valid UTF-8 script name")]
    InvalidScriptName(PathBuf),

    #[error("failed to read FSC source {path:?}: {source}")]
    ReadFscSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to compile FSC source {path:?}")]
    CompileFsc {
        path: PathBuf,
        failure: fsc_compiler::CompileFailure,
    },

    #[error("FSC source {path:?} requires configuration value {name:?} of type {expected}")]
    MissingFscConfig {
        path: PathBuf,
        name: String,
        expected: &'static str,
    },

    #[error(
        "FSC source {path:?} requires configuration value {name:?} of type {expected}, but its configured type is {found}"
    )]
    InvalidFscConfigType {
        path: PathBuf,
        name: String,
        expected: &'static str,
        found: &'static str,
    },

    #[error("failed to write FSB {path:?}: {source}")]
    WriteFsb {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    pub(super) fn missing_config(path: PathBuf, name: String, expected: ConfigType) -> Self {
        Self::MissingFscConfig {
            path,
            name,
            expected: config_type_name(expected),
        }
    }

    pub(super) fn invalid_config_type(
        path: PathBuf,
        name: String,
        expected: ConfigType,
        found: &BuildConfigValue,
    ) -> Self {
        Self::InvalidFscConfigType {
            path,
            name,
            expected: config_type_name(expected),
            found: value_type_name(found),
        }
    }
}

const fn config_type_name(config_type: ConfigType) -> &'static str {
    match config_type {
        ConfigType::Int => "int",
        ConfigType::Float => "float",
        ConfigType::Bool => "bool",
        ConfigType::String => "string",
    }
}

const fn value_type_name(value: &BuildConfigValue) -> &'static str {
    match value {
        BuildConfigValue::Integer(_) => "int",
        BuildConfigValue::Float(_) => "float",
        BuildConfigValue::Boolean(_) => "bool",
        BuildConfigValue::String(_) => "string",
    }
}
