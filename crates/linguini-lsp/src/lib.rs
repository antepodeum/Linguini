mod code_action;
mod document;
mod server;

pub use document::{
    completion_items, diagnostics, diagnostics_with_workspace, document_symbols, format_document,
    hover_at, prepare_rename_at, references_at, rename_workspace_edits, semantic_tokens,
    workspace_symbols, LinguiniDocument, LinguiniSemanticToken, SemanticLegend, WorkspaceTextEdit,
};
pub use server::{run_stdio, run_stdio_blocking};

pub const CRATE_PURPOSE: &str = "language server";

#[cfg(test)]
mod tests;
