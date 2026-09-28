use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to resolve staged directory destination {path:?}: {source}")]
    ResolveDestination {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect staged directory destination {path:?}: {source}")]
    InspectDestination {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("staged directory destination is a symbolic link: {0:?}")]
    DestinationIsSymlink(PathBuf),
    #[error("staged directory destination is not a directory: {0:?}")]
    DestinationIsNotDirectory(PathBuf),
    #[error("staged directory destination {0:?} must have a parent directory")]
    InvalidDestination(PathBuf),
    #[error("failed to create a temporary directory in {parent:?}: {source}")]
    CreateTemporaryDirectory {
        parent: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to move previous directory {destination:?} to backup {backup:?}: {source}")]
    Backup {
        destination: PathBuf,
        backup: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to commit staged directory to {destination:?}: {source}")]
    Commit {
        destination: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(
        "failed to commit staged directory to {destination:?}: {source}; restoring the previous directory also failed: {rollback_error}; previous directory preserved at {backup:?}"
    )]
    Rollback {
        destination: PathBuf,
        backup: PathBuf,
        #[source]
        source: io::Error,
        rollback_error: io::Error,
    },
    #[error("directory committed, but failed to remove backup directory {path:?}: {source}")]
    RemoveBackup {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
