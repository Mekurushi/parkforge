use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::diagnostic::BuildDiagnostic;
use crate::error::{Error, Result};
use crate::formats::fsb;
use crate::rules::{self, Classification, Format};

pub(crate) struct Operation {
    source: PathBuf,
    target: PathBuf,
    classification: Classification,
}

impl Operation {
    pub(crate) fn source(&self) -> &Path {
        &self.source
    }

    pub(crate) fn target(&self) -> &Path {
        &self.target
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
        let relative =
            source
                .strip_prefix(sources_root)
                .map_err(|error| Error::RelativeSourcePath {
                    path: source.clone(),
                    root: sources_root.to_path_buf(),
                    source: error,
                })?;
        let target = match classification {
            Classification::LooseFile(Format::Fsb) => relative.with_extension("fsb"),
            Classification::PatchDirectory(_) => relative.with_extension(""),
        };

        Ok(Some(Self {
            source,
            target,
            classification,
        }))
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let target = staging_root.join(&self.target);
        match self.classification {
            Classification::LooseFile(Format::Fsb) => {
                fsb::compile_loose(&self.source, &target, diagnostics)
            }
            Classification::PatchDirectory(Format::Fsb) => {
                fsb::patch_directory(&self.source, &target, diagnostics)
            }
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
