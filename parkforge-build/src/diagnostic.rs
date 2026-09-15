use std::ops::Range;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildDiagnostic<'a> {
    pub source_path: &'a Path,
    pub source_text: &'a str,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub labels: Vec<DiagnosticLabel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLabel {
    pub span: Range<usize>,
    pub message: String,
    pub style: DiagnosticLabelStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLabelStyle {
    Primary,
    Secondary,
}
