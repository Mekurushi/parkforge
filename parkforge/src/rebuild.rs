use std::path::Path;

use parkforge_game_tree::{RebuildProgress, rebuild_game_tree};
use parkforge_project::Project;
use parkforge_types::GameId;

use crate::error::{Error, Result};

pub fn rebuild<F>(
    project_root: &Path,
    game_id: &GameId,
    output_iso: &Path,
    progress: F,
) -> Result<()>
where
    F: FnMut(RebuildProgress),
{
    let project = Project::open(project_root).map_err(|source| Error::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    let build = project
        .build_for(game_id)
        .map_err(|source| Error::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })?;
    rebuild_game_tree(&build, output_iso, progress).map_err(|source| Error::Rebuild {
        output: output_iso.to_path_buf(),
        source: Box::new(source),
    })
}
