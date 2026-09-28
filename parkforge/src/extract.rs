use std::path::Path;

use parkforge_game_tree::{ExtractionProgress, extract_game_tree};
use parkforge_project::Project;
use parkforge_types::GameId;

use crate::error::{Error, Result};
use crate::identify::identify;

#[derive(Debug, Clone, Copy)]
pub struct ExtractionPaths<'a> {
    pub input_iso: &'a Path,
    /// game tree is extracted directly
    /// does not append another game-ID directory
    /// replaces existing output
    /// in the project convention it would be `original/<game-id>`
    pub destination: &'a Path,
}

pub fn extract<F>(project_root: &Path, input_iso: &Path, progress: F) -> Result<GameId>
where
    F: FnMut(ExtractionProgress),
{
    let game_id = identify(input_iso)?;
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
    extract_to_with_game_id(
        ExtractionPaths {
            input_iso,
            destination: &destination,
        },
        game_id,
        progress,
    )
}

pub fn extract_to<F>(paths: ExtractionPaths<'_>, progress: F) -> Result<GameId>
where
    F: FnMut(ExtractionProgress),
{
    let game_id = identify(paths.input_iso)?;
    extract_to_with_game_id(paths, game_id, progress)
}

fn extract_to_with_game_id<F>(
    paths: ExtractionPaths<'_>,
    game_id: GameId,
    progress: F,
) -> Result<GameId>
where
    F: FnMut(ExtractionProgress),
{
    extract_game_tree(paths.input_iso, paths.destination, progress).map_err(|source| {
        Error::Extract {
            input: paths.input_iso.to_path_buf(),
            destination: paths.destination.to_path_buf(),
            source: Box::new(source),
        }
    })?;
    Ok(game_id)
}
