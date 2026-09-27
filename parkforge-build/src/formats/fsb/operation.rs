use std::fs;
use std::path::{Path, PathBuf};

use fsc_compiler::{CompileRequest, compile, required_configs};
use fsc_patcher::{PatchFailure, PatchRequest, parse_symbol_table, patch};
use parkforge_types::BuildConfig;

use super::config::FscConfigMapper;
use super::diagnostic::emit;
use super::error::{Error, Result};
use crate::diagnostic::BuildDiagnostic;

pub(crate) struct Compile {
    source: PathBuf,
    target: PathBuf,
}

pub(crate) struct Patch {
    directory: PathBuf,
    source: PathBuf,
    symbols: PathBuf,
    target: PathBuf,
}

impl Compile {
    pub(crate) fn new(source: PathBuf, relative_source: &Path) -> Self {
        Self {
            source,
            target: relative_source.with_extension("fsb"),
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
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        check_source(&self.source, config, diagnostics)
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        config: &BuildConfig,
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let target = staging_root.join(&self.target);
        let script_name = self
            .source
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::InvalidScriptName(self.source.clone()))?;
        let text = fs::read_to_string(&self.source).map_err(|error| Error::ReadFscSource {
            path: self.source.clone(),
            source: error,
        })?;
        let requirements = required_configs(&text).map_err(|failure| {
            emit(&self.source, &text, failure.diagnostics(), diagnostics);
            Error::CompileFsc {
                path: self.source.clone(),
                failure,
            }
        })?;
        let config_values = FscConfigMapper::new(requirements).map(&self.source, config)?;
        let artifact = compile(CompileRequest::new(&text, script_name, &config_values)).map_err(
            |failure| {
                emit(&self.source, &text, failure.diagnostics(), diagnostics);
                Error::CompileFsc {
                    path: self.source.clone(),
                    failure,
                }
            },
        )?;
        // TODO: decide how to handle missing target directories
        fs::write(&target, artifact.bytes()).map_err(|error| Error::WriteFsb {
            path: target,
            source: error,
        })
    }
}

impl Patch {
    pub(crate) fn new(directory: PathBuf, relative_directory: &Path) -> Result<Self> {
        let target = relative_directory.with_extension("");
        let target_name = target
            .file_name()
            .ok_or_else(|| Error::UnsupportedSourceFormat(directory.clone()))?;
        let source = directory.join(Path::new(target_name).with_extension("fsc"));
        let symbols = directory.join("symbols.toml");
        Ok(Self {
            directory,
            source,
            symbols,
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
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        check_source(&self.source, config, diagnostics)
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        config: &BuildConfig,
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let target = staging_root.join(&self.target);
        let text = fs::read_to_string(&self.source).map_err(|error| Error::ReadFscSource {
            path: self.source.clone(),
            source: error,
        })?;
        let requirements = required_configs(&text).map_err(|failure| {
            emit(&self.source, &text, failure.diagnostics(), diagnostics);
            Error::CompileFsc {
                path: self.source.clone(),
                failure,
            }
        })?;
        let config_values = FscConfigMapper::new(requirements).map(&self.source, config)?;
        let original = fs::read(&target).map_err(|source| Error::ReadFsb {
            path: target.clone(),
            source,
        })?;
        let symbols_text =
            fs::read_to_string(&self.symbols).map_err(|source| Error::ReadSymbols {
                path: self.symbols.clone(),
                source,
            })?;
        let symbols = parse_symbol_table(&symbols_text).map_err(|source| Error::ParseSymbols {
            path: self.symbols.clone(),
            source,
        })?;
        let artifact = patch(PatchRequest::new(
            &text,
            &original,
            &symbols,
            &config_values,
        ))
        .map_err(|error| {
            if let PatchFailure::InvalidPatchSource(failure_diagnostics) = &error {
                emit(&self.source, &text, failure_diagnostics, diagnostics);
            }
            Error::PatchFsb {
                path: self.source.clone(),
                target: target.clone(),
                source: error,
            }
        })?;
        // TODO: find out how to export symbols
        fs::write(&target, artifact.binary()).map_err(|source| Error::WriteFsb {
            path: target,
            source,
        })
    }
}

fn check_source(
    source: &Path,
    config: &BuildConfig,
    diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
) -> Result<()> {
    //TODO: aggregated diagnostics output
    let text = fs::read_to_string(source).map_err(|error| Error::ReadFscSource {
        path: source.to_path_buf(),
        source: error,
    })?;
    // The required_configs check passes also the sema phase so we can use that to do both; sema
    // check the source and validate the configs
    let requirements = required_configs(&text).map_err(|failure| {
        emit(source, &text, failure.diagnostics(), diagnostics);
        Error::CompileFsc {
            path: source.to_path_buf(),
            failure,
        }
    })?;
    FscConfigMapper::new(requirements).validate(source, config)
}
