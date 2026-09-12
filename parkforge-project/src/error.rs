use std::path::PathBuf;

use parkforge_types::GameId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
