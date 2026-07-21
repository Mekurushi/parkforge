use std::io;
use std::path::PathBuf;

use thiserror::Error;

use parkforge_model::GameId;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Build(#[from] parkforge_build::Error),
    #[error(transparent)]
    GameId(#[from] parkforge_model::GameIdError),
    #[error(transparent)]
    VirtualPath(#[from] parkforge_model::VirtualPathError),
    #[error("I/O error while accessing {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse {path:?}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("{path:?} is not a project: no project.toml found")]
    NotAProject { path: PathBuf },
    #[error("a project already exists at {path:?}")]
    ProjectAlreadyExists { path: PathBuf },
    #[error("game ID {game_id} appears more than once in project.toml")]
    DuplicateGameId { game_id: GameId },
    #[error("game ID {game_id} is already registered in project.toml")]
    GameAlreadyRegistered { game_id: GameId },
    #[error("game ID {game_id} is not registered in project.toml")]
    GameNotRegistered { game_id: GameId },
    #[error("{path:?} exists but game ID {game_id} is not registered in project.toml")]
    UnregisteredGameDirectory { game_id: GameId, path: PathBuf },
    #[error("game ID {game_id} has no extracted original directory at {path:?}")]
    MissingOriginalDirectory { game_id: GameId, path: PathBuf },
    #[error("game ID {game_id} has no build directory at {path:?}")]
    MissingBuildDirectory { game_id: GameId, path: PathBuf },
    #[error("path {path:?} contains non-UTF-8 components")]
    InvalidUtf8Path { path: PathBuf },
}
