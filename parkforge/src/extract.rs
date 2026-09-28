use std::path::Path;

use parkforge_game_tree::{ExtractionProgress, extract_game_tree};
use parkforge_project::Project;
use parkforge_types::GameId;
use parkforge_wii_disc::read_game_id;

use crate::error::{Error, Result};

pub fn extract<F>(project_root: &Path, input_iso: &Path, progress: F) -> Result<GameId>
where
    F: FnMut(ExtractionProgress),
{
    let game_id = read_game_id(input_iso).map_err(|source| Error::ReadGameId {
        input: input_iso.to_path_buf(),
        source: Box::new(source),
    })?;
    let project = Project::open(project_root).map_err(|source| Error::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    let destination = project
        .original_for(&game_id)
        .map_err(|source| Error::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })?;
    extract_game_tree(input_iso, &destination, progress).map_err(|source| Error::Extract {
        input: input_iso.to_path_buf(),
        destination,
        source: Box::new(source),
    })?;
    Ok(game_id)
}
