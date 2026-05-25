mod plural_hover;
mod sample;
mod symbols;
mod tokens;

use linguini_analyzer::{
    analyze_locale_coverage_with_options, analyze_locale_file, schema_public_messages, Diagnostic,
    DiagnosticSeverity, LocaleCoverageOptions,
};
use linguini_format::{format_source, FormatOptions, SourceKind};
use linguini_syntax::{
    parse_locale_with_recovery, parse_schema_with_recovery, LocaleDeclaration, Span,
};

use self::plural_hover::plural_branch_hover;
use self::sample::render_message_sample;
use self::symbols::symbols;
use self::tokens::{
    base_keywords, identifier_at, is_placeholder_context, matching_identifier_spans,
    semantic_token_type, tokens,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LinguiniDocument {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub kind: SourceKind,
    line_starts: Vec<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub detail: String,
    pub span: Span,
    pub docs: Vec<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextEdit {
    pub span: Span,
    pub new_text: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceTextEdit {
    pub uri: String,
    pub edit: TextEdit,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LinguiniSemanticToken {
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub token_type: u32,
    pub modifiers: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SemanticLegend;

impl SemanticLegend {
    pub const TYPES: [&'static str; 8] = [
        "keyword",
        "variable",
        "enum",
        "enumMember",
        "string",
        "comment",
        "operator",
        "function",
    ];
}

impl LinguiniDocument {
    pub fn new(
        uri: impl Into<String>,
        language_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        let language_id = language_id.into();
        let kind = if language_id == "linguini-schema" || language_id == "lgs" {
            SourceKind::Schema
        } else {
            SourceKind::Locale
        };
        let text = text.into();
        let line_starts = line_starts(&text);
        Self {
            uri: uri.into(),
            language_id,
            text,
            kind,
            line_starts,
        }
    }

    pub fn position(&self, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.text.len());
        let line = self.line_starts.partition_point(|start| *start <= offset) - 1;
        let line_start = self.line_starts[line];
        let character = utf16_len(&self.text[line_start..offset]);
        (line as u32, character as u32)
    }

    pub fn offset(&self, line: u32, character: u32) -> usize {
        let line = line as usize;
        let line_start = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.text.len());
        let line_end = line_end(&self.text, &self.line_starts, line);
        utf16_column_to_offset(&self.text, line_start, line_end, character as usize)
    }

    pub fn range(&self, span: Span) -> ((u32, u32), (u32, u32)) {
        (self.position(span.start), self.position(span.end))
    }
}

pub fn diagnostics(document: &LinguiniDocument) -> Vec<Diagnostic> {
    diagnostics_with_workspace(document, [])
}

pub fn diagnostics_with_workspace(
    document: &LinguiniDocument,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<Diagnostic> {
    let errors = match document.kind {
        SourceKind::Schema => parse_schema_with_recovery(&document.text).errors,
        SourceKind::Locale => parse_locale_with_recovery(&document.text).errors,
    };

    let mut diagnostics = errors
        .into_iter()
        .map(|error| {
            Diagnostic::error(
                format!(
                    "{} syntax error: {}",
                    match document.kind {
                        SourceKind::Schema => "schema",
                        SourceKind::Locale => "locale",
                    },
                    error.message
                ),
                error.span,
            )
        })
        .collect::<Vec<_>>();

    if !diagnostics.is_empty() {
        return diagnostics;
    }

    let SourceKind::Locale = document.kind else {
        return diagnostics;
    };
    let parsed_locale = parse_locale_with_recovery(&document.text);
    let Some(locale) = parsed_locale.ast else {
        return diagnostics;
    };

    let schemas = workspace
        .into_iter()
        .filter(|candidate| candidate.kind == SourceKind::Schema)
        .filter_map(|candidate| parse_schema_with_recovery(&candidate.text).ast)
        .collect::<Vec<_>>();

    if schemas.is_empty() {
        diagnostics.extend(analyze_locale_file(&locale));
    } else {
        for schema in schemas {
            diagnostics.extend(analyze_locale_coverage_with_options(
                &schema,
                &locale,
                LocaleCoverageOptions {
                    missing_message_severity: DiagnosticSeverity::Warning,
                    subject: "locale".to_owned(),
                    quick_fix_id: Some("linguini.addMissingLocaleMessages".to_owned()),
                },
            ));
        }
    }

    diagnostics
}

pub fn completion_items(document: &LinguiniDocument, offset: usize) -> Vec<String> {
    let mut items = base_keywords(document.kind);
    items.extend(symbols(document).into_iter().map(|symbol| symbol.name));

    if is_placeholder_context(&document.text, offset) {
        items.extend(["count".to_owned(), "other".to_owned(), "_".to_owned()]);
    }

    items.sort();
    items.dedup();
    items
}

pub fn hover_at(document: &LinguiniDocument, offset: usize) -> Option<String> {
    hover_at_with_workspace(document, offset, [])
}

pub fn hover_at_with_workspace(
    document: &LinguiniDocument,
    offset: usize,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Option<String> {
    let workspace = workspace.into_iter().collect::<Vec<_>>();
    if let Some(hover) = plural_branch_hover(document, offset) {
        return Some(hover);
    }

    let mut symbol = symbols(document)
        .into_iter()
        .find(|symbol| contains(symbol.span, offset))?;

    let message_name = (document.kind == SourceKind::Locale)
        .then(|| locale_message_name_at(document, offset))
        .flatten();

    if document.kind == SourceKind::Locale && symbol.docs.is_empty() {
        if let Some(name) = &message_name {
            if let Some(docs) = schema_docs_for_message(workspace.clone(), name) {
                symbol.docs = docs;
            }
        }
    }
    if let Some(name) = message_name {
        if let Some(sample) = render_message_sample(document, workspace, &name) {
            symbol.preview = Some(format!("{}\n=> {}", sample.signature, sample.rendered));
        }
    }

    let mut parts = Vec::new();
    parts.push(format!("{} `{}`", symbol.detail, symbol.name));
    if !symbol.docs.is_empty() {
        parts.push(symbol.docs.join("\n"));
    }
    if let Some(preview) = symbol.preview {
        parts.push(format!("Sample\n\n```text\n{preview}\n```"));
    }
    Some(parts.join("\n\n"))
}

pub fn references_at(document: &LinguiniDocument, offset: usize) -> Vec<Span> {
    let Some((word, _)) = identifier_at(document, offset) else {
        return Vec::new();
    };
    matching_identifier_spans(document, &word)
}

pub fn definition_at_with_workspace(
    document: &LinguiniDocument,
    offset: usize,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Option<(String, Span)> {
    if document.kind == SourceKind::Locale {
        if let Some(name) = locale_message_name_at(document, offset) {
            for candidate in workspace {
                if candidate.kind != SourceKind::Schema {
                    continue;
                }
                let Some(schema) = parse_schema_with_recovery(&candidate.text).ast else {
                    continue;
                };
                let Some(message) = schema_public_messages(&schema)
                    .into_iter()
                    .find(|message| message.name == name)
                else {
                    continue;
                };
                return Some((candidate.uri, message.span));
            }
        }
    }

    references_at(document, offset)
        .into_iter()
        .next()
        .map(|span| (document.uri.clone(), span))
}

pub fn prepare_rename_at(document: &LinguiniDocument, offset: usize) -> Option<Span> {
    identifier_at(document, offset).map(|(_, span)| span)
}

pub fn rename_workspace_edits(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
    new_name: &str,
) -> Vec<WorkspaceTextEdit> {
    let Some((word, _)) = identifier_at(source, offset) else {
        return Vec::new();
    };

    documents
        .into_iter()
        .flat_map(|document| {
            matching_identifier_spans(&document, &word)
                .into_iter()
                .map(move |span| WorkspaceTextEdit {
                    uri: document.uri.clone(),
                    edit: TextEdit {
                        span,
                        new_text: new_name.to_owned(),
                    },
                })
        })
        .collect()
}

pub fn document_symbols(document: &LinguiniDocument) -> Vec<Symbol> {
    symbols(document)
}

pub fn workspace_symbols(documents: impl IntoIterator<Item = LinguiniDocument>) -> Vec<Symbol> {
    documents
        .into_iter()
        .flat_map(|document| symbols(&document))
        .collect()
}

pub fn semantic_tokens(document: &LinguiniDocument) -> Vec<LinguiniSemanticToken> {
    let source_tokens = tokens(document);
    let mut raw = Vec::new();
    for (index, token) in source_tokens.iter().enumerate() {
        let Some(token_type) = semantic_token_type(&source_tokens, index) else {
            continue;
        };
        let (line, start) = document.position(token.span.start);
        let (end_line, end) = document.position(token.span.end);
        if end_line != line {
            continue;
        }
        raw.push(LinguiniSemanticToken {
            line,
            start,
            length: end.saturating_sub(start),
            token_type,
            modifiers: 0,
        });
    }

    raw.sort_by_key(|token| (token.line, token.start));
    raw
}

pub fn format_document(
    document: &LinguiniDocument,
) -> Result<TextEdit, linguini_format::FormatError> {
    let formatted = format_source(document.kind, &document.text, &FormatOptions::default())?;
    Ok(TextEdit {
        span: Span::new(0, document.text.len()),
        new_text: formatted,
    })
}

pub(super) fn contains(span: Span, offset: usize) -> bool {
    span.start <= offset && offset <= span.end
}

fn schema_docs_for_message(
    workspace: impl IntoIterator<Item = LinguiniDocument>,
    name: &str,
) -> Option<Vec<String>> {
    for candidate in workspace {
        if candidate.kind != SourceKind::Schema {
            continue;
        }
        let Some(schema) = parse_schema_with_recovery(&candidate.text).ast else {
            continue;
        };
        let Some(message) = schema_public_messages(&schema)
            .into_iter()
            .find(|message| message.name == name)
        else {
            continue;
        };
        if !message.docs.is_empty() {
            return Some(message.docs);
        }
    }
    None
}

fn locale_message_name_at(document: &LinguiniDocument, offset: usize) -> Option<String> {
    let locale = parse_locale_with_recovery(&document.text).ast?;
    for declaration in &locale.declarations {
        if let Some(name) = locale_declaration_message_name_at(declaration, None, offset) {
            return Some(name);
        }
    }
    None
}

fn locale_declaration_message_name_at(
    declaration: &LocaleDeclaration,
    group: Option<&str>,
    offset: usize,
) -> Option<String> {
    match declaration {
        LocaleDeclaration::Message(message) if contains(message.name.span, offset) => {
            Some(qualified_name(group, &message.name.value))
        }
        LocaleDeclaration::Group(group_declaration) => {
            for message in &group_declaration.messages {
                if contains(message.name.span, offset) {
                    return Some(qualified_name(
                        Some(&group_declaration.name.value),
                        &message.name.value,
                    ));
                }
            }
            None
        }
        LocaleDeclaration::Override(inner) => {
            locale_declaration_message_name_at(inner, group, offset)
        }
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Function(_) => None,
        LocaleDeclaration::Message(_) => None,
    }
}

fn qualified_name(group: Option<&str>, name: &str) -> String {
    match group {
        Some(group) => format!("{group}.{name}"),
        None => name.to_owned(),
    }
}

fn utf16_len(text: &str) -> usize {
    text.chars().map(|ch| ch.len_utf16()).sum()
}

fn utf16_column_to_offset(
    source: &str,
    line_start: usize,
    line_end: usize,
    column: usize,
) -> usize {
    let mut units = 0usize;
    for (relative_offset, ch) in source[line_start..line_end].char_indices() {
        let next_units = units + ch.len_utf16();
        if next_units > column {
            return line_start + relative_offset;
        }
        if next_units == column {
            return line_start + relative_offset + ch.len_utf8();
        }
        units = next_units;
    }
    line_end
}

fn line_end(source: &str, line_starts: &[usize], line: usize) -> usize {
    let mut end = line_starts.get(line + 1).copied().unwrap_or(source.len());
    let bytes = source.as_bytes();
    if end > 0 && bytes[end - 1] == b'\n' {
        end -= 1;
    }
    if end > 0 && bytes[end - 1] == b'\r' {
        end -= 1;
    }
    end
}

fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}
