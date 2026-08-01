use ariadne::{CharSet, Color, Config, Fmt, IndexType, Label, Report, ReportKind, Source};
use linguini_syntax::Span;
use std::fmt;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Advice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCategory {
    Syntax,
    Semantic,
    Lint,
    Project,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedSpan {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickFix {
    pub title: String,
    pub action: QuickFixAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickFixAction {
    Hint,
    Command {
        id: String,
    },
    Replace {
        id: Option<String>,
        replacement: Replacement,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replacement {
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable machine-readable identifier. Human-readable message text is not an API.
    pub code: &'static str,
    pub category: DiagnosticCategory,
    pub lint_name: Option<&'static str>,
    pub severity: DiagnosticSeverity,
    pub message: String,
    /// `None` represents a genuinely source-less project diagnostic.
    pub source_span: Option<Span>,
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
            code: "linguini.semantic",
            category: DiagnosticCategory::Semantic,
            lint_name: None,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source_span: Some(span),
            note: None,
            related: Vec::new(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            code: "linguini.semantic",
            category: DiagnosticCategory::Semantic,
            lint_name: None,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source_span: Some(span),
            note: None,
            related: Vec::new(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn advice(message: impl Into<String>, span: Span) -> Self {
        Self {
            code: "linguini.semantic",
            category: DiagnosticCategory::Semantic,
            lint_name: None,
            severity: DiagnosticSeverity::Advice,
            message: message.into(),
            source_span: Some(span),
            note: None,
            related: Vec::new(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }

    pub fn with_category(mut self, category: DiagnosticCategory) -> Self {
        self.category = category;
        self
    }

    pub fn as_lint(mut self, name: &'static str) -> Self {
        self.category = DiagnosticCategory::Lint;
        self.lint_name = Some(name);
        self.code = name;
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

    pub fn without_source(mut self) -> Self {
        self.source_span = None;
        self
    }
}

impl QuickFix {
    pub fn hint(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action: QuickFixAction::Hint,
        }
    }

    pub fn command(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action: QuickFixAction::Command { id: id.into() },
        }
    }

    pub fn replacement(title: impl Into<String>, replacement: Replacement) -> Self {
        Self {
            title: title.into(),
            action: QuickFixAction::Replace {
                id: None,
                replacement,
            },
        }
    }

    pub fn replacement_with_id(
        id: impl Into<String>,
        title: impl Into<String>,
        replacement: Replacement,
    ) -> Self {
        let id = id.into();
        Self {
            title: title.into(),
            action: QuickFixAction::Replace {
                id: Some(id),
                replacement,
            },
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        let id = id.into();
        self.action = match self.action {
            QuickFixAction::Replace { replacement, .. } => QuickFixAction::Replace {
                id: Some(id),
                replacement,
            },
            QuickFixAction::Hint | QuickFixAction::Command { .. } => QuickFixAction::Command { id },
        };
        self
    }
}

impl QuickFixAction {
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Hint => None,
            Self::Command { id } => Some(id),
            Self::Replace { id, .. } => id.as_deref(),
        }
    }

    pub fn replacement(&self) -> Option<&Replacement> {
        match self {
            Self::Replace { replacement, .. } => Some(replacement),
            Self::Hint | Self::Command { .. } => None,
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
    let source_length = source.len();
    let source = Source::from(source);
    let config = Config::default()
        .with_color(color)
        .with_char_set(CharSet::Unicode)
        .with_index_type(IndexType::Byte);
    let mut output = Vec::new();

    for diagnostic in diagnostics {
        let Some(primary_span) = diagnostic.source_span else {
            render_summary_diagnostic(path, &mut output, diagnostic, color);
            continue;
        };

        let mut builder = Report::build(
            report_kind(diagnostic.severity),
            (path.to_string(), span_range(primary_span, source_length)),
        )
        .with_config(config)
        .with_message(&diagnostic.message)
        .with_label(
            Label::new((path.to_string(), span_range(primary_span, source_length)))
                .with_color(label_color(diagnostic.severity))
                .with_message(&diagnostic.message),
        );

        for related in &diagnostic.related {
            if related.span.source != primary_span.source {
                builder = builder.with_note(format!(
                    "{} (related source #{})",
                    related.message, related.span.source.0
                ));
                continue;
            }
            builder = builder.with_label(
                Label::new((path.to_string(), span_range(related.span, source_length)))
                    .with_color(Color::Cyan)
                    .with_message(&related.message),
            );
        }

        if let Some(note) = &diagnostic.note {
            builder = builder.with_note(note);
        }

        for quick_fix in &diagnostic.quick_fixes {
            builder = builder.with_help(quick_fix_description(quick_fix));
        }

        builder
            .finish()
            .write((path.to_string(), &source), &mut output)
            .map_err(|source| RenderError { source })?;
    }

    let rendered = String::from_utf8(output).map_err(|source| RenderError {
        source: io::Error::new(io::ErrorKind::InvalidData, source),
    })?;
    Ok(trim_trailing_layout_padding(&rendered))
}

fn trim_trailing_layout_padding(rendered: &str) -> String {
    let mut output = rendered
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    if rendered.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn render_summary_diagnostic(
    path: &str,
    output: &mut Vec<u8>,
    diagnostic: &Diagnostic,
    color: bool,
) {
    let label = severity_label(diagnostic.severity);
    let rendered_label = if color {
        format!("{}", label.fg(label_color(diagnostic.severity)))
    } else {
        label.to_owned()
    };

    push_line(output, &format!("{rendered_label}: {}", diagnostic.message));
    push_line(output, &format!("  in {path}"));

    for quick_fix in &diagnostic.quick_fixes {
        push_line(
            output,
            &format!("  Fix: {}", quick_fix_description(quick_fix)),
        );
    }

    if let Some(note) = &diagnostic.note {
        push_line(output, &format!("  Note: {note}"));
    }

    output.push(b'\n');
}

fn push_line(output: &mut Vec<u8>, line: &str) {
    output.extend_from_slice(line.as_bytes());
    output.push(b'\n');
}

fn quick_fix_description(quick_fix: &QuickFix) -> String {
    match quick_fix.action.id() {
        Some(id) => format!(
            "{} (run `linguini fix {}` or `linguini fix --all`)",
            quick_fix.title, id
        ),
        None => format!("quick fix: {}", quick_fix.title),
    }
}

fn severity_label(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "Error",
        DiagnosticSeverity::Warning => "Warning",
        DiagnosticSeverity::Advice => "Advice",
    }
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

fn span_range(span: Span, source_length: usize) -> std::ops::Range<usize> {
    let start = span.start.min(source_length);
    let end = span.end.max(start).min(source_length);
    start..end
}
