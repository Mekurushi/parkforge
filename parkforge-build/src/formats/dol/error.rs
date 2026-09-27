use std::path::PathBuf;

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
}
