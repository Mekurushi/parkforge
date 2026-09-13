use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to resolve build path {path:?}: {source}")]
    ResolvePath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to inspect build path {path:?}: {source}")]
    InspectPath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("build path {0:?} must be an existing directory")]
    NotDirectory(PathBuf),

    #[error("symbolic links are not supported: {0:?}")]
    SymbolicLink(PathBuf),

    #[error("build destination {0:?} must name a directory below an existing parent")]
    InvalidDestination(PathBuf),

    #[error("original {original:?} and build destination {destination:?} must not overlap")]
    OverlappingTrees {
        original: PathBuf,
        destination: PathBuf,
    },

    #[error("failed to walk original tree {root:?}: {source}")]
    WalkOriginal {
        root: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    #[error("failed to resolve entry {path:?} below original {root:?}: {source}")]
    RelativeEntry {
        path: PathBuf,
        root: PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },

    #[error("unsupported original entry type: {0:?}")]
    UnsupportedEntry(PathBuf),

    #[error("failed to create staged directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to copy {input:?} to {output:?}: {source}")]
    CopyFile {
        input: PathBuf,
        output: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create temporary build directory in {parent:?}: {source}")]
    CreateTemporaryDirectory {
        parent: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to move previous build {destination:?} to backup {backup:?}: {source}")]
    BackupBuild {
        destination: PathBuf,
        backup: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to commit staged build to {destination:?}: {source}")]
    CommitBuild {
        destination: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "failed to commit build to {destination:?}: {source}; restoring the previous build also failed: {rollback_error}; previous build preserved at {backup:?}"
    )]
    RollbackBuild {
        destination: PathBuf,
        backup: PathBuf,
        #[source]
        source: std::io::Error,
        rollback_error: std::io::Error,
    },

    #[error("build committed, but failed to remove backup directory {path:?}: {source}")]
    RemoveBackup {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
