use crate::archive::{ArchiveFormat, unpack_archive};
use crate::compression::{CompressionFormat, decompress};
use crate::error::{Error, Result};
use parkforge_wii_disc::{PartitionKind, WiiIsoExtractor};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::Builder;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionProgress {
    Disc(parkforge_wii_disc::Progress),
    Archives { completed: usize, discovered: usize },
}

pub fn extract_game_tree<F>(iso_path: &Path, destination: &Path, mut progress: F) -> Result<()>
where
    F: FnMut(ExtractionProgress),
{
    let mut extractor = WiiIsoExtractor::open(iso_path)?;
    extractor.prepare_partition(PartitionKind::Data)?;
    extractor.extract_to(destination, |disc_progress| {
        progress(ExtractionProgress::Disc(disc_progress));
    })?;
    walk_archive_extraction(destination, &mut progress)
}

fn walk_archive_extraction<F>(root: &Path, progress: &mut F) -> Result<()>
where
    F: FnMut(ExtractionProgress),
{
    let mut pending = Vec::new();
    enqueue_archives(root, &mut pending)?;
    let mut completed = 0;
    let mut discovered = pending.len();
    progress(ExtractionProgress::Archives {
        completed,
        discovered,
    });

    while let Some(archive) = pending.pop() {
        extract_archive(&archive.path, archive.format, archive.compression_format)?;
        let queued_before = pending.len();
        enqueue_archives(&archive.path, &mut pending)?;
        discovered += pending.len() - queued_before;
        completed += 1;
        progress(ExtractionProgress::Archives {
            completed,
            discovered,
        });
    }
    Ok(())
}

fn enqueue_archives(root: &Path, pending: &mut Vec<PendingArchive>) -> Result<()> {
    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|source| Error::Walk {
            root: root.to_path_buf(),
            source,
        })?;
        if !entry.file_type().is_file() {
            continue;
        }

        let compression_format = match entry.path().extension() {
            Some(extension) if extension == "dac" => CompressionFormat::Nlzss11,
            Some(extension) if extension == "dan" => CompressionFormat::None,
            _ => continue,
        };
        pending.push(PendingArchive {
            path: entry.into_path(),
            format: ArchiveFormat::U8,
            compression_format,
        });
    }
    Ok(())
}

fn extract_archive(
    path: &Path,
    archive_format: ArchiveFormat,
    compression_format: CompressionFormat,
) -> Result<()> {
    let data = fs::read(path).map_err(|source| Error::ReadArchive {
        path: path.to_path_buf(),
        source,
    })?;

    let data = match compression_format {
        CompressionFormat::None => data,
        CompressionFormat::Nlzss11 => decompress(&data)?,
    };
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temporary = Builder::new()
        .prefix(".parkforge-archive-")
        .tempdir_in(parent)
        .map_err(|source| Error::CreateTemporaryArchiveDirectory {
            directory: parent.to_path_buf(),
            source,
        })?;
    match archive_format {
        ArchiveFormat::U8 => unpack_archive(data, temporary.path())?,
    }
    fs::remove_file(path).map_err(|source| Error::RemoveArchive {
        path: path.to_path_buf(),
        source,
    })?;
    let temporary_path = temporary.keep();
    fs::rename(&temporary_path, path).map_err(|source| Error::CommitExtractedArchive {
        temporary_path,
        destination: path.to_path_buf(),
        source,
    })?;

    Ok(())
}

struct PendingArchive {
    path: PathBuf,
    format: ArchiveFormat,
    compression_format: CompressionFormat,
}
