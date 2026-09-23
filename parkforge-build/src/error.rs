use std::path::PathBuf;

use fsc_compiler::ConfigType;
use parkforge_types::BuildConfigValue;
use rlb_domain::Error as RlbError;
use thiserror::Error;

//TODO: decentralize errors into their corresponding concerns; rlb errors into rlb-errors, fsb
// errrors into fsb-errors etc.
#[derive(Debug, Error)]
pub enum Error {
    #[error("sources {first:?} and {second:?} both target {target:?}")]
    ConflictingTarget {
        target: PathBuf,
        first: PathBuf,
        second: PathBuf,
    },

    #[error("failed to read RLB source {path:?}: {source}")]
    ReadRlbSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse RLB source {path:?}: {source}")]
    ParseRlbSource {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error(
        "RLB source {path:?} assigns table {table:?}, row {row}, field {field:?} more than once"
    )]
    DuplicateRlbAssignment {
        path: PathBuf,
        table: String,
        row: usize,
        field: String,
    },

    #[error(
        "RLB source {path:?} has an invalid assignment for table {table:?}, row {row}, field {field:?}: {message}"
    )]
    InvalidRlbAssignment {
        path: PathBuf,
        table: String,
        row: usize,
        field: String,
        message: &'static str,
    },

    #[error(
        "RLB source {path:?} assignment for table {table:?}, row {row}, field {field:?} requires missing configuration value {name:?}"
    )]
    MissingRlbConfig {
        path: PathBuf,
        table: String,
        row: usize,
        field: String,
        name: String,
    },

    #[error(
        "RLB source {path:?} assignment for table {table:?}, row {row}, field {field:?} has integer {value}, which does not fit in u32"
    )]
    InvalidRlbInteger {
        path: PathBuf,
        table: String,
        row: usize,
        field: String,
        value: i64,
    },

    #[error(
        "RLB source {path:?} assignment for table {table:?}, row {row}, field {field:?} has float {value}, which must be finite after conversion to f32"
    )]
    InvalidRlbFloat {
        path: PathBuf,
        table: String,
        row: usize,
        field: String,
        value: f64,
    },

    #[error("failed to read RLB {path:?}: {source}")]
    ReadRlb {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse RLB {path:?}: {source}")]
    ParseRlb {
        path: PathBuf,
        #[source]
        source: RlbError,
    },

    #[error("failed to inspect RLB {path:?}: {source}")]
    InspectRlb {
        path: PathBuf,
        #[source]
        source: RlbError,
    },

    #[error("RLB source {path:?} references missing table {table:?} in {target:?}")]
    MissingRlbTable {
        path: PathBuf,
        target: PathBuf,
        table: String,
    },

    #[error("failed to create table {table:?} from RLB source {path:?} for {target:?}: {source}")]
    CreateRlbTable {
        path: PathBuf,
        target: PathBuf,
        table: String,
        #[source]
        source: RlbError,
    },

    #[error(
        "failed to apply RLB source {path:?} assignment for table {table:?}, row {row}, field {field:?} to {target:?}: {source}"
    )]
    PatchRlb {
        path: PathBuf,
        target: PathBuf,
        table: String,
        row: usize,
        field: String,
        #[source]
        source: RlbError,
    },

    #[error("failed to serialize RLB {path:?}: {source}")]
    SerializeRlb {
        path: PathBuf,
        #[source]
        source: RlbError,
    },

    #[error("failed to write RLB {path:?}: {source}")]
    WriteRlb {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read FSB {path:?}: {source}")]
    ReadFsb {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read symbols {path:?}: {source}")]
    ReadSymbols {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse symbols {path:?}: {source}")]
    ParseSymbols {
        path: PathBuf,
        #[source]
        source: fsc_patcher::SymbolTableParseError,
    },

    #[error("failed to apply FSC patch {path:?} to FSB {target:?}: {source}")]
    PatchFsb {
        path: PathBuf,
        target: PathBuf,
        #[source]
        source: fsc_patcher::PatchFailure,
    },

    #[error("FSC source {0:?} must have a valid UTF-8 script name")]
    InvalidScriptName(PathBuf),

    #[error("failed to read FSC source {path:?}: {source}")]
    ReadFscSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to compile FSC source {path:?}")]
    CompileFsc {
        path: PathBuf,
        failure: fsc_compiler::CompileFailure,
    },

    #[error("FSC source {path:?} requires configuration value {name:?} of type {expected}")]
    MissingFscConfig {
        path: PathBuf,
        name: String,
        expected: &'static str,
    },

    #[error(
        "FSC source {path:?} requires configuration value {name:?} of type {expected}, but its configured type is {found}"
    )]
    InvalidFscConfigType {
        path: PathBuf,
        name: String,
        expected: &'static str,
        found: &'static str,
    },

    #[error("failed to write FSB {path:?}: {source}")]
    WriteFsb {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to walk source tree {root:?}: {source}")]
    WalkSources {
        root: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    #[error("failed to resolve source {path:?} below {root:?}: {source}")]
    RelativeSourcePath {
        path: PathBuf,
        root: PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },

    #[error("unsupported source format: {0:?}")]
    UnsupportedSourceFormat(PathBuf),

    #[error("patch directory {path:?} must not be nested inside {outer:?}")]
    NestedPatchDirectory { path: PathBuf, outer: PathBuf },

    #[error("failed to resolve build path {path:?}: {source}")]
    ResolvePath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to inspect build path {path:?}: {source}")]
    InspectPath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("build path {0:?} must be an existing directory")]
    NotDirectory(PathBuf),

    #[error("symbolic links are not supported: {0:?}")]
    SymbolicLink(PathBuf),

    #[error("build destination {0:?} must name a directory below an existing parent")]
    InvalidDestination(PathBuf),

    #[error("build destination {destination:?} must not overlap input tree {input:?}")]
    OverlappingTrees {
        input: PathBuf,
        destination: PathBuf,
    },

    #[error("failed to walk original tree {root:?}: {source}")]
    WalkOriginal {
        root: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    #[error("failed to resolve entry {path:?} below original {root:?}: {source}")]
    RelativeEntry {
        path: PathBuf,
        root: PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },

    #[error("unsupported original entry type: {0:?}")]
    UnsupportedEntry(PathBuf),

    #[error("failed to create staged directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to copy {input:?} to {output:?}: {source}")]
    CopyFile {
        input: PathBuf,
        output: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create temporary build directory in {parent:?}: {source}")]
    CreateTemporaryDirectory {
        parent: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to move previous build {destination:?} to backup {backup:?}: {source}")]
    BackupBuild {
        destination: PathBuf,
        backup: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to commit staged build to {destination:?}: {source}")]
    CommitBuild {
        destination: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "failed to commit build to {destination:?}: {source}; restoring the previous build also failed: {rollback_error}; previous build preserved at {backup:?}"
    )]
    RollbackBuild {
        destination: PathBuf,
        backup: PathBuf,
        #[source]
        source: std::io::Error,
        rollback_error: std::io::Error,
    },

    #[error("build committed, but failed to remove backup directory {path:?}: {source}")]
    RemoveBackup {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    pub(crate) fn missing_fsc_config(path: PathBuf, name: String, expected: ConfigType) -> Self {
        Self::MissingFscConfig {
            path,
            name,
            expected: config_type_name(expected),
        }
    }

    pub(crate) fn invalid_fsc_config_type(
        path: PathBuf,
        name: String,
        expected: ConfigType,
        found: &BuildConfigValue,
    ) -> Self {
        Self::InvalidFscConfigType {
            path,
            name,
            expected: config_type_name(expected),
            found: value_type_name(found),
        }
    }
}

const fn config_type_name(config_type: ConfigType) -> &'static str {
    match config_type {
        ConfigType::Int => "int",
        ConfigType::Float => "float",
        ConfigType::Bool => "bool",
        ConfigType::String => "string",
    }
}

const fn value_type_name(value: &BuildConfigValue) -> &'static str {
    match value {
        BuildConfigValue::Integer(_) => "int",
        BuildConfigValue::Float(_) => "float",
        BuildConfigValue::Boolean(_) => "bool",
        BuildConfigValue::String(_) => "string",
    }
}

pub type Result<T> = std::result::Result<T, Error>;
