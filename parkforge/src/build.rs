use std::path::Path;

use parkforge_build::{BuildDiagnostic, BuildProgress};
use parkforge_project::{Project, merge_sources, read_build_config};
use parkforge_types::{BuildConfig, GameId};
use tempfile::Builder;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub struct BuildPaths<'a> {
    /// expects game tree directly at root
    /// in the project convention it would be `original/<game-id>`
    pub original: &'a Path,
    /// shared part of the src path
    /// in the project convention it would be `src/shared`
    pub shared_sources: &'a Path,
    /// game Id resolved part of the src path
    /// in the project convention it would be `src/<game-id>`
    pub revision_sources: &'a Path,
    /// puts the game tree directly to the destination
    /// does not append another game-ID directory
    /// replaces existing output
    /// in the project convention it would be `build/<game-id>`
    pub destination: &'a Path,
}

pub fn build<P, D>(project_root: &Path, game_id: &GameId, progress: P, diagnostics: D) -> Result<()>
where
    P: FnMut(BuildProgress),
    D: for<'a> FnMut(BuildDiagnostic<'a>),
{
    let project = Project::open(project_root).map_err(|source| Error::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    // TODO: multiple sources type input (path override, typed structure, default project path)
    let config = match read_build_config(&project.build_config_path()) {
        Ok(config) => config,
        Err(parkforge_project::Error::ReadBuildConfig { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            BuildConfig::default()
        }
        Err(source) => {
            return Err(Error::Project {
                root: project.root().to_path_buf(),
                source: Box::new(source),
            });
        }
    };
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
    let shared_sources = project.shared_sources();
    let revision_sources = project
        .sources_for(game_id)
        .map_err(|source| Error::Project {
            root: project.root().to_path_buf(),
            source: Box::new(source),
        })?;
    build_with_paths(
        BuildPaths {
            original: &original,
            shared_sources: &shared_sources,
            revision_sources: &revision_sources,
            destination: &destination,
        },
        config,
        progress,
        diagnostics,
    )
}

pub fn build_with_paths<P, D>(
    paths: BuildPaths<'_>,
    config: BuildConfig,
    progress: P,
    diagnostics: D,
) -> Result<()>
where
    P: FnMut(BuildProgress),
    D: for<'a> FnMut(BuildDiagnostic<'a>),
{
    let sources = Builder::new()
        .prefix(".parkforge-merged-sources-")
        .tempdir()
        .map_err(|source| Error::CreateMergedSources { source })?;
    merge_sources(paths.shared_sources, paths.revision_sources, sources.path()).map_err(
        |source| Error::MergeSources {
            shared: paths.shared_sources.to_path_buf(),
            revision: paths.revision_sources.to_path_buf(),
            source: Box::new(source),
        },
    )?;
    parkforge_build::build(
        paths.original,
        sources.path(),
        paths.destination,
        config,
        progress,
        diagnostics,
    )
    .map_err(|source| Error::Build {
        destination: paths.destination.to_path_buf(),
        source: Box::new(source),
    })
}
