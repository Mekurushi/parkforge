mod build;
mod check;
mod extract;
mod identify;
mod overlay;
mod rebuild;

pub use build::{BuildError, BuildPaths, build, build_with_paths};
pub use check::{
    CheckError, ProjectCheckReport, RevisionCheckError, RevisionCheckReport, check,
    check_with_config,
};
pub use extract::{ExtractError, ExtractionPaths, extract, extract_to};
pub use identify::{IdentifyError, identify};
pub use overlay::{OverlayError, create_source_overlay};
pub use parkforge_build::{
    BuildDiagnostic, BuildProgress, DiagnosticLabel, DiagnosticLabelStyle, DiagnosticSeverity,
};
pub use parkforge_game_tree::{ExtractionProgress, RebuildProgress};
pub use parkforge_project::{
    Error as ProjectError, ProjectConfig, Result as ProjectResult, read_project_config,
};
pub use parkforge_types::{BuildConfig, BuildConfigValue, GameId, GameIdError};
pub use rebuild::{RebuildError, RebuildPaths, rebuild, rebuild_from};
