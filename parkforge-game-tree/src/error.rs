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
}

pub type Result<T> = std::result::Result<T, Error>;
