use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error while accessing {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse {path:?}: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("binrw error at {path:?}: {source}")]
    BinRw { path: PathBuf, source: binrw::Error },

    #[error("section {0} already added")]
    DuplicateSection(String),

    #[error("unknown section {0}")]
    UnknownSection(String),

    #[error("section {0} doesn't exist")]
    MissingSection(String),

    #[error("cannot open partition: {0}")]
    OpenPartition(String),

    #[error("failed to read certificate chain: {0}")]
    ReadCertificates(String),

    #[error("failed to read TMD: {0}")]
    ReadTmd(String),

    #[error("failed to walk directory {path:?}: {message}")]
    WalkDir { path: PathBuf, message: String },
    #[error("failed to decompress {path:?}: {message}")]
    Decompress { path: PathBuf, message: String },
    #[error("failed to read archive {path:?}: {message}")]
    Archive { path: PathBuf, message: String },
    #[error("path {path:?} contains non-UTF-8 components")]
    InvalidUtf8Path { path: PathBuf },
    #[error("path {path:?} is not inside workspace root {root:?}")]
    OutsideWorkspace { path: PathBuf, root: PathBuf },
    #[error(transparent)]
    VirtualPath(#[from] parkforge_model::VirtualPathError),
    #[error("archive entry {entry:?} in {path:?} is not a valid relative path: {reason}")]
    InvalidArchiveEntryPath {
        path: PathBuf,
        entry: String,
        reason: &'static str,
    },
    #[error("game ID read from {path:?} is invalid: {reason}")]
    InvalidGameId { path: PathBuf, reason: &'static str },
    #[error("extraction target already exists at {path:?}")]
    ExtractionTargetExists { path: PathBuf },
    #[error("could not create a unique extraction staging directory under {path:?}")]
    StagingDirectoryUnavailable { path: PathBuf },
    #[error("manifest file {virtual_path:?} references unknown container {container}")]
    UnknownContainer {
        virtual_path: String,
        container: u32,
    },
    #[error("manifest is inconsistent for container {virtual_path:?}: {message}")]
    InvalidManifest {
        virtual_path: String,
        message: String,
    },
    #[error("{value:?} is not a valid hash")]
    InvalidHash { value: String },
    #[error("unsupported hash algorithm {algorithm:?}")]
    UnsupportedHashAlgorithm { algorithm: String },
    #[error("value is too large to represent in the manifest")]
    ManifestSizeOverflow,
    #[error("failed to rebuild ISO from {path:?}: {message}")]
    Rebuild { path: PathBuf, message: String },
    #[error("archive {archive:?} contains unsupported empty directory {directory:?}")]
    EmptyArchiveDirectory {
        archive: PathBuf,
        directory: PathBuf,
    },
}
