use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};

use crate::error::{Error, Result};

pub(crate) fn absolute_directory(path: &Path, allow_missing: bool) -> Result<PathBuf> {
    let path = path::absolute(path).map_err(|source| Error::ResolvePath {
        path: path.to_path_buf(),
        source,
    })?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(Error::SymbolicLink(path)),
        Ok(metadata) if metadata.is_dir() => Ok(path),
        Ok(_) => Err(Error::NotDirectory(path)),
        Err(source) if allow_missing && source.kind() == io::ErrorKind::NotFound => Ok(path),
        Err(source) => Err(Error::InspectPath { path, source }),
    }
}

pub(crate) fn canonical_directory(path: &Path, allow_missing: bool) -> Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(source) if allow_missing && source.kind() == io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .ok_or_else(|| Error::InvalidDestination(path.to_path_buf()))?;
            let name = path
                .file_name()
                .ok_or_else(|| Error::InvalidDestination(path.to_path_buf()))?;
            let parent = fs::canonicalize(parent).map_err(|source| Error::ResolvePath {
                path: parent.to_path_buf(),
                source,
            })?;
            Ok(parent.join(name))
        }
        Err(source) => Err(Error::ResolvePath {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub(crate) fn validate_no_overlap(input: &Path, destination: &Path) -> Result<()> {
    let input = canonical_directory(input, false)?;
    let destination = canonical_directory(destination, true)?;
    if destination.starts_with(&input) || input.starts_with(&destination) {
        return Err(Error::OverlappingTrees { input, destination });
    }
    Ok(())
}
