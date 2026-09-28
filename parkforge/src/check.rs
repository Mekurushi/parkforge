use std::path::Path;

use parkforge_build::{BuildDiagnostic, CheckReport as BuildCheckReport};
use parkforge_project::{Project, read_build_config};
use parkforge_types::{BuildConfig, GameId};
use tempfile::Builder;
use thiserror::Error;

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct ProjectCheckReport {
    revisions: Vec<RevisionCheckReport>,
}

impl ProjectCheckReport {
    #[must_use]
    pub fn revisions(&self) -> &[RevisionCheckReport] {
        &self.revisions
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.revisions.iter().all(RevisionCheckReport::is_success)
    }

    #[must_use]
    pub fn into_revisions(self) -> Vec<RevisionCheckReport> {
        self.revisions
    }
}

#[derive(Debug)]
pub struct RevisionCheckReport {
    game_id: GameId,
    result: std::result::Result<BuildCheckReport, RevisionCheckError>,
}

impl RevisionCheckReport {
    #[must_use]
    pub fn game_id(&self) -> &GameId {
        &self.game_id
    }

    #[must_use]
    pub fn result(&self) -> std::result::Result<&BuildCheckReport, &RevisionCheckError> {
        self.result.as_ref()
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.result.as_ref().is_ok_and(BuildCheckReport::is_success)
    }

    #[must_use]
    pub fn into_result(self) -> std::result::Result<BuildCheckReport, RevisionCheckError> {
        self.result
    }
}

#[derive(Debug, Error)]
pub enum RevisionCheckError {
    #[error("failed to merge project sources: {0}")]
    Merge(#[source] Box<parkforge_project::Error>),

    #[error("failed to prepare operation checks: {0}")]
    Build(#[source] Box<parkforge_build::Error>),
}

pub fn check<D>(project_root: &Path, diagnostics: D) -> Result<ProjectCheckReport>
where
    D: for<'a> FnMut(&GameId, BuildDiagnostic<'a>),
{
    let project = Project::open(project_root).map_err(|source| Error::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
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
    check_project(&project, &config, diagnostics)
}

pub fn check_with_config<D>(
    project_root: &Path,
    config: &BuildConfig,
    diagnostics: D,
) -> Result<ProjectCheckReport>
where
    D: for<'a> FnMut(&GameId, BuildDiagnostic<'a>),
{
    let project = Project::open(project_root).map_err(|source| Error::Project {
        root: project_root.to_path_buf(),
        source: Box::new(source),
    })?;
    check_project(&project, config, diagnostics)
}

fn check_project<D>(
    project: &Project,
    config: &BuildConfig,
    mut diagnostics: D,
) -> Result<ProjectCheckReport>
where
    D: for<'a> FnMut(&GameId, BuildDiagnostic<'a>),
{
    let mut revisions = Vec::new();
    for game_id in project.config().games.keys() {
        let sources = Builder::new()
            .prefix(".parkforge-merged-sources-")
            .tempdir()
            .map_err(|source| Error::CreateMergedSources { source })?;
        let result = match project.merge_sources(game_id, sources.path()) {
            Ok(()) => parkforge_build::check(sources.path(), config, |diagnostic| {
                diagnostics(game_id, diagnostic);
            })
            .map_err(|source| RevisionCheckError::Build(Box::new(source))),
            Err(source) => Err(RevisionCheckError::Merge(Box::new(source))),
        };
        revisions.push(RevisionCheckReport {
            game_id: game_id.clone(),
            result,
        });
    }

    Ok(ProjectCheckReport { revisions })
}
