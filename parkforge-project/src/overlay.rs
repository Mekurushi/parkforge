use std::fs;
use std::io;
use std::path::Path;

use walkdir::WalkDir;

use crate::error::{Error, Result};

pub(crate) fn merge_sources(shared: &Path, revision: &Path, destination: &Path) -> Result<()> {
    // orchestration is responsible for temporary lifetime contract of destination
    let metadata = fs::metadata(destination).map_err(|source| Error::InspectMergeDestination {
        path: destination.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() {
        return Err(Error::MergeDestinationNotDirectory(
            destination.to_path_buf(),
        ));
    }
    let mut contents =
        fs::read_dir(destination).map_err(|source| Error::InspectMergeDestination {
            path: destination.to_path_buf(),
            source,
        })?;
    if let Some(entry) = contents.next() {
        drop(entry.map_err(|source| Error::InspectMergeDestination {
            path: destination.to_path_buf(),
            source,
        })?);
        return Err(Error::MergeDestinationNotEmpty(destination.to_path_buf()));
    }

    copy_sources(shared, destination)?;
    copy_sources(revision, destination)
}

fn copy_sources(root: &Path, destination: &Path) -> Result<()> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(Error::SourceSymlink(root.to_path_buf()));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(Error::SourceNotDirectory(root.to_path_buf()));
        }
        Ok(_) => {}
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(Error::InspectSource {
                path: root.to_path_buf(),
                source,
            });
        }
    }

    for entry in WalkDir::new(root).min_depth(1).sort_by_file_name() {
        let entry = entry.map_err(|source| Error::WalkSources {
            root: root.to_path_buf(),
            source,
        })?;
        if entry.file_type().is_symlink() {
            return Err(Error::SourceSymlink(entry.into_path()));
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map(Path::to_path_buf)
            .map_err(|source| Error::RelativeSourcePath {
                path: entry.path().to_path_buf(),
                root: root.to_path_buf(),
                source,
            })?;
        let target = destination.join(&relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|source| Error::CreateDirectory {
                path: target,
                source,
            })?;
        } else if entry.file_type().is_file() {
            copy_source_file(entry.path(), &target).map_err(|error| match error {
                Error::CopySource { ref source, .. }
                    if source.kind() == io::ErrorKind::AlreadyExists =>
                {
                    Error::SourceConflict(relative)
                }
                error => error,
            })?;
        }
    }
    Ok(())
}

fn copy_source_file(source: &Path, destination: &Path) -> Result<()> {
    let mut input = fs::File::open(source).map_err(|error| Error::CopySource {
        path: source.to_path_buf(),
        destination: destination.to_path_buf(),
        source: error,
    })?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| Error::CopySource {
            path: source.to_path_buf(),
            destination: destination.to_path_buf(),
            source: error,
        })?;
    let _ = io::copy(&mut input, &mut output).map_err(|error| Error::CopySource {
        path: source.to_path_buf(),
        destination: destination.to_path_buf(),
        source: error,
    })?;
    Ok(())
}
