use std::fs;
use std::path::Path;

use crate::diagnostic::{
    BuildDiagnostic, DiagnosticLabel, DiagnosticLabelStyle, DiagnosticSeverity,
};
use crate::error::{Error, Result};
use fsc_compiler::{CompileRequest, compile};
use fsc_diagnostics::{Diagnostic, LabelStyle, Severity};
use fsc_patcher::{PatchFailure, PatchRequest, parse_symbol_table, patch};

pub(crate) fn compile_loose(
    source: &Path,
    target: &Path,
    diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
) -> Result<()> {
    let script_name = source
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::InvalidScriptName(source.to_path_buf()))?;
    let text = fs::read_to_string(source).map_err(|error| Error::ReadFscSource {
        path: source.to_path_buf(),
        source: error,
    })?;
    let artifact = compile(CompileRequest::new(&text, script_name)).map_err(|failure| {
        emit_diagnostics(source, &text, failure.diagnostics(), diagnostics);
        Error::CompileFsc {
            path: source.to_path_buf(),
            failure,
        }
    })?;
    // TODO: decide how to handle missing target directories
    fs::write(target, artifact.bytes()).map_err(|error| Error::WriteFsb {
        path: target.to_path_buf(),
        source: error,
    })
}

pub(crate) fn patch_directory(
    directory: &Path,
    target: &Path,
    diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
) -> Result<()> {
    let mut entries = fs::read_dir(directory).map_err(|source| Error::InspectPath {
        path: directory.to_path_buf(),
        source,
    })?;
    match entries.next() {
        None => return Ok(()),
        Some(entry) => drop(entry.map_err(|source| Error::InspectPath {
            path: directory.to_path_buf(),
            source,
        })?),
    }

    let target_name = target
        .file_name()
        .ok_or_else(|| Error::UnsupportedSourceFormat(directory.to_path_buf()))?;
    let source = directory.join(Path::new(target_name).with_extension("fsc"));
    let symbols_path = directory.join("symbols.toml");
    let text = fs::read_to_string(&source).map_err(|error| Error::ReadFscSource {
        path: source.clone(),
        source: error,
    })?;
    let original = fs::read(target).map_err(|source| Error::ReadFsb {
        path: target.to_path_buf(),
        source,
    })?;
    let symbols_text = fs::read_to_string(&symbols_path).map_err(|source| Error::ReadSymbols {
        path: symbols_path.clone(),
        source,
    })?;
    let symbols = parse_symbol_table(&symbols_text).map_err(|source| Error::ParseSymbols {
        path: symbols_path,
        source,
    })?;
    let artifact = patch(PatchRequest::new(&text, &original, &symbols)).map_err(|error| {
        if let PatchFailure::InvalidPatchSource(failure_diagnostics) = &error {
            emit_diagnostics(&source, &text, failure_diagnostics, diagnostics);
        }
        Error::PatchFsb {
            path: source,
            target: target.to_path_buf(),
            source: error,
        }
    })?;
    // TODO: find out how to export symbols
    fs::write(target, artifact.binary()).map_err(|source| Error::WriteFsb {
        path: target.to_path_buf(),
        source,
    })
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
