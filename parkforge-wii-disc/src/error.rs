use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::partition::PartitionKind;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error while accessing {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to open Wii ISO {path:?}: {source}")]
    OpenIso {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse Wii ISO: {source}")]
    ParseIso {
        #[source]
        source: binrw::Error,
    },

    #[error("disc header contains an invalid Wii game ID: {raw:?}")]
    InvalidGameId { raw: [u8; 6] },

    #[error("extraction destination is a symlink: {0:?}")]
    DestinationIsSymlink(PathBuf),

    #[error("extraction destination is not a directory: {0:?}")]
    DestinationIsNotDirectory(PathBuf),

    #[error("binary write failed at {path:?}: {source}")]
    BinRw {
        path: PathBuf,
        #[source]
        source: binrw::Error,
    },

    #[error("partition {0} already added")]
    DuplicatePartition(PartitionKind),

    #[error("partition {0} doesn't exist")]
    MissingPartition(PartitionKind),

    #[error("failed to open {partition} partition: {source}")]
    OpenPartition {
        partition: PartitionKind,
        #[source]
        source: binrw::Error,
    },

    #[error("failed to read certificate chain from {partition} partition: {source}")]
    ReadCertificates {
        partition: PartitionKind,
        #[source]
        source: binrw::Error,
    },

    #[error("failed to read TMD from {partition} partition: {source}")]
    ReadTmd {
        partition: PartitionKind,
        #[source]
        source: binrw::Error,
    },

    #[error("failed to rebuild ISO from {path:?}: {source}")]
    Rebuild {
        path: PathBuf,
        // source error in disc_riider is not reachable
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to flush rebuilt ISO: {source}")]
    FlushIso {
        #[source]
        source: io::Error,
    },

    #[error("failed to create a temporary ISO in {directory:?}: {source}")]
    CreateTemporaryIso {
        directory: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to replace rebuilt ISO at {destination:?}: {source}")]
    PersistIso {
        destination: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
