use std::path::PathBuf;

use thiserror::Error;

use crate::formats::dol::Error as DolError;
use crate::formats::fsb::Error as FsbError;
use crate::formats::rlb::Error as RlbError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to stage build directory: {0}")]
    Staging(#[from] parkforge_staging::Error),

    #[error(transparent)]
    Dol(#[from] DolError),

    #[error(transparent)]
    Fsb(#[from] FsbError),

    #[error(transparent)]
    Rlb(#[from] RlbError),

    #[error("sources {first:?} and {second:?} both target {target:?}")]
    ConflictingTarget {
        target: PathBuf,
        first: PathBuf,
        second: PathBuf,
    },

    #[error("failed to walk source tree {root:?}: {source}")]
    WalkSources {
        root: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    #[error("failed to resolve source {path:?} below {root:?}: {source}")]
    RelativeSourcePath {
        path: PathBuf,
        root: PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },

    #[error("unsupported source format: {0:?}")]
    UnsupportedSourceFormat(PathBuf),

    #[error("patch directory {path:?} must not be nested inside {outer:?}")]
    NestedPatchDirectory { path: PathBuf, outer: PathBuf },

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

    #[error("build destination {destination:?} must not overlap input tree {input:?}")]
    OverlappingTrees {
        input: PathBuf,
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
}

pub type Result<T> = std::result::Result<T, Error>;
