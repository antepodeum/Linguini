mod plural_hover;
mod semantic;
mod symbols;
mod tokens;

use linguini_analyzer::{
    analyze_locale_coverage_with_options, analyze_locale_file, schema_public_messages, Diagnostic,
    DiagnosticSeverity, LocaleCoverageOptions,
};
use linguini_format::{format_source, FormatOptions, SourceKind};
use linguini_schema::build_schema_symbols_from_files;
use linguini_syntax::{
    parse_locale_with_recovery_in, parse_schema_with_recovery_in, validate_locale_ast,
    validate_schema_ast, LocaleDeclaration, LocaleFile, ParseOutput, SchemaDeclaration, SchemaFile,
    SourceId, Span, Token,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, OnceLock};

use self::plural_hover::plural_branch_hover;
use self::semantic::{
    definition_occurrence, occurrence_at, occurrences, rename_occurrences, resolved_occurrences,
    SemanticKey,
};
use self::symbols::symbols;
use self::tokens::{base_keywords, is_placeholder_context, semantic_token_type, tokens};

const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_NESTING_DEPTH: usize = 256;

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParsedDocument {
    Schema(ParseOutput<SchemaFile>),
    Locale(ParseOutput<LocaleFile>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LinguiniDocument {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub kind: SourceKind,
    pub source_id: SourceId,
    pub namespace: Option<String>,
    pub locale: Option<String>,
    pub version: Option<i32>,
    line_starts: Vec<usize>,
    parsed: Arc<OnceLock<ParsedDocument>>,
    token_cache: Arc<OnceLock<Vec<Token>>>,
    semantic_occurrences: Arc<OnceLock<Vec<semantic::SemanticOccurrence>>>,
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
pub struct WorkspaceReference {
    pub uri: String,
    pub span: Span,
    pub declaration: bool,
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
        let kind = match language_id.as_str() {
            "linguini-schema" | "lgs" => SourceKind::Schema,
            "linguini-locale" | "lgl" => SourceKind::Locale,
            unsupported => panic!("unsupported Linguini language ID `{unsupported}`"),
        };
        let text = text.into();
        let line_starts = line_starts(&text);
        Self {
            uri: uri.into(),
            language_id,
            text,
            kind,
            source_id: SourceId::default(),
            namespace: None,
            locale: None,
            version: None,
            line_starts,
            parsed: Arc::new(OnceLock::new()),
            token_cache: Arc::new(OnceLock::new()),
            semantic_occurrences: Arc::new(OnceLock::new()),
        }
    }

    pub fn try_new(
        uri: impl Into<String>,
        language_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Option<Self> {
        let language_id = language_id.into();
        let text = text.into();
        if !matches!(
            language_id.as_str(),
            "linguini-schema" | "lgs" | "linguini-locale" | "lgl"
        ) || text.len() > MAX_DOCUMENT_BYTES
        {
            return None;
        }
        Some(Self::new(uri, language_id, text))
    }

    pub fn with_source_identity(
        mut self,
        namespace: impl Into<String>,
        locale: Option<String>,
    ) -> Self {
        self.namespace = Some(namespace.into());
        self.locale = locale;
        self
    }

    pub fn with_source_id(mut self, source_id: SourceId) -> Self {
        self.set_source_id(source_id);
        self
    }

    pub(crate) fn set_source_id(&mut self, source_id: SourceId) {
        if self.source_id == source_id {
            return;
        }
        self.source_id = source_id;
        self.parsed = Arc::new(OnceLock::new());
        self.token_cache = Arc::new(OnceLock::new());
        self.semantic_occurrences = Arc::new(OnceLock::new());
    }

    pub fn with_version(mut self, version: i32) -> Self {
        self.version = Some(version);
        self
    }

    pub fn is_within_safety_limits(&self) -> bool {
        !document_too_complex(self)
    }

    pub fn position(&self, offset: usize) -> (u32, u32) {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
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

pub(crate) fn prime_semantic_cache(document: &LinguiniDocument) {
    let _ = occurrences(document);
}

fn parsed_document(document: &LinguiniDocument) -> &ParsedDocument {
    document.parsed.get_or_init(|| match document.kind {
        SourceKind::Schema => ParsedDocument::Schema(parse_schema_with_recovery_in(
            &document.text,
            document.source_id,
        )),
        SourceKind::Locale => ParsedDocument::Locale(parse_locale_with_recovery_in(
            &document.text,
            document.source_id,
        )),
    })
}

fn parsed_schema(document: &LinguiniDocument) -> Option<&ParseOutput<SchemaFile>> {
    match parsed_document(document) {
        ParsedDocument::Schema(parsed) => Some(parsed),
        ParsedDocument::Locale(_) => None,
    }
}

fn parsed_locale(document: &LinguiniDocument) -> Option<&ParseOutput<LocaleFile>> {
    match parsed_document(document) {
        ParsedDocument::Locale(parsed) => Some(parsed),
        ParsedDocument::Schema(_) => None,
    }
}

pub fn diagnostics(document: &LinguiniDocument) -> Vec<Diagnostic> {
    diagnostics_with_workspace(document, [])
}

pub fn diagnostics_with_workspace(
    document: &LinguiniDocument,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<Diagnostic> {
    if document_too_complex(document) {
        let end = document
            .text
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0);
        return vec![Diagnostic::error(
            "document exceeds Linguini language-server safety limits",
            Span::new(0, end),
        )];
    }
    match document.kind {
        SourceKind::Schema => {
            let parsed = parsed_schema(document).expect("schema document has schema parse cache");
            let mut diagnostics = parse_diagnostics("schema", "syntax", parsed.errors.clone());
            if let Some(schema) = parsed.ast.as_ref() {
                diagnostics.extend(parse_diagnostics(
                    "schema",
                    "validation",
                    validate_schema_ast(schema),
                ));
                diagnostics.extend(schema_builder_diagnostics(document, workspace));
            }
            deduplicate_diagnostics(diagnostics)
        }
        SourceKind::Locale => {
            let parsed = parsed_locale(document).expect("locale document has locale parse cache");
            let mut diagnostics = parse_diagnostics("locale", "syntax", parsed.errors.clone());
            let Some(locale) = parsed.ast.as_ref() else {
                return diagnostics;
            };
            diagnostics.extend(parse_diagnostics(
                "locale",
                "validation",
                validate_locale_ast(locale),
            ));
            let schemas = matching_schema_documents(document, workspace);

            if schemas.is_empty() {
                diagnostics.extend(analyze_locale_file(locale));
            } else {
                for schema_document in schemas {
                    let Some(schema) =
                        parsed_schema(&schema_document).and_then(|parsed| parsed.ast.as_ref())
                    else {
                        continue;
                    };
                    diagnostics.extend(analyze_locale_coverage_with_options(
                        schema,
                        locale,
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
    }
}

fn parse_diagnostics(
    source_kind: &str,
    category: &str,
    errors: Vec<linguini_syntax::ParseError>,
) -> Vec<Diagnostic> {
    errors
        .into_iter()
        .map(|error| {
            Diagnostic::error(
                format!("{source_kind} {category} error: {}", error.message),
                error.span,
            )
        })
        .collect()
}

pub fn completion_items(document: &LinguiniDocument, offset: usize) -> Vec<String> {
    completion_items_with_workspace(document, offset, [])
}

pub fn completion_items_with_workspace(
    document: &LinguiniDocument,
    offset: usize,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<String> {
    if !document.is_within_safety_limits() {
        return Vec::new();
    }
    let mut items = base_keywords(document.kind);
    items.extend(symbols(document).into_iter().map(|symbol| symbol.name));
    items.extend(
        occurrences(document)
            .into_iter()
            .map(|occurrence| match occurrence.key {
                SemanticKey::Message(name)
                | SemanticKey::Type(name)
                | SemanticKey::Variable(name)
                | SemanticKey::Function(name) => name,
                SemanticKey::EnumVariant { variant, .. } => variant,
                SemanticKey::FormAttribute { path, .. } => path,
                SemanticKey::Parameter { name, .. } => name,
            }),
    );

    if is_placeholder_context(&document.text, offset) {
        let schemas = matching_schema_documents(document, workspace);
        items.extend(schemas.iter().flat_map(symbols).map(|symbol| symbol.name));
        if let Some(path) = locale_message_path_containing(document, offset) {
            for schema in schemas {
                let Some(schema) = parsed_schema(&schema).and_then(|parsed| parsed.ast.as_ref())
                else {
                    continue;
                };
                let Some(message) = schema_message_by_path(schema, &path) else {
                    continue;
                };
                items.extend(
                    message
                        .parameters
                        .iter()
                        .map(|parameter| parameter.name.value.clone()),
                );
            }
        }
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
    if !document.is_within_safety_limits() {
        return None;
    }
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
            if let Some(docs) = schema_docs_for_message(
                matching_schema_documents(document, workspace.clone()),
                name,
            ) {
                symbol.docs = docs;
            }
        }
    }
    if let Some(name) = &message_name {
        if let Some(signature) = schema_signature_for_message(
            matching_schema_documents(document, workspace.clone()),
            name,
        ) {
            symbol.preview = Some(match symbol.preview {
                Some(locale_preview) => format!("{signature}\n{locale_preview}"),
                None => signature,
            });
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
    references_at_with_workspace(document, offset, [document.clone()])
        .into_iter()
        .filter(|reference| reference.uri == document.uri)
        .map(|reference| reference.span)
        .collect()
}

pub fn references_at_with_workspace(
    document: &LinguiniDocument,
    offset: usize,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<WorkspaceReference> {
    if !document.is_within_safety_limits() {
        return Vec::new();
    }
    resolved_occurrences(workspace, document, offset)
        .unwrap_or_default()
        .into_iter()
        .map(|resolved| WorkspaceReference {
            uri: resolved.document.uri,
            span: resolved.occurrence.span,
            declaration: resolved.occurrence.declaration,
        })
        .collect()
}

pub fn definition_at_with_workspace(
    document: &LinguiniDocument,
    offset: usize,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Option<(String, Span)> {
    if !document.is_within_safety_limits() {
        return None;
    }
    definition_occurrence(workspace, document, offset)
        .map(|resolved| (resolved.document.uri, resolved.occurrence.span))
}

pub fn prepare_rename_at(document: &LinguiniDocument, offset: usize) -> Option<Span> {
    if !document.is_within_safety_limits() {
        return None;
    }
    occurrence_at(document, offset).map(|occurrence| occurrence.span)
}

pub fn rename_workspace_edits(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
    new_name: &str,
) -> Vec<WorkspaceTextEdit> {
    if !source.is_within_safety_limits() {
        return Vec::new();
    }
    rename_occurrences(documents, source, offset, new_name)
        .unwrap_or_default()
        .into_iter()
        .map(|resolved| WorkspaceTextEdit {
            uri: resolved.document.uri,
            edit: TextEdit {
                span: resolved.occurrence.span,
                new_text: new_name.to_owned(),
            },
        })
        .collect()
}

pub fn document_symbols(document: &LinguiniDocument) -> Vec<Symbol> {
    if !document.is_within_safety_limits() {
        return Vec::new();
    }
    symbols(document)
}

pub fn workspace_symbols(documents: impl IntoIterator<Item = LinguiniDocument>) -> Vec<Symbol> {
    documents
        .into_iter()
        .flat_map(|document| symbols(&document))
        .collect()
}

pub fn semantic_tokens(document: &LinguiniDocument) -> Vec<LinguiniSemanticToken> {
    if !document.is_within_safety_limits() {
        return Vec::new();
    }
    let source_tokens = tokens(document);
    let semantic_types = occurrences(document)
        .into_iter()
        .map(|occurrence| {
            let token_type = match occurrence.key {
                SemanticKey::Type(_) => 2,
                SemanticKey::EnumVariant { .. } => 3,
                SemanticKey::Function(_) | SemanticKey::FormAttribute { .. } => 7,
                SemanticKey::Message(_)
                | SemanticKey::Variable(_)
                | SemanticKey::Parameter { .. } => 1,
            };
            ((occurrence.span.start, occurrence.span.end), token_type)
        })
        .collect::<BTreeMap<_, _>>();
    let mut raw = Vec::new();
    for (index, token) in source_tokens.iter().enumerate() {
        let token_type = semantic_types
            .get(&(token.span.start, token.span.end))
            .copied()
            .or_else(|| semantic_token_type(&source_tokens, index));
        let Some(token_type) = token_type else {
            continue;
        };
        raw.extend(semantic_tokens_for_span(document, token.span, token_type));
    }

    raw.sort_by_key(|token| (token.line, token.start));
    raw
}

pub fn format_document(
    document: &LinguiniDocument,
) -> Result<TextEdit, linguini_format::FormatError> {
    format_document_with_options(document, &FormatOptions::default())
}

pub fn format_document_with_options(
    document: &LinguiniDocument,
    options: &FormatOptions,
) -> Result<TextEdit, linguini_format::FormatError> {
    let formatted = format_source(document.kind, &document.text, options)?;
    Ok(TextEdit {
        span: Span::new(0, document.text.len()),
        new_text: formatted,
    })
}

pub(super) fn contains(span: Span, offset: usize) -> bool {
    span.start <= offset && offset < span.end
}

fn schema_docs_for_message(
    workspace: impl IntoIterator<Item = LinguiniDocument>,
    name: &str,
) -> Option<Vec<String>> {
    for candidate in workspace {
        if candidate.kind != SourceKind::Schema {
            continue;
        }
        let Some(schema) = parsed_schema(&candidate).and_then(|parsed| parsed.ast.as_ref()) else {
            continue;
        };
        let Some(message) = schema_public_messages(schema)
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

fn schema_signature_for_message(
    workspace: impl IntoIterator<Item = LinguiniDocument>,
    name: &str,
) -> Option<String> {
    for candidate in workspace {
        let schema = parsed_schema(&candidate)?.ast.as_ref()?;
        if let Some(message) = schema_message_by_path(schema, name) {
            let parameters = message
                .parameters
                .iter()
                .map(|parameter| format!("{}: {}", parameter.name.value, parameter.ty.value))
                .collect::<Vec<_>>()
                .join(", ");
            return Some(format!("{name}({parameters})"));
        }
    }
    None
}

fn schema_message_by_path<'a>(
    schema: &'a linguini_syntax::SchemaFile,
    path: &str,
) -> Option<&'a linguini_syntax::MessageSignature> {
    let parts = path.split('.').collect::<Vec<_>>();
    if parts.len() == 1 {
        return schema.declarations.iter().find_map(|declaration| {
            let SchemaDeclaration::Message(message) = declaration else {
                return None;
            };
            (message.name.value == parts[0]).then_some(message)
        });
    }
    schema.declarations.iter().find_map(|declaration| {
        let SchemaDeclaration::Group(group) = declaration else {
            return None;
        };
        (group.name.value == parts[0])
            .then(|| schema_group_message_by_path(group, &parts[1..]))
            .flatten()
    })
}

fn schema_group_message_by_path<'a>(
    group: &'a linguini_syntax::MessageGroup,
    parts: &[&str],
) -> Option<&'a linguini_syntax::MessageSignature> {
    let (head, tail) = parts.split_first()?;
    if tail.is_empty() {
        return group
            .messages
            .iter()
            .find(|message| message.name.value == *head);
    }
    group
        .groups
        .iter()
        .find(|child| child.name.value == *head)
        .and_then(|child| schema_group_message_by_path(child, tail))
}

fn matching_schema_documents(
    document: &LinguiniDocument,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<LinguiniDocument> {
    let schemas = workspace
        .into_iter()
        .filter(|candidate| candidate.kind == SourceKind::Schema)
        .collect::<Vec<_>>();
    if let Some(namespace) = &document.namespace {
        return schemas
            .into_iter()
            .filter(|candidate| candidate.namespace.as_ref() == Some(namespace))
            .collect();
    }

    if schemas.len() == 1 {
        schemas
    } else {
        Vec::new()
    }
}

fn schema_builder_diagnostics(
    document: &LinguiniDocument,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<Diagnostic> {
    let mut documents = if document.namespace.is_some() {
        matching_schema_documents(document, workspace)
    } else {
        Vec::new()
    };
    if !documents
        .iter()
        .any(|candidate| candidate.uri == document.uri)
    {
        documents.push(document.clone());
    }
    let schemas = documents
        .iter()
        .filter_map(|candidate| parsed_schema(candidate)?.ast.clone())
        .collect::<Vec<_>>();
    let (_, diagnostics) = build_schema_symbols_from_files(&schemas);
    diagnostics
        .into_iter()
        .filter(|diagnostic| {
            diagnostic
                .source_span
                .is_some_and(|span| span.source == document.source_id)
        })
        .collect()
}

fn deduplicate_diagnostics(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    diagnostics
        .into_iter()
        .filter(|diagnostic| {
            let location = diagnostic
                .source_span
                .map(|span| (span.source, span.start, span.end));
            seen.insert((location, diagnostic.message.clone()))
        })
        .collect()
}

fn document_too_complex(document: &LinguiniDocument) -> bool {
    if document.text.len() > MAX_DOCUMENT_BYTES {
        return true;
    }
    let mut depth = 0usize;
    for character in document.text.chars() {
        match character {
            '{' | '(' => {
                depth += 1;
                if depth > MAX_NESTING_DEPTH {
                    return true;
                }
            }
            '}' | ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

fn semantic_tokens_for_span(
    document: &LinguiniDocument,
    span: Span,
    token_type: u32,
) -> Vec<LinguiniSemanticToken> {
    let mut output = Vec::new();
    let mut segment_start = span.start.min(document.text.len());
    let span_end = span.end.min(document.text.len());
    while segment_start < span_end {
        let newline = document.text[segment_start..span_end]
            .find('\n')
            .map(|relative| segment_start + relative);
        let mut segment_end = newline.unwrap_or(span_end);
        if segment_end > segment_start
            && document.text.as_bytes().get(segment_end - 1) == Some(&b'\r')
        {
            segment_end -= 1;
        }
        if segment_end > segment_start {
            let (line, start) = document.position(segment_start);
            let (_, end) = document.position(segment_end);
            output.push(LinguiniSemanticToken {
                line,
                start,
                length: end.saturating_sub(start),
                token_type,
                modifiers: 0,
            });
        }
        let Some(newline) = newline else {
            break;
        };
        segment_start = newline + 1;
    }
    output
}

fn locale_message_name_at(document: &LinguiniDocument, offset: usize) -> Option<String> {
    let locale = parsed_locale(document)?.ast.as_ref()?;
    for declaration in &locale.declarations {
        if let Some(name) = locale_declaration_message_name_at(declaration, None, offset) {
            return Some(name);
        }
    }
    None
}

fn locale_message_path_containing(document: &LinguiniDocument, offset: usize) -> Option<String> {
    let locale = parsed_locale(document)?.ast.as_ref()?;
    for declaration in &locale.declarations {
        if let Some(path) = locale_declaration_message_containing(declaration, None, offset) {
            return Some(path);
        }
    }
    None
}

fn locale_declaration_message_containing(
    declaration: &LocaleDeclaration,
    parent: Option<&str>,
    offset: usize,
) -> Option<String> {
    match declaration {
        LocaleDeclaration::Message(message) if contains(message.value.span, offset) => {
            Some(qualified_name(parent, &message.name.value))
        }
        LocaleDeclaration::Group(group) => locale_group_message_containing(group, parent, offset),
        LocaleDeclaration::Override(inner) => {
            locale_declaration_message_containing(inner, parent, offset)
        }
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Function(_)
        | LocaleDeclaration::Message(_) => None,
    }
}

fn locale_group_message_containing(
    group: &linguini_syntax::MessageImplementationGroup,
    parent: Option<&str>,
    offset: usize,
) -> Option<String> {
    let path = qualified_name(parent, &group.name.value);
    for message in &group.messages {
        if contains(message.value.span, offset) {
            return Some(qualified_name(Some(&path), &message.name.value));
        }
    }
    for child in &group.groups {
        if let Some(message) = locale_group_message_containing(child, Some(&path), offset) {
            return Some(message);
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
            locale_group_message_name_at(group_declaration, group, offset)
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

fn locale_group_message_name_at(
    group: &linguini_syntax::MessageImplementationGroup,
    parent: Option<&str>,
    offset: usize,
) -> Option<String> {
    let path = qualified_name(parent, &group.name.value);
    for message in &group.messages {
        if contains(message.name.span, offset) {
            return Some(qualified_name(Some(&path), &message.name.value));
        }
    }
    for child in &group.groups {
        if let Some(name) = locale_group_message_name_at(child, Some(&path), offset) {
            return Some(name);
        }
    }
    None
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
