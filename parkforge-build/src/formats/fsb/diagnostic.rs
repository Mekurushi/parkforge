use std::path::Path;

use fsc_diagnostics::{Diagnostic, LabelStyle, Severity};

use crate::diagnostic::{
    BuildDiagnostic, DiagnosticLabel, DiagnosticLabelStyle, DiagnosticSeverity,
};

pub(super) fn emit(
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
