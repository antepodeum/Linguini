use super::LinguiniDocument;
use linguini_format::SourceKind;
use linguini_syntax::{lex_schema_with_recovery_in, lex_with_recovery_in, Token, TokenKind};

pub(super) fn tokens(document: &LinguiniDocument) -> Vec<Token> {
    document
        .token_cache
        .get_or_init(|| match document.kind {
            SourceKind::Schema => {
                lex_schema_with_recovery_in(&document.text, document.source_id).tokens
            }
            SourceKind::Locale => lex_with_recovery_in(&document.text, document.source_id).tokens,
        })
        .clone()
}

pub(super) fn semantic_token_type(tokens: &[Token], index: usize) -> Option<u32> {
    let kind = &tokens.get(index)?.kind;
    match kind {
        TokenKind::Ident(value) if is_keyword(value) => Some(0),
        TokenKind::Ident(value) if value == "Plural" => Some(2),
        TokenKind::Ident(_) | TokenKind::LocaleTag(_) => None,
        TokenKind::String(_) | TokenKind::RawText(_) => Some(4),
        TokenKind::Comment(_) | TokenKind::DocComment(_) => Some(5),
        TokenKind::Equals | TokenKind::Arrow | TokenKind::At => Some(6),
        _ => None,
    }
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
