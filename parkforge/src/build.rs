use std::path::Path;

use parkforge_build::{BuildDiagnostic, BuildProgress};
use parkforge_project::Project;
use parkforge_types::GameId;
use tempfile::Builder;

use crate::error::{Error, Result};

pub fn build<P, D>(project_root: &Path, game_id: &GameId, progress: P, diagnostics: D) -> Result<()>
where
    P: FnMut(BuildProgress),
    D: for<'a> FnMut(BuildDiagnostic<'a>),
{
    let project = Project::open(project_root).map_err(|source| Error::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    let original = project
        .original_for(game_id)
        .map_err(|source| Error::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })?;
    let destination = project
        .build_for(game_id)
        .map_err(|source| Error::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })?;
    let sources = Builder::new()
        .prefix(".parkforge-merged-sources-")
        .tempdir()
        .map_err(|source| Error::CreateMergedSources { source })?;
    project
        .merge_sources(game_id, sources.path())
        .map_err(|source| Error::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })?;
    parkforge_build::build(
        &original,
        sources.path(),
        &destination,
        progress,
        diagnostics,
    )
    .map_err(|source| Error::Build {
        destination,
        source: Box::new(source),
    })
}
