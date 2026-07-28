use crate::Diagnostic;
use linguini_syntax::Span;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicMessage {
    pub name: String,
    pub span: Span,
}

impl PublicMessage {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

pub fn analyze_message_coverage(
    schema_messages: &[PublicMessage],
    locale_messages: &[PublicMessage],
) -> Vec<Diagnostic> {
    let schema = message_map(schema_messages);
    let locale = message_map(locale_messages);
    let mut diagnostics = duplicate_messages("schema", schema_messages);
    diagnostics.extend(duplicate_messages("locale", locale_messages));

    for schema_message in schema_messages {
        if !locale.contains_key(schema_message.name.as_str()) {
            diagnostics.push(
                Diagnostic::error(
                    format!(
                        "missing locale implementation for public message `{}`",
                        schema_message.name
                    ),
                    schema_message.span,
                )
                .with_code("linguini.missing_message")
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
            .with_code("linguini.unknown_message")
            .with_note("remove this message or add it to the schema"),
        );
    }

    diagnostics
}

fn duplicate_messages(scope: &str, messages: &[PublicMessage]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeMap::new();
    for message in messages {
        if let Some(first) = seen.insert(message.name.as_str(), message.span) {
            diagnostics.push(
                Diagnostic::error(
                    format!("duplicate {scope} public message `{}`", message.name),
                    message.span,
                )
                .with_code("linguini.duplicate_message")
                .with_related(first, "first public message is here"),
            );
        }
    }
    diagnostics
}

fn message_map(messages: &[PublicMessage]) -> BTreeMap<&str, &PublicMessage> {
    messages
        .iter()
        .map(|message| (message.name.as_str(), message))
        .collect()
}
