use super::{contains, LinguiniDocument};
use linguini_format::SourceKind;
use linguini_syntax::{lex_schema_with_recovery, lex_with_recovery, Span, Token, TokenKind};

pub(super) fn tokens(document: &LinguiniDocument) -> Vec<Token> {
    match document.kind {
        SourceKind::Schema => lex_schema_with_recovery(&document.text).tokens,
        SourceKind::Locale => lex_with_recovery(&document.text).tokens,
    }
}

pub(super) fn identifier_at(document: &LinguiniDocument, offset: usize) -> Option<(String, Span)> {
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

pub(super) fn matching_identifier_spans(document: &LinguiniDocument, word: &str) -> Vec<Span> {
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

pub(super) fn semantic_token_type(tokens: &[Token], index: usize) -> Option<u32> {
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

pub(super) fn base_keywords(kind: SourceKind) -> Vec<String> {
    match kind {
        SourceKind::Schema => ["enum", "type"].into_iter().map(str::to_owned).collect(),
        SourceKind::Locale => [
            "enum", "impl", "form", "fn", "let", "override", "Plural", "_",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    }
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "enum" | "type" | "impl" | "form" | "fn" | "let" | "override"
    )
}

pub(super) fn is_placeholder_context(source: &str, offset: usize) -> bool {
    let before = &source[..offset.min(source.len())];
    before.rfind('{') > before.rfind('}')
}
