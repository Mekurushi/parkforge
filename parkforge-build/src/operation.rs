use std::ffi::OsStr;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};

use crate::diagnostic::BuildDiagnostic;
use crate::error::{Error, Result};
use crate::formats::fsb;
use crate::rules::{self, Classification, Format};

pub(crate) enum Operation {
    CompileFsb(fsb::Compile),
    PatchFsb(fsb::Patch),
}

impl Operation {
    pub(crate) fn source(&self) -> &Path {
        match self {
            Self::CompileFsb(operation) => operation.source(),
            Self::PatchFsb(operation) => operation.directory(),
        }
    }

    pub(crate) fn target(&self) -> &Path {
        match self {
            Self::CompileFsb(operation) => operation.target(),
            Self::PatchFsb(operation) => operation.target(),
        }
    }

    pub(crate) fn discover(root: &Path) -> Result<Vec<Self>> {
        let entries = WalkDir::new(root)
            .min_depth(1)
            .sort_by_file_name()
            .into_iter()
            .map(|entry| {
                entry.map_err(|source| Error::WalkSources {
                    root: root.to_path_buf(),
                    source,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        validate_source_entries(root, &entries)?;

        let sources = entries
            .into_iter()
            .filter(|entry| enclosing_patch_directory(root, entry.path()).is_none());

        let mut operations = Vec::new();
        for entry in sources {
            if rules::is_patch_directory(&entry) && directory_is_empty(entry.path())? {
                continue;
            }
            if let Some(operation) = Self::from_entry(root, entry)? {
                operations.push(operation);
            }
        }
        Ok(operations)
    }

    fn from_entry(sources_root: &Path, entry: DirEntry) -> Result<Option<Self>> {
        let Some(classification) = rules::classify(&entry)? else {
            return Ok(None);
        };

        let source = entry.into_path();
        let relative = source
            .strip_prefix(sources_root)
            .map_err(|error| Error::RelativeSourcePath {
                path: source.clone(),
                root: sources_root.to_path_buf(),
                source: error,
            })?
            .to_path_buf();
        let operation = match classification {
            Classification::LooseFile(Format::Fsb) => {
                Self::CompileFsb(fsb::Compile::new(source, &relative))
            }
            Classification::PatchDirectory(Format::Fsb) => {
                Self::PatchFsb(fsb::Patch::new(source, &relative)?)
            }
        };

        Ok(Some(operation))
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        match self {
            Self::CompileFsb(operation) => operation.process(staging_root, diagnostics),
            Self::PatchFsb(operation) => operation.process(staging_root, diagnostics),
        }
    }
}

fn directory_is_empty(directory: &Path) -> Result<bool> {
    let mut entries = std::fs::read_dir(directory).map_err(|source| Error::InspectPath {
        path: directory.to_path_buf(),
        source,
    })?;
    match entries.next() {
        None => Ok(true),
        Some(entry) => {
            drop(entry.map_err(|source| Error::InspectPath {
                path: directory.to_path_buf(),
                source,
            })?);
            Ok(false)
        }
    }
}

fn enclosing_patch_directory<'a>(root: &Path, path: &'a Path) -> Option<&'a Path> {
    path.ancestors()
        .skip(1)
        .take_while(|ancestor| *ancestor != root)
        .find(|ancestor| ancestor.extension() == Some(OsStr::new("patch")))
}

fn validate_source_entries(root: &Path, entries: &[DirEntry]) -> Result<()> {
    for entry in entries {
        if entry.file_type().is_symlink() {
            return Err(Error::SymbolicLink(entry.path().to_path_buf()));
        }

        if rules::is_patch_directory(entry)
            && let Some(outer) = enclosing_patch_directory(root, entry.path())
        {
            return Err(Error::NestedPatchDirectory {
                path: entry.path().to_path_buf(),
                outer: outer.to_path_buf(),
            });
        }
    }
    Ok(())
}
