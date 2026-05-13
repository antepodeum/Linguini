use crate::Diagnostic;
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

pub fn analyze_locale_file(_locale: &LocaleFile) -> Vec<Diagnostic> {
    Vec::new()
}

pub fn analyze_locale_coverage(schema: &SchemaFile, locale: &LocaleFile) -> Vec<Diagnostic> {
    analyze_locale_message_coverage(
        &schema_public_messages(schema),
        &locale_public_messages(locale),
        locale.span,
    )
}

pub fn analyze_locale_message_coverage(
    schema_messages: &[RequiredLocaleMessage],
    locale_messages: &[ImplementedLocaleMessage],
    locale_span: Span,
) -> Vec<Diagnostic> {
    let schema = schema_message_map(schema_messages);
    let locale = locale_message_map(locale_messages);
    let mut diagnostics = Vec::new();

    for schema_message in schema_messages {
        if !locale.contains_key(schema_message.name.as_str()) {
            diagnostics.push(
                Diagnostic::error(
                    format!(
                        "missing locale implementation for schema message `{}`",
                        schema_message.name
                    ),
                    locale_span,
                )
                .with_note("add this message to the locale file"),
            );
        }
    }

    for locale_message in locale_messages {
        if schema.contains_key(locale_message.name.as_str()) {
            continue;
        }

        diagnostics.push(
            Diagnostic::error(
                format!(
                    "unknown public message implementation `{}`",
                    locale_message.name
                ),
                locale_message.span,
            )
            .with_note("remove this message or add it to the schema"),
        );
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
        LocaleDeclaration::Enum(_) | LocaleDeclaration::Form(_) | LocaleDeclaration::Function(_) => {}
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
