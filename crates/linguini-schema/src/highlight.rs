//! Highlight classification shared by editor and build-time hosts.
//! Tokens and semantic occurrences come from the normal recoverable language pipeline.

use std::collections::BTreeMap;

use linguini_syntax::{Span, Token, TokenKind};

use crate::{SemanticKey, SemanticOccurrence};

/// Stable token IDs used by the Linguini semantic-token protocol.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum HighlightKind {
    Keyword,
    Variable,
    Enum,
    EnumMember,
    String,
    Comment,
    Operator,
    Function,
    Type,
    Parameter,
}

impl HighlightKind {
    pub const NAMES: [&'static str; 10] = [
        "keyword",
        "variable",
        "enum",
        "enumMember",
        "string",
        "comment",
        "operator",
        "function",
        "type",
        "parameter",
    ];
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HighlightToken {
    pub span: Span,
    pub kind: HighlightKind,
}

/// Classifies the original source spans without reparsing text or duplicating lexer rules.
/// Semantic identity refines identifiers; lexical colors survive incomplete declarations.
pub fn highlight_tokens(
    tokens: &[Token],
    occurrences: &[SemanticOccurrence],
) -> Vec<HighlightToken> {
    let identities = occurrences
        .iter()
        .map(|occurrence| {
            let kind = match &occurrence.key {
                SemanticKey::Type(name) if name == "Plural" => HighlightKind::Enum,
                SemanticKey::Type(_) => HighlightKind::Type,
                SemanticKey::EnumVariant { .. } => HighlightKind::EnumMember,
                SemanticKey::Function(_) | SemanticKey::FormAttribute { .. } => {
                    HighlightKind::Function
                }
                SemanticKey::Parameter { .. } => HighlightKind::Parameter,
                SemanticKey::Message(_) | SemanticKey::Variable(_) => HighlightKind::Variable,
            };
            (
                (
                    occurrence.span.source,
                    occurrence.span.start,
                    occurrence.span.end,
                ),
                kind,
            )
        })
        .collect::<BTreeMap<_, _>>();
    tokens
        .iter()
        .filter_map(|token| {
            let kind = identities
                .get(&(token.span.source, token.span.start, token.span.end))
                .copied()
                .or_else(|| lexical_kind(&token.kind))?;
            Some(HighlightToken {
                span: token.span,
                kind,
            })
        })
        .collect()
}

fn lexical_kind(kind: &TokenKind) -> Option<HighlightKind> {
    Some(match kind {
        TokenKind::Ident(value)
            if matches!(
                value.as_str(),
                "enum" | "type" | "impl" | "form" | "fn" | "let" | "override"
            ) =>
        {
            HighlightKind::Keyword
        }
        TokenKind::Ident(value) if value == "Plural" => HighlightKind::Enum,
        TokenKind::Ident(_) => HighlightKind::Variable,
        TokenKind::LocaleTag(_)
        | TokenKind::String(_)
        | TokenKind::RawText(_)
        | TokenKind::TripleQuote
        | TokenKind::RawTripleQuote => HighlightKind::String,
        TokenKind::Comment(_) | TokenKind::DocComment(_) => HighlightKind::Comment,
        TokenKind::Equals
        | TokenKind::Arrow
        | TokenKind::At
        | TokenKind::LBrace
        | TokenKind::RBrace
        | TokenKind::LParen
        | TokenKind::RParen
        | TokenKind::Comma
        | TokenKind::Colon
        | TokenKind::Dot => HighlightKind::Operator,
        TokenKind::Whitespace | TokenKind::Newline | TokenKind::Error(_) => return None,
    })
}
