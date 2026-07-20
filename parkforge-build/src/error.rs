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
    #[error("build rule {rule:?} is invalid: {message}")]
    InvalidBuildRule { rule: String, message: String },
    #[error("source file {path:?} does not match a build rule")]
    UnmatchedBuildInput { path: PathBuf },
    #[error("source file {path:?} matches more than one build rule: {rules:?}")]
    AmbiguousBuildRule { path: PathBuf, rules: Vec<String> },
    #[error("build output {path:?} is produced by both {first:?} and {second:?}")]
    DuplicateBuildOutput {
        path: PathBuf,
        first: PathBuf,
        second: PathBuf,
    },
    #[error(
        "unsupported filesystem entry at {path:?}; only regular files and directories are allowed"
    )]
    UnsupportedFilesystemEntry { path: PathBuf },
    #[error("build rule {rule:?} requires an unknown compiler {compiler:?}")]
    CompilerUnavailable { rule: String, compiler: String },
    #[error("{compiler} compilation failed for {path:?}: {message}")]
    CompilerFailed {
        compiler: String,
        path: PathBuf,
        message: String,
    },
    #[error(transparent)]
    VirtualPath(#[from] parkforge_model::VirtualPathError),
}
