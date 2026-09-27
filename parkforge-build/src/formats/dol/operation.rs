use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use parkforge_types::BuildConfig;

use super::artifact::Artifact;
use super::binary::Dol;
use super::error::{Error, Result};
use super::patch::{self, PatchMetadata};
use crate::diagnostic::BuildDiagnostic;

pub(crate) struct Patch {
    directory: PathBuf,
    metadata: PathBuf,
    patches: PathBuf,
    target: PathBuf,
}

impl Patch {
    pub(crate) fn new(directory: PathBuf, relative_directory: &Path) -> Self {
        Self {
            metadata: directory.join("metadata.toml"),
            patches: directory.join("patches"),
            directory,
            target: relative_directory.with_extension(""),
        }
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn target(&self) -> &Path {
        &self.target
    }
    //TODO: config support
    pub(crate) fn check(
        &self,
        _config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        drop(self.read_sources()?);
        Ok(())
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        _config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let (metadata, artifacts) = self.read_sources()?;
        let target = staging_root.join(&self.target);
        let bytes = fs::read(&target).map_err(|source| Error::ReadDol {
            path: target.clone(),
            source,
        })?;
        let mut dol = Dol::parse(bytes)?;

        for artifact in artifacts {
            patch::apply(&mut dol, &artifact.patchlets, &metadata)?;
        }

        fs::write(&target, dol.into_bytes()).map_err(|source| Error::WriteDol {
            path: target,
            source,
        })
    }

    fn read_sources(&self) -> Result<(PatchMetadata, Vec<Artifact>)> {
        let metadata =
            fs::read_to_string(&self.metadata).map_err(|source| Error::ReadPatchSource {
                path: self.metadata.clone(),
                source,
            })?;
        let metadata = PatchMetadata::parse(&metadata)?;
        let mut paths = Vec::new();

        let entries = fs::read_dir(&self.patches).map_err(|source| Error::ReadPatchDirectory {
            path: self.patches.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| Error::ReadPatchDirectory {
                path: self.patches.clone(),
                source,
            })?;
            let file_type = entry
                .file_type()
                .map_err(|source| Error::InspectPatchSource {
                    path: entry.path(),
                    source,
                })?;
            let path = entry.path();
            if file_type.is_file() && path.extension() == Some(OsStr::new("yaml")) {
                paths.push(path);
            }
        }
        paths.sort();

        let artifacts = paths
            .into_iter()
            .map(|path| {
                let text = fs::read_to_string(&path).map_err(|source| Error::ReadPatchSource {
                    path: path.clone(),
                    source,
                })?;
                Artifact::parse(&text)
            })
            .collect::<Result<_>>()?;
        Ok((metadata, artifacts))
    }
}
