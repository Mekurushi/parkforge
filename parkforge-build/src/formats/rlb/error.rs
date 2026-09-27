use std::fmt;
use std::path::PathBuf;

use rlb_domain::Error as RlbError;
use thiserror::Error;

pub(super) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum RlbValueLocation {
    Row(usize),
    Append(usize),
}

impl fmt::Display for RlbValueLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Row(index) => write!(formatter, "row {index}"),
            Self::Append(index) => write!(formatter, "append {index}"),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported source format: {0:?}")]
    UnsupportedSourceFormat(PathBuf),

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

    #[error("RLB source {path:?} removes table {table:?}, row {row} more than once")]
    DuplicateRlbRemoval {
        path: PathBuf,
        table: String,
        row: usize,
    },

    #[error(
        "RLB source {path:?} has an invalid assignment for table {table:?}, {location}, field {field:?}: {message}"
    )]
    InvalidRlbAssignment {
        path: PathBuf,
        table: String,
        location: RlbValueLocation,
        field: String,
        message: &'static str,
    },

    #[error(
        "RLB source {path:?} assignment for table {table:?}, {location}, field {field:?} requires missing configuration value {name:?}"
    )]
    MissingRlbConfig {
        path: PathBuf,
        table: String,
        location: RlbValueLocation,
        field: String,
        name: String,
    },

    #[error(
        "RLB source {path:?} assignment for table {table:?}, {location}, field {field:?} has integer {value}, which does not fit in u32"
    )]
    InvalidRlbInteger {
        path: PathBuf,
        table: String,
        location: RlbValueLocation,
        field: String,
        value: i64,
    },

    #[error(
        "RLB source {path:?} assignment for table {table:?}, {location}, field {field:?} has float {value}, which must be finite after conversion to f32"
    )]
    InvalidRlbFloat {
        path: PathBuf,
        table: String,
        location: RlbValueLocation,
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

    #[error(
        "failed to remove row {row} from table {table:?} in {target:?} for RLB source {path:?}: {source}"
    )]
    RemoveRlbRow {
        path: PathBuf,
        target: PathBuf,
        table: String,
        row: usize,
        #[source]
        source: RlbError,
    },

    #[error(
        "failed to apply append {append} to table {table:?} in {target:?} for RLB source {path:?}: {source}"
    )]
    AppendRlbRow {
        path: PathBuf,
        target: PathBuf,
        table: String,
        append: usize,
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
}
