use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {

    #[error("project operation failed for {root:?}: {source}")]
    Project {
        root: PathBuf,
        #[source]
        source: Box<parkforge_project::Error>,
    },

    #[error("failed to create temporary merged-source directory: {source}")]
    CreateMergedSources {
        #[source]
        source: std::io::Error,
    },

    #[error("failed to build game tree at {destination:?}: {source}")]
    Build {
        destination: PathBuf,
        #[source]
        source: Box<parkforge_build::Error>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
