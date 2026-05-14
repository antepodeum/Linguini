use linguini_analyzer::Diagnostic;
use linguini_format::{format_source, FormatOptions, SourceKind};
use linguini_syntax::{
    lex_schema_with_recovery, lex_with_recovery, parse_locale_with_recovery,
    parse_schema_with_recovery, DocComment, LocaleDeclaration, Name, SchemaDeclaration, Span,
    Token, TokenKind,
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextEdit {
    pub span: Span,
    pub new_text: String,
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
    pub fn new(uri: impl Into<String>, language_id: impl Into<String>, text: impl Into<String>) -> Self {
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
        let line_start = self.line_starts.get(line).copied().unwrap_or(self.text.len());
        let line_end = line_end(&self.text, &self.line_starts, line);
        utf16_column_to_offset(&self.text, line_start, line_end, character as usize)
    }

    pub fn range(&self, span: Span) -> ((u32, u32), (u32, u32)) {
        (self.position(span.start), self.position(span.end))
    }
}

pub fn diagnostics(document: &LinguiniDocument) -> Vec<Diagnostic> {
    let errors = match document.kind {
        SourceKind::Schema => parse_schema_with_recovery(&document.text).errors,
        SourceKind::Locale => parse_locale_with_recovery(&document.text).errors,
    };

    errors
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
        .collect()
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
    symbols(document)
        .into_iter()
        .find(|symbol| contains(symbol.span, offset))
        .map(|symbol| {
            if symbol.docs.is_empty() {
                format!("{} `{}`", symbol.detail, symbol.name)
            } else {
                format!("{} `{}`\n\n{}", symbol.detail, symbol.name, symbol.docs.join("\n"))
            }
        })
}

pub fn references_at(document: &LinguiniDocument, offset: usize) -> Vec<Span> {
    let Some(word) = word_at(&document.text, offset) else {
        return Vec::new();
    };
    tokens(document)
        .into_iter()
        .filter_map(|token| match token.kind {
            TokenKind::Ident(value) if value == word => Some(token.span),
            _ => None,
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

pub fn format_document(document: &LinguiniDocument) -> Result<TextEdit, linguini_format::FormatError> {
    let formatted = format_source(document.kind, &document.text, &FormatOptions::default())?;
    Ok(TextEdit {
        span: Span::new(0, document.text.len()),
        new_text: formatted,
    })
}

fn symbols(document: &LinguiniDocument) -> Vec<Symbol> {
    match document.kind {
        SourceKind::Schema => parse_schema_with_recovery(&document.text)
            .ast
            .map(|file| {
                file.declarations
                    .iter()
                    .flat_map(schema_declaration_symbols)
                    .collect()
            })
            .unwrap_or_default(),
        SourceKind::Locale => parse_locale_with_recovery(&document.text)
            .ast
            .map(|file| {
                file.declarations
                    .iter()
                    .flat_map(locale_declaration_symbols)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn schema_declaration_symbols(declaration: &SchemaDeclaration) -> Vec<Symbol> {
    match declaration {
        SchemaDeclaration::Enum(item) => vec![symbol(&item.name, "enum", &item.docs)],
        SchemaDeclaration::TypeAlias(item) => vec![symbol(&item.name, "type", &item.docs)],
        SchemaDeclaration::Message(item) => vec![symbol(&item.name, "message", &item.docs)],
        SchemaDeclaration::Group(item) => {
            let mut output = vec![symbol(&item.name, "message group", &item.docs)];
            output.extend(item.messages.iter().map(|message| {
                symbol(&message.name, "message", &message.docs)
            }));
            output
        }
    }
}

fn locale_declaration_symbols(declaration: &LocaleDeclaration) -> Vec<Symbol> {
    match declaration {
        LocaleDeclaration::Enum(item) => vec![symbol(&item.name, "enum", &item.docs)],
        LocaleDeclaration::Form(item) => vec![symbol(&item.name, "impl", &item.docs)],
        LocaleDeclaration::Function(item) => vec![symbol(&item.name, "function", &item.docs)],
        LocaleDeclaration::Message(item) => vec![symbol(&item.name, "message", &item.docs)],
        LocaleDeclaration::Group(item) => {
            let mut output = vec![symbol(&item.name, "message group", &item.docs)];
            output.extend(item.messages.iter().map(|message| {
                symbol(&message.name, "message", &message.docs)
            }));
            output
        }
        LocaleDeclaration::Override(inner) => locale_declaration_symbols(inner),
    }
}

fn symbol(name: &Name, detail: &str, docs: &[DocComment]) -> Symbol {
    Symbol {
        name: name.value.clone(),
        detail: detail.to_owned(),
        span: name.span,
        docs: docs.iter().map(|doc| doc.text.trim().to_owned()).collect(),
    }
}

fn tokens(document: &LinguiniDocument) -> Vec<Token> {
    match document.kind {
        SourceKind::Schema => lex_schema_with_recovery(&document.text).tokens,
        SourceKind::Locale => lex_with_recovery(&document.text).tokens,
    }
}

fn semantic_token_type(tokens: &[Token], index: usize) -> Option<u32> {
    let kind = &tokens.get(index)?.kind;
    match kind {
        TokenKind::Ident(value) if is_keyword(value) => Some(0),
        TokenKind::Ident(_) if is_function_like_identifier(tokens, index) => Some(7),
        TokenKind::Ident(value) if value.chars().next().is_some_and(char::is_uppercase) => Some(2),
        TokenKind::Ident(_) | TokenKind::LocaleTag(_) => Some(1),
        TokenKind::String(_) | TokenKind::RawText(_) => Some(4),
        TokenKind::Comment(_) | TokenKind::DocComment(_) => Some(5),
        TokenKind::Equals | TokenKind::Arrow | TokenKind::At => Some(6),
        _ => None,
    }
}

fn is_function_like_identifier(tokens: &[Token], index: usize) -> bool {
    matches!(next_significant_kind(tokens, index), Some(TokenKind::LParen))
        || matches!(
            previous_significant_kind(tokens, index),
            Some(TokenKind::Ident(value)) if matches!(value.as_str(), "fn" | "form")
        )
}

fn previous_significant_kind(tokens: &[Token], index: usize) -> Option<&TokenKind> {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !is_trivia(&token.kind))
        .map(|token| &token.kind)
}

fn next_significant_kind(tokens: &[Token], index: usize) -> Option<&TokenKind> {
    tokens
        .get(index + 1..)
        .unwrap_or(&[])
        .iter()
        .find(|token| !is_trivia(&token.kind))
        .map(|token| &token.kind)
}

fn is_trivia(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Whitespace | TokenKind::Newline | TokenKind::Comment(_) | TokenKind::DocComment(_)
    )
}

fn base_keywords(kind: SourceKind) -> Vec<String> {
    match kind {
        SourceKind::Schema => ["enum", "type"].into_iter().map(str::to_owned).collect(),
        SourceKind::Locale => ["enum", "impl", "form", "fn", "override", "Plural", "_"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }
}

fn is_keyword(value: &str) -> bool {
    matches!(value, "enum" | "type" | "impl" | "form" | "fn" | "override")
}

fn is_placeholder_context(source: &str, offset: usize) -> bool {
    let before = &source[..offset.min(source.len())];
    before.rfind('{') > before.rfind('}')
}

fn contains(span: Span, offset: usize) -> bool {
    span.start <= offset && offset <= span.end
}

fn word_at(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();
    let mut start = offset.min(bytes.len());
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset.min(bytes.len());
    while end < bytes.len() && is_word_byte(bytes[end]) {
        end += 1;
    }
    (start < end).then(|| source[start..end].to_owned())
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn utf16_len(text: &str) -> usize {
    text.chars().map(|ch| ch.len_utf16()).sum()
}

fn utf16_column_to_offset(source: &str, line_start: usize, line_end: usize, column: usize) -> usize {
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
