use linguini_analyzer::{
    analyze_locale_coverage_with_options, analyze_locale_file, Diagnostic, DiagnosticSeverity,
    LocaleCoverageOptions,
};
use linguini_format::{format_source, FormatOptions, SourceKind};
use linguini_syntax::{
    lex_schema_with_recovery, lex_with_recovery, parse_locale_with_recovery,
    parse_schema_with_recovery, DocComment, FunctionBranch, FunctionBranchValue,
    FunctionDeclaration, LocaleDeclaration, MessageSignature, Name, Parameter, SchemaDeclaration,
    Span, TextPart, TextPattern, Token, TokenKind,
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
    if let Some(hover) = plural_branch_hover(document, offset) {
        return Some(hover);
    }

    symbols(document)
        .into_iter()
        .find(|symbol| contains(symbol.span, offset))
        .map(|symbol| {
            let mut parts = Vec::new();
            parts.push(format!("{} `{}`", symbol.detail, symbol.name));
            if symbol.docs.is_empty() {
            } else {
                parts.push(symbol.docs.join("\n"));
            }
            if let Some(preview) = symbol.preview {
                parts.push(format!("Sample\n\n```text\n{preview}\n```"));
            }
            parts.join("\n\n")
        })
}

pub fn references_at(document: &LinguiniDocument, offset: usize) -> Vec<Span> {
    let Some((word, _)) = identifier_at(document, offset) else {
        return Vec::new();
    };
    matching_identifier_spans(document, &word)
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
        SchemaDeclaration::Message(item) => {
            vec![schema_message_symbol(item, None)]
        }
        SchemaDeclaration::Group(item) => {
            let mut output = vec![symbol(&item.name, "message group", &item.docs)];
            output.extend(
                item.messages
                    .iter()
                    .map(|message| schema_message_symbol(message, Some(&item.name.value))),
            );
            output
        }
    }
}

