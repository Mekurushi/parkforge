use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use parkforge_types::BuildConfig;

use super::artifact::Artifact;
use super::binary::Dol;
use super::config::{ConfigMappings, ConfigWrite};
use super::error::{Error, Result};
use super::patch::{self, PatchMetadata};
use super::symbols::CustomSymbols;
use crate::diagnostic::BuildDiagnostic;

pub(crate) struct Patch {
    configs: PathBuf,
    custom_symbols: PathBuf,
    directory: PathBuf,
    metadata: PathBuf,
    patches: PathBuf,
    target: PathBuf,
}

impl Patch {
    pub(crate) fn new(directory: PathBuf, relative_directory: &Path) -> Self {
        Self {
            configs: directory.join("configs.toml"),
            custom_symbols: directory.join("custom_symbols.yaml"),
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
    pub(crate) fn check(
        &self,
        config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        drop(self.read_sources(config)?);
        Ok(())
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let (metadata, artifacts, config_writes) = self.read_sources(config)?;
        let target = staging_root.join(&self.target);
        let bytes = fs::read(&target).map_err(|source| Error::ReadDol {
            path: target.clone(),
            source,
        })?;
        let mut dol = Dol::parse(bytes)?;

        for artifact in artifacts {
            patch::apply(&mut dol, &artifact.patchlets, &metadata)?;
        }
        for config_write in config_writes {
            dol.write(config_write.address, &config_write.data)?;
        }

        fs::write(&target, dol.into_bytes()).map_err(|source| Error::WriteDol {
            path: target,
            source,
        })
    }

    fn read_sources(
        &self,
        config: &BuildConfig,
    ) -> Result<(PatchMetadata, Vec<Artifact>, Vec<ConfigWrite>)> {
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

        let config_writes = match fs::read_to_string(&self.configs) {
            Ok(text) => {
                let mappings = ConfigMappings::parse(&text)?;
                let symbols = fs::read_to_string(&self.custom_symbols).map_err(|source| {
                    Error::ReadPatchSource {
                        path: self.custom_symbols.clone(),
                        source,
                    }
                })?;
                let symbols = CustomSymbols::parse(&symbols)?;
                mappings.resolve(config, &symbols)?
            }
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(source) => {
                return Err(Error::ReadPatchSource {
                    path: self.configs.clone(),
                    source,
                });
            }
        };

        Ok((metadata, artifacts, config_writes))
    }
}
