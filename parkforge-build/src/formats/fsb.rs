use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostic::{
    BuildDiagnostic, DiagnosticLabel, DiagnosticLabelStyle, DiagnosticSeverity,
};
use crate::error::{Error, Result};
use fsc_compiler::{CompileRequest, compile};
use fsc_diagnostics::{Diagnostic, LabelStyle, Severity};
use fsc_patcher::{PatchFailure, PatchRequest, parse_symbol_table, patch};

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

    pub(crate) fn process(
        &self,
        staging_root: &Path,
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
        let artifact = compile(CompileRequest::new(&text, script_name)).map_err(|failure| {
            emit_diagnostics(&self.source, &text, failure.diagnostics(), diagnostics);
            Error::CompileFsc {
                path: self.source.clone(),
                failure,
            }
        })?;
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

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let target = staging_root.join(&self.target);
        let text = fs::read_to_string(&self.source).map_err(|error| Error::ReadFscSource {
            path: self.source.clone(),
            source: error,
        })?;
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
        let artifact = patch(PatchRequest::new(&text, &original, &symbols)).map_err(|error| {
            if let PatchFailure::InvalidPatchSource(failure_diagnostics) = &error {
                emit_diagnostics(&self.source, &text, failure_diagnostics, diagnostics);
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

fn emit_diagnostics(
    source_path: &Path,
    source_text: &str,
    source_diagnostics: &[Diagnostic],
    diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
) {
    for diagnostic in source_diagnostics {
        diagnostics(BuildDiagnostic {
            source_path,
            source_text,
            severity: match diagnostic.severity() {
                Severity::Error => DiagnosticSeverity::Error,
                Severity::Warning => DiagnosticSeverity::Warning,
            },
            message: diagnostic.message().to_owned(),
            labels: diagnostic
                .labels()
                .iter()
                .map(|label| DiagnosticLabel {
                    span: label.span().range(),
                    message: label.message().to_owned(),
                    style: match label.style() {
                        LabelStyle::Primary => DiagnosticLabelStyle::Primary,
                        LabelStyle::Secondary => DiagnosticLabelStyle::Secondary,
                    },
                })
                .collect(),
        });
    }
}
