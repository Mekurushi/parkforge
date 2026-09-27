use std::fs;
use std::path::{Path, PathBuf};

use parkforge_types::BuildConfig;
use rlb_domain::RLBFile;

use super::create::CreateDocument;
use super::error::{Error, Result};
use super::patch::PatchDocument;
use crate::diagnostic::BuildDiagnostic;

pub(crate) struct Create {
    source: PathBuf,
    target: PathBuf,
}

pub(crate) struct Patch {
    directory: PathBuf,
    source: PathBuf,
    target: PathBuf,
}

impl Create {
    pub(crate) fn new(source: PathBuf, relative_source: &Path) -> Self {
        Self {
            source,
            target: relative_source.with_extension(""),
        }
    }

    pub(crate) fn source(&self) -> &Path {
        &self.source
    }

    pub(crate) fn target(&self) -> &Path {
        &self.target
    }

    pub(crate) fn check(
        &self,
        config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        //TODO: diagnostics
        CreateDocument::read(&self.source)?.check(&self.source, config)
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let target = staging_root.join(&self.target);
        let document = CreateDocument::read(&self.source)?;
        let file = document.create(&self.source, &target, config)?;
        let bytes = file.write().map_err(|source| Error::SerializeRlb {
            path: target.clone(),
            source,
        })?;
        fs::write(&target, bytes).map_err(|source| Error::WriteRlb {
            path: target,
            source,
        })
    }
}

impl Patch {
    pub(crate) fn new(directory: PathBuf, relative_directory: &Path) -> Result<Self> {
        let target = relative_directory.with_extension("");
        let target_name = target
            .file_name()
            .ok_or_else(|| Error::UnsupportedSourceFormat(directory.clone()))?;
        let source = directory.join(Path::new(target_name).with_extension("toml"));
        Ok(Self {
            directory,
            source,
            target,
        })
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
        PatchDocument::read(&self.source)?.check(&self.source, config)
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let target = staging_root.join(&self.target);
        let source = PatchDocument::read(&self.source)?;
        let bytes = fs::read(&target).map_err(|source| Error::ReadRlb {
            path: target.clone(),
            source,
        })?;
        let mut file = RLBFile::parse(&bytes).map_err(|source| Error::ParseRlb {
            path: target.clone(),
            source,
        })?;

        source.apply(&self.source, &target, config, &mut file)?;

        let bytes = file.write().map_err(|source| Error::SerializeRlb {
            path: target.clone(),
            source,
        })?;
        fs::write(&target, bytes).map_err(|source| Error::WriteRlb {
            path: target,
            source,
        })
    }
}
