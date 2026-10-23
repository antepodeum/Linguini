use super::LinguiniDocument;
use linguini_format::SourceKind;
use linguini_syntax::{lex_schema_with_recovery_in, lex_with_recovery_in, Token};

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

pub(super) fn is_placeholder_context(source: &str, offset: usize) -> bool {
    let before = prefix_at_boundary(source, offset);
    let Some(open) = before
        .rfind('{')
        .filter(|open| Some(*open) > before.rfind('}'))
    else {
        return false;
    };
    !before[open + 1..].contains(['\r', '\n'])
}

pub(super) fn prefix_at_boundary(source: &str, offset: usize) -> &str {
    let mut boundary = offset.min(source.len());
    while !source.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &source[..boundary]
}
