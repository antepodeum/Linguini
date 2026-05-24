use crate::{Diagnostic, DiagnosticSeverity, QuickFix, Replacement};
use linguini_syntax::{
    DocComment, LocaleDeclaration, LocaleFile, SchemaDeclaration, SchemaFile, Span,
};

mod branches;
mod messages;

use self::branches::analyze_locale_branch_coverage;
use self::messages::{
    format_name_list, locale_message_map, missing_message_stub_text, pluralize, schema_message_map,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredLocaleMessage {
    pub name: String,
    pub span: Span,
    pub docs: Vec<String>,
}

impl RequiredLocaleMessage {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
            docs: Vec::new(),
        }
    }

    pub fn with_docs(mut self, docs: &[DocComment]) -> Self {
        self.docs = docs.iter().map(|doc| doc.text.trim().to_owned()).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementedLocaleMessage {
    pub name: String,
    pub span: Span,
    pub docs: Vec<String>,
}

impl ImplementedLocaleMessage {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
            docs: Vec::new(),
        }
    }

    pub fn with_docs(mut self, docs: &[DocComment]) -> Self {
        self.docs = docs.iter().map(|doc| doc.text.trim().to_owned()).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleCoverageOptions {
    pub missing_message_severity: DiagnosticSeverity,
    pub subject: String,
    pub quick_fix_id: Option<String>,
}

impl Default for LocaleCoverageOptions {
    fn default() -> Self {
        Self {
            missing_message_severity: DiagnosticSeverity::Error,
            subject: "locale".to_owned(),
            quick_fix_id: None,
        }
    }
}

pub fn analyze_locale_file(locale: &LocaleFile) -> Vec<Diagnostic> {
    analyze_locale_branch_coverage(None, locale)
}

pub fn analyze_locale_coverage(schema: &SchemaFile, locale: &LocaleFile) -> Vec<Diagnostic> {
    analyze_locale_coverage_with_options(schema, locale, LocaleCoverageOptions::default())
}

pub fn analyze_locale_coverage_with_options(
    schema: &SchemaFile,
    locale: &LocaleFile,
    options: LocaleCoverageOptions,
) -> Vec<Diagnostic> {
    let mut diagnostics = analyze_locale_message_coverage_with_options(
        &schema_public_messages(schema),
        &locale_public_messages(locale),
        locale.span,
        options,
    );
    diagnostics.extend(analyze_locale_branch_coverage(Some(schema), locale));
    diagnostics
}

pub fn analyze_locale_message_coverage(
    schema_messages: &[RequiredLocaleMessage],
    locale_messages: &[ImplementedLocaleMessage],
    locale_span: Span,
) -> Vec<Diagnostic> {
    analyze_locale_message_coverage_with_options(
        schema_messages,
        locale_messages,
        locale_span,
        LocaleCoverageOptions::default(),
    )
}

pub fn analyze_locale_message_coverage_with_options(
    schema_messages: &[RequiredLocaleMessage],
    locale_messages: &[ImplementedLocaleMessage],
    locale_span: Span,
    options: LocaleCoverageOptions,
) -> Vec<Diagnostic> {
    let schema = schema_message_map(schema_messages);
    let locale = locale_message_map(locale_messages);
    let missing = schema_messages
        .iter()
        .filter(|schema_message| !locale.contains_key(schema_message.name.as_str()))
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();

    if !missing.is_empty() {
        diagnostics.push(missing_messages_diagnostic(
            &missing,
            locale_span,
            options.missing_message_severity,
            &options.subject,
            options.quick_fix_id.as_deref(),
        ));
    }

    let unknown = locale_messages
        .iter()
        .filter(|locale_message| !schema.contains_key(locale_message.name.as_str()))
        .collect::<Vec<_>>();

    if !unknown.is_empty() {
        diagnostics.push(unknown_messages_diagnostic(&unknown));
    }

    diagnostics
}

pub fn schema_public_messages(schema: &SchemaFile) -> Vec<RequiredLocaleMessage> {
    let mut messages = Vec::new();
    for declaration in &schema.declarations {
        collect_schema_messages(declaration, None, &mut messages);
    }
    messages
}

pub fn locale_public_messages(locale: &LocaleFile) -> Vec<ImplementedLocaleMessage> {
    let mut messages = Vec::new();
    for declaration in &locale.declarations {
        collect_locale_messages(declaration, None, &mut messages);
    }
    messages
}

fn missing_messages_diagnostic(
    missing: &[&RequiredLocaleMessage],
    locale_span: Span,
    severity: DiagnosticSeverity,
    subject: &str,
    quick_fix_id: Option<&str>,
) -> Diagnostic {
    let names = missing
        .iter()
        .map(|message| message.name.as_str())
        .collect::<Vec<_>>();
    let message = format!(
        "{subject} is missing {} schema {}: {}",
        names.len(),
        pluralize(names.len(), "message", "messages"),
        format_name_list(&names),
    );
    let diagnostic = match severity {
        DiagnosticSeverity::Error => Diagnostic::error(message, Span::new(0, 0)),
        DiagnosticSeverity::Warning => Diagnostic::warning(message, Span::new(0, 0)),
        DiagnosticSeverity::Advice => Diagnostic::advice(message, Span::new(0, 0)),
    }
    .without_source()
    .with_note("add implementations for the missing schema messages");

    let quick_fix = QuickFix::replacement(
        "add missing locale message stubs",
        Replacement {
            span: Span::new(locale_span.end, locale_span.end),
            text: missing_message_stub_text(&names),
        },
    );

    let mut diagnostic = match quick_fix_id {
        Some(id) => diagnostic.with_quick_fix(quick_fix.with_id(id)),
        None => diagnostic.with_quick_fix(quick_fix),
    };

    for name in names {
        diagnostic = diagnostic.with_quick_fix(QuickFix::replacement(
            format!("add locale message stub `{name}`"),
            Replacement {
                span: Span::new(locale_span.end, locale_span.end),
                text: missing_message_stub_text(&[name]),
            },
        ));
    }

    diagnostic
}

fn unknown_messages_diagnostic(unknown: &[&ImplementedLocaleMessage]) -> Diagnostic {
    let names = unknown
        .iter()
        .map(|message| message.name.as_str())
        .collect::<Vec<_>>();
    let mut diagnostic = Diagnostic::error(
        format!(
            "locale implements {} unknown public {}: {}",
            names.len(),
            pluralize(names.len(), "message", "messages"),
            format_name_list(&names),
        ),
        unknown[0].span,
    )
    .with_note("remove these messages or add matching declarations to the schema");

    for message in unknown.iter().skip(1) {
        diagnostic = diagnostic.with_related(
            message.span,
            format!("unknown implementation `{}`", message.name),
        );
    }

    diagnostic
}

fn collect_schema_messages(
    declaration: &SchemaDeclaration,
    group: Option<&str>,
    messages: &mut Vec<RequiredLocaleMessage>,
) {
    match declaration {
        SchemaDeclaration::Message(message) => messages.push(
            RequiredLocaleMessage::new(
                qualified_name(group, &message.name.value),
                message.name.span,
            )
            .with_docs(&message.docs),
        ),
        SchemaDeclaration::Group(group_declaration) => {
            for message in &group_declaration.messages {
                messages.push(
                    RequiredLocaleMessage::new(
                        qualified_name(Some(&group_declaration.name.value), &message.name.value),
                        message.name.span,
                    )
                    .with_docs(&message.docs),
                );
            }
        }
        SchemaDeclaration::Enum(_) | SchemaDeclaration::TypeAlias(_) => {}
    }
}

fn collect_locale_messages(
    declaration: &LocaleDeclaration,
    group: Option<&str>,
    messages: &mut Vec<ImplementedLocaleMessage>,
) {
    match declaration {
        LocaleDeclaration::Message(message) => messages.push(
            ImplementedLocaleMessage::new(
                qualified_name(group, &message.name.value),
                message.name.span,
            )
            .with_docs(&message.docs),
        ),
        LocaleDeclaration::Group(group_declaration) => {
            for message in &group_declaration.messages {
                messages.push(
                    ImplementedLocaleMessage::new(
                        qualified_name(Some(&group_declaration.name.value), &message.name.value),
                        message.name.span,
                    )
                    .with_docs(&message.docs),
                );
            }
        }
        LocaleDeclaration::Override(inner) => collect_locale_messages(inner, group, messages),
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Function(_) => {}
    }
}

fn qualified_name(group: Option<&str>, name: &str) -> String {
    match group {
        Some(group) => format!("{group}.{name}"),
        None => name.to_owned(),
    }
}
