mod code_action;
mod document;
mod server;

pub use document::{
    completion_items, definition_at_with_workspace, diagnostics, diagnostics_with_workspace,
    document_symbols, format_document, hover_at, hover_at_with_workspace, prepare_rename_at,
    references_at, rename_workspace_edits, semantic_tokens, workspace_symbols, LinguiniDocument,
    LinguiniSemanticToken, SemanticLegend, WorkspaceTextEdit,
};
pub use server::{run_stdio, run_stdio_blocking};

pub const CRATE_PURPOSE: &str = "language server";

#[cfg(test)]
mod tests;