fn locale_declaration_symbols(declaration: &LocaleDeclaration) -> Vec<Symbol> {
    match declaration {
        LocaleDeclaration::Enum(item) => vec![symbol(&item.name, "enum", &item.docs)],
        LocaleDeclaration::Form(item) => vec![symbol(&item.name, "impl", &item.docs)],
        LocaleDeclaration::Function(item) => vec![symbol(&item.name, "function", &item.docs)],
        LocaleDeclaration::Message(item) => vec![locale_message_symbol(
            &item.name,
            None,
            &item.docs,
            &item.value,
        )],
        LocaleDeclaration::Group(item) => {
            let mut output = vec![symbol(&item.name, "message group", &item.docs)];
            output.extend(item.messages.iter().map(|message| {
                locale_message_symbol(
                    &message.name,
                    Some(&item.name.value),
                    &message.docs,
                    &message.value,
                )
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
        preview: None,
    }
}

fn schema_message_symbol(
    message: &MessageSignature,
    group: Option<&str>,
) -> Symbol {
    let mut symbol = symbol(&message.name, "message", &message.docs);
    let display_name = qualified_name(group, &message.name.value);
    symbol.preview = Some(schema_message_preview(&display_name, &message.parameters));
    symbol
}

fn locale_message_symbol(
    name: &Name,
    group: Option<&str>,
    docs: &[DocComment],
    value: &TextPattern,
) -> Symbol {
    let mut symbol = symbol(name, "message", docs);
    let display_name = qualified_name(group, &name.value);
    symbol.preview = Some(format!("{display_name} -> {}", text_preview(value)));
    symbol
}

fn schema_message_preview(name: &str, parameters: &[Parameter]) -> String {
    let arguments = parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}: {}",
                parameter.name.value,
                sample_value_for_type(&parameter.ty.value)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name}({arguments})")
}

fn sample_value_for_type(ty: &str) -> &'static str {
    match ty {
        "Number" => "3",
        "Decimal" => "12.5",
        "String" => "\"Sample\"",
        "Date" => "2026-05-15",
        _ => "sample",
    }
}

fn text_preview(value: &TextPattern) -> String {
    value
        .parts
        .iter()
        .map(|part| match part {
            TextPart::Text(text) => text.value.clone(),
            TextPart::Placeholder(placeholder) => {
                format!("{{{}}}", expression_preview(&placeholder.expression))
            }
        })
        .collect::<String>()
}

fn expression_preview(expression: &linguini_syntax::Expression) -> String {
    let mut output = expression
        .path
        .iter()
        .map(|name| name.value.as_str())
        .collect::<Vec<_>>()
        .join(".");
    if !expression.arguments.is_empty() {
        output.push('(');
        output.push_str(
            &expression
                .arguments
                .iter()
                .map(expression_preview)
                .collect::<Vec<_>>()
                .join(", "),
        );
        output.push(')');
    }
    output
}

fn plural_branch_hover(document: &LinguiniDocument, offset: usize) -> Option<String> {
    let SourceKind::Locale = document.kind else {
        return None;
    };
    let locale = locale_from_uri(&document.uri)?;
    let rules = linguini_cldr::compiled_plural_rules(&locale)?;
    let file = parse_locale_with_recovery(&document.text).ast?;

    for declaration in &file.declarations {
        if let Some(hover) = declaration_plural_branch_hover(declaration, offset, &locale, &rules) {
            return Some(hover);
        }
    }
    None
}

fn declaration_plural_branch_hover(
    declaration: &LocaleDeclaration,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    match declaration {
        LocaleDeclaration::Function(function) => function_plural_branch_hover(
            function,
            &dispatch_types(function),
            &function.branches,
            0,
            offset,
            locale,
            rules,
        ),
        LocaleDeclaration::Override(inner) => {
            declaration_plural_branch_hover(inner, offset, locale, rules)
        }
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Message(_)
        | LocaleDeclaration::Group(_) => None,
    }
}

fn function_plural_branch_hover(
    function: &FunctionDeclaration,
    dispatch_types: &[&str],
    branches: &[FunctionBranch],
    depth: usize,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    let dispatch_type = dispatch_types.get(depth).copied();
    for branch in branches {
        if dispatch_type == Some("Plural") && contains(branch.key.span, offset) {
            return Some(plural_samples_hover(
                &function.name.value,
                &branch.key.value,
                locale,
                rules,
            ));
        }
        if let FunctionBranchValue::Dispatch(children) = &branch.value {
            if let Some(hover) = function_plural_branch_hover(
                function,
                dispatch_types,
                children,
                depth + 1,
                offset,
                locale,
                rules,
            ) {
                return Some(hover);
            }
        }
    }
    None
}

fn dispatch_types(function: &FunctionDeclaration) -> Vec<&str> {
    function
        .parameters
        .iter()
        .filter_map(|parameter| {
            (parameter.ty.value != "String").then_some(parameter.ty.value.as_str())
        })
        .collect()
}

fn plural_samples_hover(
    function_name: &str,
    branch: &str,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> String {
    let category = if branch == "_" { "other" } else { branch };
    let samples = plural_samples_for_category(rules, category);
    let sample_text = if samples.is_empty() {
        "no integer samples in 0..200".to_owned()
    } else {
        samples.join(", ")
    };
    format!(
        "plural branch `{branch}` in `{function_name}`\n\nLocale `{locale}` category `{category}`\n\nSample numbers: {sample_text}"
    )
}

fn plural_samples_for_category(
    rules: &linguini_cldr::CompiledPluralRules,
    category: &str,
) -> Vec<String> {
    (0..=200)
        .map(|number| number.to_string())
        .filter(|sample| {
            rules
                .category_for(sample)
                .is_ok_and(|candidate| candidate == category)
        })
        .take(12)
        .collect()
}

fn locale_from_uri(uri: &str) -> Option<String> {
    let file_name = uri
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())?;
    file_name
        .strip_suffix(".lgl")
        .or_else(|| file_name.strip_suffix(".linguini"))
        .map(|locale| locale.to_owned())
}

fn qualified_name(group: Option<&str>, name: &str) -> String {
    match group {
        Some(group) => format!("{group}.{name}"),
        None => name.to_owned(),
    }
}

fn tokens(document: &LinguiniDocument) -> Vec<Token> {
    match document.kind {
        SourceKind::Schema => lex_schema_with_recovery(&document.text).tokens,
        SourceKind::Locale => lex_with_recovery(&document.text).tokens,
    }
}

fn identifier_at(document: &LinguiniDocument, offset: usize) -> Option<(String, Span)> {
    tokens(document)
        .into_iter()
        .find_map(|token| match token.kind {
            TokenKind::Ident(value) | TokenKind::LocaleTag(value)
                if contains(token.span, offset) =>
            {
                Some((value, token.span))
            }
            _ => None,
        })
}

fn matching_identifier_spans(document: &LinguiniDocument, word: &str) -> Vec<Span> {
    tokens(document)
        .into_iter()
        .filter_map(|token| match token.kind {
            TokenKind::Ident(value) | TokenKind::LocaleTag(value) if value == word => {
                Some(token.span)
            }
            _ => None,
        })
        .collect()
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
    matches!(
        next_significant_kind(tokens, index),
        Some(TokenKind::LParen)
    ) || matches!(
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
        TokenKind::Whitespace
            | TokenKind::Newline
            | TokenKind::Comment(_)
            | TokenKind::DocComment(_)
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
