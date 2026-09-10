use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Wii disc operation failed: {0}")]
    WiiDisc(#[from] parkforge_wii_disc::Error),

    #[error("failed to walk {root:?}: {source}")]
    Walk {
        root: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    #[error("failed to decompress NLZSS11 data: {source}")]
    DecompressNlzss11 {
        #[source]
        source: nlzss11::DecompressError,
    },

    #[error("failed to parse U8 archive for {destination:?}: {source}")]
    ParseU8 {
        destination: PathBuf,
        #[source]
        source: u8arc::U8ParseError,
    },

    #[error("U8 archive data for {path:?} is outside the archive")]
    InvalidArchiveData { path: PathBuf },

    #[error("invalid archive entry path: {0:?}")]
    InvalidArchiveEntryPath(PathBuf),

    #[error("failed to read archive entry {path:?}: {source}")]
    ReadArchiveEntry {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to pack U8 archive from {path:?}: {source}")]
    PackU8 {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to create archive directory {path:?}: {source}")]
    CreateArchiveDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write archive entry {path:?}: {source}")]
    WriteArchiveEntry {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read archive {path:?}: {source}")]
    ReadArchive {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create a temporary archive directory in {directory:?}: {source}")]
    CreateTemporaryArchiveDirectory {
        directory: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to remove archive file {path:?}: {source}")]
    RemoveArchive {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "failed to move extracted archive from {temporary_path:?} to \
       {destination:?}: {source}"
    )]
    CommitExtractedArchive {
        temporary_path: PathBuf,
        destination: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create a temporary archive file in {directory:?}: {source}")]
    CreateTemporaryArchiveFile {
        directory: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write temporary archive {path:?}: {source}")]
    WriteTemporaryArchive {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to remove expanded archive directory {path:?}: {source}")]
    RemoveArchiveDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to commit repacked archive at {destination:?}: {source}")]
    CommitRepackedArchive {
        destination: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
