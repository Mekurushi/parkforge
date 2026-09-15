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
    let staging = Builder::new()
        .prefix(".parkforge-rebuild-tree-")
        .tempdir()
        .map_err(|source| Error::CreateRebuildStaging { source })?;
    let staged_tree = staging.path().join("tree");
    copy_tree(tree, &staged_tree)?;

    let archives = discover_archives(&staged_tree)?;
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

    rebuild_iso(&staged_tree, destination_iso, |disc_progress| {
        progress(RebuildProgress::Disc(disc_progress));
    })?;
    Ok(())
}

// TODO: check how propely deduplicating copy_tree across the whole project
fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|error| Error::CopyRebuildTree {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
            error: Box::new(error),
        })?;
        let input = entry.path();
        if entry.file_type().is_symlink() {
            return Err(Error::RebuildSymlink(input.to_path_buf()));
        }
        let relative = input
            .strip_prefix(source)
            .map_err(|error| Error::RelativeRebuildPath {
                path: input.to_path_buf(),
                root: source.to_path_buf(),
                error,
            })?;
        let output = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&output).map_err(|source| Error::RebuildIo {
                path: output,
                source,
            })?;
        } else if entry.file_type().is_file() {
            let _ = fs::copy(input, &output).map_err(|source| Error::RebuildIo {
                path: output,
                source,
            })?;
        } else {
            return Err(Error::UnsupportedRebuildEntry(input.to_path_buf()));
        }
    }
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
