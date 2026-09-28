use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read game ID from ISO {input:?}: {source}")]
    ReadGameId {
        input: PathBuf,
        #[source]
        source: Box<parkforge_wii_disc::Error>,
    },

    #[error("failed to extract ISO {input:?} to {destination:?}: {source}")]
    Extract {
        input: PathBuf,
        destination: PathBuf,
        #[source]
        source: Box<parkforge_game_tree::Error>,
    },

    #[error("failed to rebuild ISO at {output:?}: {source}")]
    Rebuild {
        output: PathBuf,
        #[source]
        source: Box<parkforge_game_tree::Error>,
    },

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

    #[error(
        "failed to merge shared sources {shared:?} and revision sources {revision:?}: {source}"
    )]
    MergeSources {
        shared: PathBuf,
        revision: PathBuf,
        #[source]
        source: Box<parkforge_project::Error>,
    },

    #[error("failed to build game tree at {destination:?}: {source}")]
    Build {
        destination: PathBuf,
        #[source]
        source: Box<parkforge_build::Error>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
