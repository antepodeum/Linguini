use ariadne::{CharSet, Color, Config, IndexType, Label, Report, ReportKind, Source};
use linguini_syntax::Span;
use std::fmt;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Advice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedSpan {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickFix {
    pub title: String,
    pub replacement: Option<Replacement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replacement {
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: Span,
    pub note: Option<String>,
    pub related: Vec<RelatedSpan>,
    pub quick_fixes: Vec<QuickFix>,
}

#[derive(Debug)]
pub struct RenderError {
    source: io::Error,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to render diagnostic: {}", self.source)
    }
}

impl std::error::Error for RenderError {}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            span,
            note: None,
            related: Vec::new(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            span,
            note: None,
            related: Vec::new(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_related(mut self, span: Span, message: impl Into<String>) -> Self {
        self.related.push(RelatedSpan {
            span,
            message: message.into(),
        });
        self
    }

    pub fn with_quick_fix(mut self, quick_fix: QuickFix) -> Self {
        self.quick_fixes.push(quick_fix);
        self
    }
}

impl QuickFix {
    pub fn hint(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            replacement: None,
        }
    }
}

pub fn render_diagnostics(
    path: &str,
    source: &str,
    diagnostics: &[Diagnostic],
) -> Result<String, RenderError> {
    render_diagnostics_with_color(path, source, diagnostics, false)
}

pub fn render_diagnostics_with_color(
    path: &str,
    source: &str,
    diagnostics: &[Diagnostic],
    color: bool,
) -> Result<String, RenderError> {
    let source = Source::from(source);
    let config = Config::default()
        .with_color(color)
        .with_char_set(CharSet::Ascii)
        .with_index_type(IndexType::Byte);
    let mut output = Vec::new();

    for diagnostic in diagnostics {
        let mut builder = Report::build(
            report_kind(diagnostic.severity),
            (path.to_string(), span_range(diagnostic.span)),
        )
        .with_config(config)
        .with_message(&diagnostic.message)
        .with_label(
            Label::new((path.to_string(), span_range(diagnostic.span)))
                .with_color(label_color(diagnostic.severity))
                .with_message(&diagnostic.message),
        );

        for related in &diagnostic.related {
            builder = builder.with_label(
                Label::new((path.to_string(), span_range(related.span)))
                    .with_color(Color::Cyan)
                    .with_message(&related.message),
            );
        }

        if let Some(note) = &diagnostic.note {
            builder = builder.with_note(note);
        }

        for quick_fix in &diagnostic.quick_fixes {
            builder = builder.with_help(format!("quick fix: {}", quick_fix.title));
        }

        builder
            .finish()
            .write((path.to_string(), &source), &mut output)
            .map_err(|source| RenderError { source })?;
    }

    String::from_utf8(output).map_err(|source| RenderError {
        source: io::Error::new(io::ErrorKind::InvalidData, source),
    })
}

fn report_kind(severity: DiagnosticSeverity) -> ReportKind<'static> {
    match severity {
        DiagnosticSeverity::Error => ReportKind::Error,
        DiagnosticSeverity::Warning => ReportKind::Warning,
        DiagnosticSeverity::Advice => ReportKind::Advice,
    }
}

fn label_color(severity: DiagnosticSeverity) -> Color {
    match severity {
        DiagnosticSeverity::Error => Color::Red,
        DiagnosticSeverity::Warning => Color::Yellow,
        DiagnosticSeverity::Advice => Color::Blue,
    }
}

fn span_range(span: Span) -> std::ops::Range<usize> {
    span.start..span.end
}
