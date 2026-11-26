mod code_action;
mod document;
mod highlight;
mod host;
mod server;

pub use document::{
    completion_candidates_with_workspace, completion_items, completion_items_with_workspace,
    definition_at_with_workspace, diagnostics, diagnostics_with_workspace, document_symbols,
    format_document, format_document_with_options, hover_at, hover_at_with_workspace,
    prepare_rename_at, references_at, references_at_with_workspace, rename_workspace_edits,
    semantic_tokens, workspace_symbols, CompletionCandidate, CompletionKind, LinguiniDocument,
    LinguiniSemanticToken, SemanticLegend, WorkspaceReference, WorkspaceTextEdit,
};
pub use highlight::highlight_source;
#[cfg(not(target_arch = "wasm32"))]
pub use host::{run_stdio, run_stdio_blocking, try_run_stdio_blocking, NativeScheduler};
pub use host::{
    LspScheduler, LspTask, MemoryWorkspace, WorkspaceFile, WorkspaceLimits, WorkspaceSourcePlan,
    WorkspaceSourceRoots, WORKSPACE_IGNORED_DIRECTORIES, WORKSPACE_LIMITS,
};

#[cfg(test)]
mod tests;
