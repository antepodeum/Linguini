mod document;
mod server;

pub use document::{
    completion_items, diagnostics, document_symbols, format_document, hover_at, references_at,
    semantic_tokens, workspace_symbols, LinguiniDocument, LinguiniSemanticToken, SemanticLegend,
};
pub use server::{run_stdio, run_stdio_blocking};

pub const CRATE_PURPOSE: &str = "language server";

#[cfg(test)]
mod tests;
