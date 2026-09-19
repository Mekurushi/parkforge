use std::path::PathBuf;

use parkforge_types::GameId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read build configuration {path:?}: {source}")]
    ReadBuildConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse build configuration {path:?}: {source}")]
    ParseBuildConfig {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("merge destination {0:?} must be an existing directory")]
    MergeDestinationNotDirectory(PathBuf),

    #[error("merge destination {0:?} must be empty")]
    MergeDestinationNotEmpty(PathBuf),

    #[error("failed to inspect merge destination {path:?}: {source}")]
    InspectMergeDestination {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to copy source {path:?} to {destination:?}: {source}")]
    CopySource {
        path: PathBuf,
        destination: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("conflicting sources for logical path {0:?}")]
    SourceConflict(PathBuf),

    #[error("source root {0:?} must be a directory when present")]
    SourceNotDirectory(PathBuf),

    #[error("source symbolic links are not supported: {0:?}")]
    SourceSymlink(PathBuf),

    #[error("failed to inspect source {path:?}: {source}")]
    InspectSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to walk source root {root:?}: {source}")]
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

    #[error("original tree {0:?} must be an existing directory")]
    OriginalNotDirectory(PathBuf),

    #[error("failed to walk original tree {root:?}: {source}")]
    WalkOriginal {
        root: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    #[error("failed to resolve original directory {path:?} below {root:?}: {source}")]
    RelativeOriginalPath {
        path: PathBuf,
        root: PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },

    #[error("failed to create project gitignore {path:?}: {source}")]
    CreateGitignore {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write project gitignore {path:?}: {source}")]
    WriteGitignore {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("project root {0:?} must be an existing directory")]
    RootNotDirectory(PathBuf),

    #[error("failed to serialize project configuration {path:?}: {source}")]
    SerializeConfig {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },

    #[error("failed to create project configuration {path:?}: {source}")]
    CreateConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write project configuration {path:?}: {source}")]
    WriteConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create project directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("game ID {0} is not supported by this project")]
    UnsupportedGameId(GameId),

    #[error("failed to resolve project root {path:?}: {source}")]
    ResolveRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read project configuration {path:?}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse project configuration {path:?}: {source}")]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
