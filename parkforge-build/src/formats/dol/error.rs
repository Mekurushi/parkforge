use std::path::PathBuf;

use parkforge_types::BuildConfigValue;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read DOL patch directory {path:?}: {source}")]
    ReadPatchDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to inspect DOL patch source {path:?}: {source}")]
    InspectPatchSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read DOL patch source {path:?}: {source}")]
    ReadPatchSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read DOL {path:?}: {source}")]
    ReadDol {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write DOL {path:?}: {source}")]
    WriteDol {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read a 32-bit value at file offset {offset:#x}")]
    ReadU32 { offset: usize },

    #[error("DOL reserved header word at file offset {offset:#x} is nonzero: {value:#010x}")]
    NonzeroReservedHeader { offset: usize, value: u32 },

    #[error("RAM address {address:#010x} is not contained in a DOL section")]
    UnmappedAddress { address: u32 },

    #[error("file offset {offset:#x} is not contained in a DOL section")]
    UnmappedOffset { offset: u32 },

    #[error("DOL section index {index} is out of range")]
    InvalidSectionIndex { index: usize },

    #[error("DOL file offset {offset:#x} cannot be represented on this platform")]
    InvalidFileOffset { offset: u32 },

    #[error("DOL write at file offset {offset:#x} is too large")]
    WriteTooLarge { offset: usize },

    #[error(
        "failed to read {length} bytes at file offset {offset:#x}: DOL length is {file_length}"
    )]
    ReadOutsideFile {
        offset: usize,
        length: usize,
        file_length: usize,
    },

    #[error("having multiple separate free space directives for main.dol is not supported")]
    MultipleFreeSpaceDirectives,

    #[error("DOL patch length {length} does not fit in a 32-bit section size")]
    PatchLengthTooLarge { length: usize },

    #[error("DOL free space pointer calculation overflowed")]
    FreeSpacePointerOverflow,

    #[error("failed to parse DOL patch artifact: {source}")]
    ParseArtifact {
        #[source]
        source: serde_yaml_ng::Error,
    },

    #[error("failed to parse DOL patch metadata: {source}")]
    ParseMetadata {
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to parse DOL patch configuration mappings: {source}")]
    ParseConfigs {
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to parse DOL custom symbols: {source}")]
    ParseCustomSymbols {
        #[source]
        source: serde_yaml_ng::Error,
    },

    #[error("DOL patch configuration mapping {name:?} requires a missing configuration value")]
    MissingConfig { name: String },

    #[error("DOL patch configuration mapping {name:?} requires a missing custom symbol")]
    MissingCustomSymbol { name: String },

    #[error(
        "DOL patch configuration mapping {name:?} requires a value of type {expected}, but its configured type is {found}"
    )]
    InvalidConfigType {
        name: String,
        expected: &'static str,
        found: &'static str,
    },

    #[error(
        "DOL patch configuration mapping {name:?} cannot encode negative integer {value} as u32"
    )]
    NegativeU32 { name: String, value: i32 },

    #[error("DOL string configuration mapping {name:?} requires a size")]
    MissingStringSize { name: String },

    #[error("DOL {encoding} configuration mapping {name:?} must not specify a size")]
    UnexpectedConfigSize {
        name: String,
        encoding: &'static str,
    },

    #[error(
        "DOL string configuration mapping {name:?} encodes to {length} bytes, exceeding its size of {size}"
    )]
    StringTooLong {
        name: String,
        length: usize,
        size: usize,
    },
}

pub(super) const fn config_value_type(value: &BuildConfigValue) -> &'static str {
    match value {
        BuildConfigValue::Integer(_) => "integer",
        BuildConfigValue::Float(_) => "float",
        BuildConfigValue::Boolean(_) => "boolean",
        BuildConfigValue::String(_) => "string",
    }
}
