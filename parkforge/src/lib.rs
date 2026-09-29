mod build;
mod check;
mod error;
mod extract;
mod identify;
mod overlay;
mod rebuild;

pub use build::{BuildPaths, build, build_with_paths};
pub use check::{
    ProjectCheckReport, RevisionCheckError, RevisionCheckReport, check, check_with_config,
};
pub use error::{Error, Result};
pub use extract::{ExtractionPaths, extract, extract_to};
pub use identify::identify;
pub use overlay::create_source_overlay;
pub use parkforge_build::{
    BuildDiagnostic, BuildProgress, DiagnosticLabel, DiagnosticLabelStyle, DiagnosticSeverity,
};
pub use parkforge_game_tree::{ExtractionProgress, RebuildProgress};
pub use parkforge_project::{
    Error as ProjectError, ProjectConfig, Result as ProjectResult, read_project_config,
};
pub use parkforge_types::{BuildConfig, BuildConfigValue, GameId, GameIdError};
pub use rebuild::{RebuildPaths, rebuild, rebuild_from};
