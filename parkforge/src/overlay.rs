use std::path::{Path, PathBuf};

use parkforge_project::Project;
use parkforge_types::GameId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OverlayError {
    #[error("project operation failed for {root:?}: {source}")]
    Project {
        root: PathBuf,
        #[source]
        source: Box<parkforge_project::Error>,
    },
}

pub fn create_source_overlay(project_root: &Path, game_id: &GameId) -> Result<(), OverlayError> {
    let project = Project::open(project_root).map_err(|source| OverlayError::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    project
        .create_source_overlay(game_id)
        .map_err(|source| OverlayError::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })
}
