use crate::{Diagnostic, DiagnosticSeverity, QuickFix, Replacement};
use linguini_syntax::{LocaleDeclaration, LocaleFile, SchemaDeclaration, SchemaFile, Span};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredLocaleMessage {
    pub name: String,
    pub span: Span,
}

impl RequiredLocaleMessage {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementedLocaleMessage {
    pub name: String,
    pub span: Span,
}

impl ImplementedLocaleMessage {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
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

pub fn analyze_locale_file(_locale: &LocaleFile) -> Vec<Diagnostic> {
    Vec::new()
}

pub fn analyze_locale_coverage(schema: &SchemaFile, locale: &LocaleFile) -> Vec<Diagnostic> {
    analyze_locale_coverage_with_options(schema, locale, LocaleCoverageOptions::default())
}

pub fn analyze_locale_coverage_with_options(
    schema: &SchemaFile,
    locale: &LocaleFile,
    options: LocaleCoverageOptions,
) -> Vec<Diagnostic> {
    analyze_locale_message_coverage_with_options(
        &schema_public_messages(schema),
        &locale_public_messages(locale),
        locale.span,
        options,
    )
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

    match quick_fix_id {
        Some(id) => diagnostic.with_quick_fix(quick_fix.with_id(id)),
        None => diagnostic.with_quick_fix(quick_fix),
    }
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
        SchemaDeclaration::Message(message) => messages.push(RequiredLocaleMessage::new(
            qualified_name(group, &message.name.value),
            message.name.span,
        )),
        SchemaDeclaration::Group(group_declaration) => {
            for message in &group_declaration.messages {
                messages.push(RequiredLocaleMessage::new(
                    qualified_name(Some(&group_declaration.name.value), &message.name.value),
                    message.name.span,
                ));
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
        LocaleDeclaration::Message(message) => messages.push(ImplementedLocaleMessage::new(
            qualified_name(group, &message.name.value),
            message.name.span,
        )),
        LocaleDeclaration::Group(group_declaration) => {
            for message in &group_declaration.messages {
                messages.push(ImplementedLocaleMessage::new(
                    qualified_name(Some(&group_declaration.name.value), &message.name.value),
                    message.name.span,
                ));
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

fn schema_message_map(
    messages: &[RequiredLocaleMessage],
) -> BTreeMap<&str, &RequiredLocaleMessage> {
    messages
        .iter()
        .map(|message| (message.name.as_str(), message))
        .collect()
}

fn locale_message_map(
    messages: &[ImplementedLocaleMessage],
) -> BTreeMap<&str, &ImplementedLocaleMessage> {
    messages
        .iter()
        .map(|message| (message.name.as_str(), message))
        .collect()
}

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

fn format_name_list(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn missing_message_stub_text(names: &[&str]) -> String {
    let mut output = String::new();
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut top_level = Vec::new();

    for name in names {
        if let Some((group, message)) = name.split_once('.') {
            groups.entry(group).or_default().push(message);
        } else {
            top_level.push(*name);
        }
    }

    if !top_level.is_empty() || !groups.is_empty() {
        output.push('\n');
    }

    for name in top_level {
        output.push_str(&format!("{name} = TODO\n"));
    }

    for (group, messages) in groups {
        output.push_str(&format!("{group} {{\n"));
        for message in messages {
            output.push_str(&format!("  {message} = TODO\n"));
        }
        output.push_str("}\n");
    }

    output
}
