use crate::archive::{ArchiveFormat, pack_archive};
use crate::compression::{CompressionFormat, compress};
use crate::error::{Error, Result};
use parkforge_wii_disc::rebuild_iso;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::Builder;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebuildProgress {
    Archives { completed: usize, total: usize },
    Disc(parkforge_wii_disc::Progress),
}

pub fn rebuild_game_tree<F>(tree: &Path, destination_iso: &Path, mut progress: F) -> Result<()>
where
    F: FnMut(RebuildProgress),
{
    let archives = discover_archives(tree)?;
    let total = archives.len();
    progress(RebuildProgress::Archives {
        completed: 0,
        total,
    });

    for (index, archive) in archives.into_iter().enumerate() {
        repack_archive(&archive.path, archive.format, archive.compression)?;
        progress(RebuildProgress::Archives {
            completed: index + 1,
            total,
        });
    }

    rebuild_iso(tree, destination_iso, |disc_progress| {
        progress(RebuildProgress::Disc(disc_progress));
    })?;
    Ok(())
}

fn discover_archives(root: &Path) -> Result<Vec<PendingArchive>> {
    let mut archives = Vec::new();
    for entry in WalkDir::new(root).contents_first(true) {
        let entry = entry.map_err(|source| Error::Walk {
            root: root.to_path_buf(),
            source,
        })?;
        if !entry.file_type().is_dir() {
            continue;
        }

        let compression = match entry.path().extension() {
            Some(extension) if extension == "dan" => CompressionFormat::None,
            Some(extension) if extension == "dac" => CompressionFormat::Nlzss11,
            _ => continue,
        };
        archives.push(PendingArchive {
            path: entry.into_path(),
            format: ArchiveFormat::U8,
            compression,
        });
    }
    Ok(archives)
}

fn repack_archive(
    root: &Path,
    format: ArchiveFormat,
    compression: CompressionFormat,
) -> Result<()> {
    let data = match format {
        ArchiveFormat::U8 => pack_archive(root)?,
    };
    let data = match compression {
        CompressionFormat::None => data,
        CompressionFormat::Nlzss11 => compress(&data),
    };

    let parent = root
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = Builder::new()
        .prefix(".parkforge-archive-")
        .tempfile_in(parent)
        .map_err(|source| Error::CreateTemporaryArchiveFile {
            directory: parent.to_path_buf(),
            source,
        })?;
    temporary
        .write_all(&data)
        .and_then(|()| temporary.flush())
        .map_err(|source| Error::WriteTemporaryArchive {
            path: temporary.path().to_path_buf(),
            source,
        })?;

    fs::remove_dir_all(root).map_err(|source| Error::RemoveArchiveDirectory {
        path: root.to_path_buf(),
        source,
    })?;
    drop(
        temporary
            .persist(root)
            .map_err(|error| Error::CommitRepackedArchive {
                destination: root.to_path_buf(),
                source: error.error,
            })?,
    );
    Ok(())
}

struct PendingArchive {
    path: PathBuf,
    format: ArchiveFormat,
    compression: CompressionFormat,
}
