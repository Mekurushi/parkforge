use std::path::Path;

use parkforge_project::Project;
use parkforge_types::GameId;

use crate::error::{Error, Result};

pub fn create_source_overlay(project_root: &Path, game_id: &GameId) -> Result<()> {
    let project = Project::open(project_root).map_err(|source| Error::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    project
        .create_source_overlay(game_id)
        .map_err(|source| Error::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })
}
