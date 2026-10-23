use serde_json::{json, Value};

use crate::{semantic_tokens, LinguiniDocument, SemanticLegend};

/// Highlights a standalone snippet through the same recoverable pipeline as open documents.
/// Returns `None` for unsupported languages or inputs exceeding document safety limits.
pub fn highlight_source(language: &str, source: &str) -> Option<Value> {
    let document = LinguiniDocument::try_new("linguini:highlight", language, source)?;
    if !document.is_within_safety_limits() {
        return None;
    }
    let tokens = semantic_tokens(&document)
        .into_iter()
        .map(|token| {
            json!({
                "line": token.line,
                "start": token.start,
                "length": token.length,
                "type": SemanticLegend::TYPES[token.token_type as usize],
            })
        })
        .collect::<Vec<_>>();
    Some(json!({ "version": 1, "legend": SemanticLegend::TYPES, "tokens": tokens }))
}
