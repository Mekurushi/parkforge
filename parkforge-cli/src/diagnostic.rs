use std::ops::Range;

use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
use parkforge::{BuildDiagnostic, DiagnosticLabelStyle, DiagnosticSeverity};

pub(crate) fn render(diagnostic: &BuildDiagnostic<'_>) {
    let source_name = diagnostic.source_path.to_string_lossy();
    let level = match diagnostic.severity {
        DiagnosticSeverity::Error => Level::ERROR,
        DiagnosticSeverity::Warning => Level::WARNING,
    };
    let mut group = Group::with_title(level.primary_title(&diagnostic.message));

    if !diagnostic.labels.is_empty() {
        let mut snippet = Snippet::source(diagnostic.source_text).path(&source_name);
        for label in &diagnostic.labels {
            let span = sanitize_span(label.span.clone(), diagnostic.source_text);
            let annotation = match label.style {
                DiagnosticLabelStyle::Primary => AnnotationKind::Primary,
                DiagnosticLabelStyle::Secondary => AnnotationKind::Context,
            }
            .span(span)
            .label(&label.message);
            snippet = snippet.annotation(annotation);
        }
        group = group.element(snippet);
    }

    eprintln!("{}", Renderer::styled().render(&[group]));
}

fn sanitize_span(span: Range<usize>, source: &str) -> Range<usize> {
    let mut start = span.start.min(source.len());
    while start > 0 && !source.is_char_boundary(start) {
        start -= 1;
    }

    let mut end = span.end.max(start).min(source.len());
    while end > start && !source.is_char_boundary(end) {
        end -= 1;
    }
    start..end
}
