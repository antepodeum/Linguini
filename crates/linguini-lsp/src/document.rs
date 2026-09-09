mod plural_hover;
mod semantic;
mod symbols;
mod tokens;

use linguini_analyzer::{
    analyze_locale_file, Diagnostic, DiagnosticSeverity, LocaleCoverageOptions,
};
use linguini_format::{format_source, FormatOptions, SourceKind};
use linguini_schema::SchemaDatabase;
use linguini_syntax::{
    parse_locale_with_recovery_in, parse_schema_with_recovery_in, validate_locale_ast,
    validate_schema_ast, FunctionDeclaration, LocaleDeclaration, LocaleFile, ParseOutput,
    SchemaFile, SourceId, Span, Token,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, OnceLock};

use self::plural_hover::plural_branch_hover;
use self::semantic::{
    definition_occurrence, occurrence_at, occurrences, rename_occurrences, resolved_occurrences,
    SemanticKey,
};
use self::symbols::symbols;
use self::tokens::{
    base_keywords, is_placeholder_context, prefix_at_boundary, semantic_token_type, tokens,
};

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CompletionKind {
    Keyword,
    Type,
    Enum,
    EnumMember,
    Function,
    Variable,
    Message,
    Property,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompletionCandidate {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
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
            let schema_asts = schemas
                .iter()
                .filter_map(|schema_document| {
                    parsed_schema(schema_document).and_then(|parsed| parsed.ast.clone())
                })
                .collect::<Vec<_>>();

            if schema_asts.is_empty() {
                diagnostics.extend(analyze_locale_file(locale));
            } else {
                diagnostics.extend(
                    SchemaDatabase::build_from_files(&schema_asts).analyze_locale_with_options(
                        locale,
                        LocaleCoverageOptions {
                            missing_message_severity: DiagnosticSeverity::Warning,
                            subject: "locale".to_owned(),
                            quick_fix_id: Some("linguini.addMissingLocaleMessages".to_owned()),
                        },
                    ),
                );
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
    completion_candidates_with_workspace(document, offset, [])
        .into_iter()
        .map(|candidate| candidate.label)
        .collect()
}

pub fn completion_items_with_workspace(
    document: &LinguiniDocument,
    offset: usize,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<String> {
    completion_candidates_with_workspace(document, offset, workspace)
        .into_iter()
        .map(|candidate| candidate.label)
        .collect()
}

pub fn completion_candidates_with_workspace(
    document: &LinguiniDocument,
    offset: usize,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
) -> Vec<CompletionCandidate> {
    if !document.is_within_safety_limits() {
        return Vec::new();
    }
    let workspace = workspace.into_iter().collect::<Vec<_>>();
    let schemas = matching_schema_documents(document, workspace.clone());
    let mut items = BTreeMap::<String, CompletionCandidate>::new();
    let mut insert = |candidate: CompletionCandidate| {
        items
            .entry(candidate.label.clone())
            .and_modify(|existing| {
                if (existing.kind == CompletionKind::Keyword
                    && candidate.kind != CompletionKind::Keyword)
                    || (candidate
                        .detail
                        .as_deref()
                        .is_some_and(|detail| detail.starts_with("parameter: "))
                        && !existing
                            .detail
                            .as_deref()
                            .is_some_and(|detail| detail.starts_with("parameter: ")))
                {
                    *existing = candidate.clone();
                }
            })
            .or_insert(candidate);
    };

    if schema_type_context(document, offset) {
        for primitive in ["String", "Boolean", "Number", "Decimal", "Date"] {
            insert(completion(
                primitive,
                CompletionKind::Type,
                "primitive type",
            ));
        }
        for schema in std::iter::once(document).chain(schemas.iter()) {
            for symbol in symbols(schema)
                .into_iter()
                .filter(|symbol| matches!(symbol.detail.as_str(), "enum" | "type"))
            {
                insert(completion_from_symbol(symbol));
            }
        }
        return items.into_values().collect();
    }

    if is_placeholder_context(&document.text, offset) {
        for occurrence in occurrences(document) {
            let candidate = completion_from_semantic_key(occurrence.key);
            if !matches!(
                candidate.kind,
                CompletionKind::Type | CompletionKind::EnumMember
            ) {
                insert(candidate);
            }
        }
        if let Some(path) = locale_message_path_containing(document, offset) {
            let database = schema_database(&schemas);
            if let Some(message) = database.symbols().messages().get(&path) {
                for parameter in message.parameters() {
                    insert(CompletionCandidate {
                        label: parameter.name().to_owned(),
                        kind: CompletionKind::Variable,
                        detail: Some(format!("parameter: {}", parameter.ty())),
                    });
                }
            }
        }
        return items.into_values().collect();
    }

    if locale_dispatch_key_context(document, offset) {
        for (label, detail) in locale_dispatch_variants(document, offset) {
            insert(completion(&label, CompletionKind::EnumMember, &detail));
        }
        insert(completion("_", CompletionKind::Keyword, "wildcard branch"));
        return items.into_values().collect();
    }

    for keyword in base_keywords(document.kind) {
        insert(completion(
            &keyword,
            CompletionKind::Keyword,
            "declaration keyword",
        ));
    }
    for symbol in symbols(document) {
        insert(completion_from_symbol(symbol));
    }
    for occurrence in occurrences(document) {
        insert(completion_from_semantic_key(occurrence.key));
    }

    items.into_values().collect()
}

fn completion(label: &str, kind: CompletionKind, detail: &str) -> CompletionCandidate {
    CompletionCandidate {
        label: label.to_owned(),
        kind,
        detail: Some(detail.to_owned()),
    }
}

fn completion_from_symbol(symbol: Symbol) -> CompletionCandidate {
    let kind = match symbol.detail.as_str() {
        "enum" => CompletionKind::Enum,
        "type" => CompletionKind::Type,
        "function" | "impl" => CompletionKind::Function,
        "variable" => CompletionKind::Variable,
        "message" | "message group" => CompletionKind::Message,
        _ => CompletionKind::Property,
    };
    CompletionCandidate {
        label: symbol.name,
        kind,
        detail: Some(symbol.detail),
    }
}

fn completion_from_semantic_key(key: SemanticKey) -> CompletionCandidate {
    match key {
        SemanticKey::Message(label) => completion(&label, CompletionKind::Message, "message"),
        SemanticKey::Type(label) => completion(&label, CompletionKind::Type, "type"),
        SemanticKey::Variable(label) => completion(&label, CompletionKind::Variable, "variable"),
        SemanticKey::Function(label) => completion(&label, CompletionKind::Function, "function"),
        SemanticKey::EnumVariant { variant, .. } => {
            completion(&variant, CompletionKind::EnumMember, "enum member")
        }
        SemanticKey::FormAttribute { path, .. } => {
            completion(&path, CompletionKind::Property, "form attribute")
        }
        SemanticKey::Parameter { name, .. } => {
            completion(&name, CompletionKind::Variable, "parameter")
        }
    }
}

fn schema_type_context(document: &LinguiniDocument, offset: usize) -> bool {
    if document.kind != SourceKind::Schema {
        return false;
    }
    let before = prefix_at_boundary(&document.text, offset);
    let line = before.rsplit_once('\n').map_or(before, |(_, line)| line);
    let delimiter = line.rfind(['(', ',']).unwrap_or(0);
    line[delimiter..].rfind(':').is_some_and(|colon| {
        let after = &line[delimiter + colon + 1..];
        !after.contains([')', ','])
    }) || line
        .trim_start()
        .strip_prefix("type ")
        .is_some_and(|rest| rest.contains('='))
}

fn locale_dispatch_key_context(document: &LinguiniDocument, offset: usize) -> bool {
    if document.kind != SourceKind::Locale || is_placeholder_context(&document.text, offset) {
        return false;
    }
    let before = prefix_at_boundary(&document.text, offset);
    let line = before.rsplit_once('\n').map_or(before, |(_, line)| line);
    !line.contains("=>") && locale_function_containing(document, offset).is_some()
}

fn locale_dispatch_variants(document: &LinguiniDocument, offset: usize) -> Vec<(String, String)> {
    let Some(function) = locale_function_containing(document, offset) else {
        return Vec::new();
    };
    let enum_variants = parsed_locale(document)
        .and_then(|parsed| parsed.ast.as_ref())
        .map(|locale| {
            locale
                .declarations()
                .iter()
                .filter_map(|declaration| match declaration {
                    LocaleDeclaration::Enum(item) => Some((
                        item.name.value.as_str(),
                        item.variants
                            .iter()
                            .map(|item| item.value.as_str())
                            .collect::<Vec<_>>(),
                    )),
                    _ => None,
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut variants = BTreeMap::new();
    for parameter in &function.parameters {
        if parameter.ty.value == "Plural" {
            for category in ["zero", "one", "two", "few", "many", "other"] {
                variants.insert(category.to_owned(), "Plural category".to_owned());
            }
        } else if let Some(values) = enum_variants.get(parameter.ty.value.as_str()) {
            for value in values {
                variants.insert(
                    (*value).to_owned(),
                    format!("{} member", parameter.ty.value),
                );
            }
        }
    }
    variants.into_iter().collect()
}

fn locale_function_containing(
    document: &LinguiniDocument,
    offset: usize,
) -> Option<&FunctionDeclaration> {
    let locale = parsed_locale(document)?.ast.as_ref()?;
    locale
        .declarations()
        .iter()
        .find_map(|declaration| match declaration {
            LocaleDeclaration::Function(function)
                if function.span.start <= offset && offset <= function.span.end =>
            {
                Some(function)
            }
            LocaleDeclaration::Override(inner) => match inner.as_ref() {
                LocaleDeclaration::Function(function)
                    if function.span.start <= offset && offset <= function.span.end =>
                {
                    Some(function)
                }
                _ => None,
            },
            _ => None,
        })
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
    let database = message_name
        .as_ref()
        .map(|_| schema_database(&matching_schema_documents(document, workspace.clone())));

    if document.kind == SourceKind::Locale && symbol.docs.is_empty() {
        if let Some(name) = &message_name {
            if let Some(docs) = database
                .as_ref()
                .and_then(|database| schema_docs_for_message(database, name))
            {
                symbol.docs = docs;
            }
        }
    }
    if let Some(name) = &message_name {
        if let Some(signature) = database
            .as_ref()
            .and_then(|database| schema_signature_for_message(database, name))
        {
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

fn schema_docs_for_message(database: &SchemaDatabase, name: &str) -> Option<Vec<String>> {
    let docs = database.symbols().messages().get(name)?.docs();
    (!docs.is_empty()).then(|| docs.to_vec())
}

fn schema_signature_for_message(database: &SchemaDatabase, name: &str) -> Option<String> {
    let message = database.symbols().messages().get(name)?;
    let parameters = message
        .parameters()
        .iter()
        .map(|parameter| format!("{}: {}", parameter.name(), parameter.ty()))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("{name}({parameters})"))
}

fn schema_database(documents: &[LinguiniDocument]) -> SchemaDatabase {
    let schemas = documents
        .iter()
        .filter_map(|document| parsed_schema(document)?.ast.clone())
        .collect::<Vec<_>>();
    SchemaDatabase::build_from_files(&schemas)
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
    let database = SchemaDatabase::build_from_files(&schemas);
    database
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .source_span
                .is_some_and(|span| span.source == document.source_id)
        })
        .cloned()
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
    for declaration in locale.declarations() {
        if let Some(name) = locale_declaration_message_name_at(declaration, None, offset) {
            return Some(name);
        }
    }
    None
}

fn locale_message_path_containing(document: &LinguiniDocument, offset: usize) -> Option<String> {
    let locale = parsed_locale(document)?.ast.as_ref()?;
    for declaration in locale.declarations() {
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
