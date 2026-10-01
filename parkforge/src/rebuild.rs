use std::path::{Path, PathBuf};

use parkforge_game_tree::{RebuildProgress, rebuild_game_tree};
use parkforge_project::Project;
use parkforge_types::GameId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RebuildError {
    #[error("project operation failed for {root:?}: {source}")]
    Project {
        root: PathBuf,
        #[source]
        source: Box<parkforge_project::Error>,
    },

    #[error("failed to rebuild ISO at {output:?}: {source}")]
    Rebuild {
        output: PathBuf,
        #[source]
        source: Box<parkforge_game_tree::Error>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct RebuildPaths<'a> {
    /// expects the game tree at root
    /// in the project convention it would be `build/<game-id>`
    pub source: &'a Path,
    /// creates iso at path
    /// replaces existing output
    pub destination_iso: &'a Path,
}

pub fn rebuild<F>(
    project_root: &Path,
    game_id: &GameId,
    output_iso: &Path,
    progress: F,
) -> Result<(), RebuildError>
where
    F: FnMut(RebuildProgress),
{
    let project = Project::open(project_root).map_err(|source| RebuildError::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    let build = project
        .build_for(game_id)
        .map_err(|source| RebuildError::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })?;
    rebuild_from(
        RebuildPaths {
            source: &build,
            destination_iso: output_iso,
        },
        progress,
    )
}

pub fn rebuild_from<F>(paths: RebuildPaths<'_>, progress: F) -> Result<(), RebuildError>
where
    F: FnMut(RebuildProgress),
{
    rebuild_game_tree(paths.source, paths.destination_iso, progress).map_err(|source| {
        RebuildError::Rebuild {
            output: paths.destination_iso.to_path_buf(),
            source: Box::new(source),
        }
    })
}
