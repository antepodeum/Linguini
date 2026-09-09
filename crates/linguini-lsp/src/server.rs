#![allow(deprecated)]

use crate::document::prime_semantic_cache;
use crate::{
    code_action::{analyzer_quick_fix_actions, to_lsp_diagnostic_with_workspace},
    completion_candidates_with_workspace, definition_at_with_workspace, diagnostics_with_workspace,
    document_symbols, format_document_with_options, hover_at_with_workspace, prepare_rename_at,
    references_at_with_workspace, rename_workspace_edits, semantic_tokens, LinguiniDocument,
    SemanticLegend,
};
use linguini_config::{
    discover_locale_files, discover_schema_files, parse_config, DEFAULT_CONFIG_FILE,
};
use linguini_format::FormatOptions;
use linguini_syntax::SourceId;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;
use tower_lsp_server::{
    jsonrpc::{Error, Result},
    lsp_types::*,
    Client, LanguageServer, LspService, Server,
};

const MAX_WORKSPACE_DEPTH: usize = 64;
const MAX_WORKSPACE_ENTRIES: usize = 50_000;
const MAX_WORKSPACE_ROOTS: usize = 32;
const MAX_WORKSPACE_PROJECTS: usize = 256;
const MAX_INDEXED_FILES: usize = 50_000;
const MAX_INDEXED_BYTES: u64 = 128 * 1024 * 1024;
const MAX_PROJECT_FILES: usize = 20_000;
const MAX_PROJECT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_DOCUMENT_BYTES: u64 = 4 * 1024 * 1024;

type Uri = Url;

#[derive(Debug, Clone, Eq, PartialEq)]
struct ProjectContext {
    root: PathBuf,
    schema_root: PathBuf,
    locale_root: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct WorkspaceIndex {
    projects: Vec<ProjectSnapshot>,
    errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProjectSnapshot {
    context: ProjectContext,
    documents: Vec<LinguiniDocument>,
}

#[derive(Debug, Clone)]
struct CachedDiagnostics {
    version: Option<i32>,
    text: String,
    diagnostics: Vec<linguini_analyzer::Diagnostic>,
}

#[derive(Debug, Clone)]
struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Uri, LinguiniDocument>>>,
    workspace_roots: Arc<RwLock<Vec<PathBuf>>>,
    workspace_index: Arc<RwLock<WorkspaceIndex>>,
    diagnostic_cache: Arc<RwLock<HashMap<Uri, CachedDiagnostics>>>,
    workspace_refresh_generation: Arc<AtomicU64>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            workspace_roots: Arc::new(RwLock::new(Vec::new())),
            workspace_index: Arc::new(RwLock::new(WorkspaceIndex::default())),
            diagnostic_cache: Arc::new(RwLock::new(HashMap::new())),
            workspace_refresh_generation: Arc::new(AtomicU64::new(0)),
        }
    }

    async fn publish(&self, document: &LinguiniDocument) {
        let workspace = self.workspace_documents_for(document);
        let analyzer_diagnostics = diagnostics_with_workspace(document, workspace.clone());
        let diagnostics = analyzer_diagnostics
            .iter()
            .map(|diagnostic| to_lsp_diagnostic_with_workspace(document, diagnostic, &workspace))
            .collect::<Vec<_>>();
        let Ok(uri) = document.uri.parse::<Uri>() else {
            return;
        };
        if !self.document_is_current(&uri, document) {
            return;
        }
        write_lock(&self.diagnostic_cache).insert(
            uri.clone(),
            CachedDiagnostics {
                version: document.version,
                text: document.text.clone(),
                diagnostics: analyzer_diagnostics,
            },
        );
        self.client
            .publish_diagnostics(uri, diagnostics, document.version)
            .await;
    }

    async fn publish_related(&self, document: &LinguiniDocument) {
        let related = self.related_open_documents(document);
        if related.is_empty() {
            self.publish(document).await;
            return;
        }

        for related_document in related {
            self.publish(&related_document).await;
        }
    }

    fn schedule_publish_related(&self, document: LinguiniDocument) {
        let backend = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(75)).await;
            let Ok(uri) = document.uri.parse::<Uri>() else {
                return;
            };
            if backend.document_is_current(&uri, &document) {
                backend.publish_related(&document).await;
            }
        });
    }

    fn cached_diagnostics_for(
        &self,
        uri: &Uri,
        document: &LinguiniDocument,
    ) -> Vec<linguini_analyzer::Diagnostic> {
        if let Some(diagnostics) = read_lock(&self.diagnostic_cache)
            .get(uri)
            .filter(|cached| cached.version == document.version && cached.text == document.text)
            .map(|cached| cached.diagnostics.clone())
        {
            return diagnostics;
        }

        let diagnostics =
            diagnostics_with_workspace(document, self.workspace_documents_for(document));
        write_lock(&self.diagnostic_cache).insert(
            uri.clone(),
            CachedDiagnostics {
                version: document.version,
                text: document.text.clone(),
                diagnostics: diagnostics.clone(),
            },
        );
        diagnostics
    }

    fn workspace_documents_for(&self, document: &LinguiniDocument) -> Vec<LinguiniDocument> {
        let open_documents = self.open_documents();
        let Some(path) = document_file_path(document) else {
            return open_documents;
        };
        let Some(context) = self.project_context_for_path(&path) else {
            return open_documents
                .into_iter()
                .filter(|candidate| {
                    document_file_path(candidate)
                        .and_then(|candidate_path| self.project_context_for_path(&candidate_path))
                        .is_none()
                })
                .collect();
        };

        let mut documents = read_lock(&self.workspace_index)
            .projects
            .iter()
            .find(|project| project.context == context)
            .map(|project| project.documents.clone())
            .unwrap_or_default();
        let open_by_path = open_documents
            .iter()
            .filter_map(|open| document_file_path(open).map(|path| (path, open.clone())))
            .collect::<HashMap<_, _>>();
        for indexed in &mut documents {
            let Some(path) = document_file_path(indexed) else {
                continue;
            };
            if let Some(open) = open_by_path.get(&path) {
                *indexed = open.clone();
            }
        }
        let mut seen = documents
            .iter()
            .filter_map(document_file_path)
            .collect::<HashSet<_>>();
        for open in open_documents {
            let Some(path) = document_file_path(&open) else {
                continue;
            };
            if path_matches_context(&path, &context) && seen.insert(path) {
                documents.push(open);
            }
        }
        documents
    }

    fn all_workspace_documents(&self) -> Vec<LinguiniDocument> {
        let mut documents = read_lock(&self.workspace_index)
            .projects
            .iter()
            .flat_map(|project| project.documents.clone())
            .collect::<Vec<_>>();
        let open = self.open_documents();
        let open_by_uri = open
            .iter()
            .map(|document| (document.uri.as_str(), document))
            .collect::<HashMap<_, _>>();
        for document in &mut documents {
            if let Some(open_document) = open_by_uri.get(document.uri.as_str()) {
                *document = (*open_document).clone();
            }
        }
        let mut seen = documents
            .iter()
            .map(|document| document.uri.clone())
            .collect::<HashSet<_>>();
        documents.extend(
            open.into_iter()
                .filter(|document| seen.insert(document.uri.clone())),
        );
        documents
    }

    fn related_open_documents(&self, document: &LinguiniDocument) -> Vec<LinguiniDocument> {
        let open_documents = self.open_documents();
        let Some(path) = document_file_path(document) else {
            return vec![document.clone()];
        };
        let context = self.project_context_for_path(&path);

        open_documents
            .into_iter()
            .filter(|candidate| {
                let Some(candidate_path) = document_file_path(candidate) else {
                    return false;
                };
                self.project_context_for_path(&candidate_path) == context
            })
            .collect()
    }

    fn open_documents(&self) -> Vec<LinguiniDocument> {
        read_lock(&self.documents).values().cloned().collect()
    }

    fn document(&self, uri: &Uri) -> Option<LinguiniDocument> {
        read_lock(&self.documents).get(uri).cloned()
    }

    fn document_is_current(&self, uri: &Uri, document: &LinguiniDocument) -> bool {
        match self.document(uri) {
            Some(current) => {
                current.version == document.version
                    && current.text == document.text
                    && current.source_id == document.source_id
                    && current.namespace == document.namespace
                    && current.locale == document.locale
            }
            None => false,
        }
    }

    fn set_workspace_roots(&self, roots: Vec<PathBuf>) {
        *write_lock(&self.workspace_roots) = roots;
    }

    fn workspace_roots(&self) -> Vec<PathBuf> {
        let roots = read_lock(&self.workspace_roots);
        if roots.is_empty() {
            std::env::current_dir().into_iter().collect()
        } else {
            roots.clone()
        }
    }

    fn project_context_for_path(&self, path: &Path) -> Option<ProjectContext> {
        read_lock(&self.workspace_index)
            .projects
            .iter()
            .map(|project| &project.context)
            .filter(|context| path_matches_context(path, context))
            .max_by_key(|context| context.root.components().count())
            .cloned()
    }

    fn enrich_document(&self, document: &mut LinguiniDocument) {
        let Some(path) = document_file_path(document) else {
            return;
        };
        let indexed = read_lock(&self.workspace_index)
            .projects
            .iter()
            .flat_map(|project| project.documents.iter())
            .find(|candidate| document_file_path(candidate).as_ref() == Some(&path))
            .cloned();
        if let Some(indexed) = indexed {
            document.set_source_id(indexed.source_id);
            document.namespace = indexed.namespace;
            document.locale = indexed.locale;
        }
    }

    async fn refresh_workspace_index(&self) -> bool {
        let generation = self
            .workspace_refresh_generation
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1);
        let roots = self.workspace_roots();
        let index = match tokio::task::spawn_blocking(move || build_workspace_index(roots)).await {
            Ok(index) => index,
            Err(error) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("failed to refresh Linguini workspace index: {error}"),
                    )
                    .await;
                return false;
            }
        };
        if self.workspace_refresh_generation.load(Ordering::SeqCst) != generation {
            return false;
        }
        let errors = index.errors.clone();
        *write_lock(&self.workspace_index) = index;
        write_lock(&self.diagnostic_cache).clear();
        {
            let mut documents = write_lock(&self.documents);
            for document in documents.values_mut() {
                let Some(path) = document_file_path(document) else {
                    continue;
                };
                if let Some(indexed) = read_lock(&self.workspace_index)
                    .projects
                    .iter()
                    .flat_map(|project| project.documents.iter())
                    .find(|candidate| document_file_path(candidate).as_ref() == Some(&path))
                {
                    document.set_source_id(indexed.source_id);
                    document.namespace = indexed.namespace.clone();
                    document.locale = indexed.locale.clone();
                }
            }
        }
        for error in errors.into_iter().take(20) {
            self.client.log_message(MessageType::WARNING, error).await;
        }
        true
    }

    async fn refresh_and_publish_open(&self) {
        if !self.refresh_workspace_index().await {
            return;
        }
        for document in self.open_documents() {
            self.publish(&document).await;
        }
    }

    async fn protocol_version(&self) -> Result<serde_json::Value> {
        Ok(protocol_version_response())
    }
}

fn protocol_version_response() -> serde_json::Value {
    serde_json::json!({
        "protocolVersion": "1",
        "compilerVersion": env!("CARGO_PKG_VERSION"),
    })
}

#[tower_lsp_server::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.set_workspace_roots(workspace_roots_from_initialize(&params));
        let _ = self.refresh_workspace_index().await;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "linguini-lsp".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_owned(),
                        "{".to_owned(),
                        "(".to_owned(),
                        ",".to_owned(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: SemanticLegend::TYPES
                                    .into_iter()
                                    .map(SemanticTokenType::new)
                                    .collect(),
                                token_modifiers: Vec::new(),
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            work_done_progress_options: Default::default(),
                        },
                    ),
                ),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: Some(workspace_file_operation_capabilities()),
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let watch_options = DidChangeWatchedFilesRegistrationOptions {
            watchers: vec![FileSystemWatcher {
                glob_pattern: GlobPattern::String("**/{linguini.toml,*.lgs,*.lgl}".to_owned()),
                kind: None,
            }],
        };
        let registration = Registration {
            id: "linguini.workspace.sources".to_owned(),
            method: "workspace/didChangeWatchedFiles".to_owned(),
            register_options: serde_json::to_value(watch_options).ok(),
        };
        if let Err(error) = self.client.register_capability(vec![registration]).await {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("client rejected Linguini file watcher registration: {error}"),
                )
                .await;
        }
        self.client
            .log_message(MessageType::INFO, "Linguini language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if params.text_document.text.len() > MAX_DOCUMENT_BYTES as usize {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("ignored Linguini document larger than {MAX_DOCUMENT_BYTES} bytes"),
                )
                .await;
            return;
        }
        let Some(mut document) = LinguiniDocument::try_new(
            params.text_document.uri.to_string(),
            params.text_document.language_id,
            params.text_document.text,
        ) else {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "ignored document with unsupported Linguini language ID",
                )
                .await;
            return;
        };
        document.version = Some(params.text_document.version);
        self.enrich_document(&mut document);
        let Ok(uri) = document.uri.parse::<Uri>() else {
            return;
        };
        write_lock(&self.documents).insert(uri.clone(), document.clone());
        write_lock(&self.diagnostic_cache).remove(&uri);
        self.schedule_publish_related(document);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };
        if change.text.len() > MAX_DOCUMENT_BYTES as usize {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!(
                        "ignored Linguini document change larger than {MAX_DOCUMENT_BYTES} bytes"
                    ),
                )
                .await;
            return;
        }
        let Some(previous) = self.document(&params.text_document.uri) else {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "ignored change for unopened Linguini document",
                )
                .await;
            return;
        };
        if previous
            .version
            .is_some_and(|version| params.text_document.version <= version)
        {
            return;
        }
        let Some(mut document) = LinguiniDocument::try_new(
            params.text_document.uri.to_string(),
            previous.language_id,
            change.text,
        ) else {
            return;
        };
        document.set_source_id(previous.source_id);
        document.namespace = previous.namespace;
        document.locale = previous.locale;
        document.version = Some(params.text_document.version);
        write_lock(&self.documents).insert(params.text_document.uri.clone(), document.clone());
        write_lock(&self.diagnostic_cache).remove(&params.text_document.uri);
        self.schedule_publish_related(document);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let closed = write_lock(&self.documents).remove(&params.text_document.uri);
        write_lock(&self.diagnostic_cache).remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
        if let Some(closed) = closed {
            for document in self.related_open_documents(&closed) {
                self.publish(&document).await;
            }
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let Some(document) = self.document(&params.text_document_position.text_document.uri) else {
            return Ok(None);
        };
        let offset = document.offset(
            params.text_document_position.position.line,
            params.text_document_position.position.character,
        );
        let items = completion_candidates_with_workspace(
            &document,
            offset,
            self.workspace_documents_for(&document),
        )
        .into_iter()
        .map(|candidate| CompletionItem {
            label: candidate.label,
            kind: Some(match candidate.kind {
                crate::CompletionKind::Keyword => CompletionItemKind::KEYWORD,
                crate::CompletionKind::Type => CompletionItemKind::TYPE_PARAMETER,
                crate::CompletionKind::Enum => CompletionItemKind::ENUM,
                crate::CompletionKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
                crate::CompletionKind::Function => CompletionItemKind::FUNCTION,
                crate::CompletionKind::Variable => CompletionItemKind::VARIABLE,
                crate::CompletionKind::Message => CompletionItemKind::VALUE,
                crate::CompletionKind::Property => CompletionItemKind::PROPERTY,
            }),
            detail: candidate.detail,
            ..Default::default()
        })
        .collect();
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;
        let Some(document) = self.document(&uri) else {
            return Ok(None);
        };
        let offset = document.offset(position.line, position.character);
        Ok(
            hover_at_with_workspace(&document, offset, self.workspace_documents_for(&document))
                .map(|value| Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value,
                    }),
                    range: None,
                }),
        )
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let Some(document) = self.document(&uri) else {
            return Ok(None);
        };
        let offset = document.offset(
            params.text_document_position_params.position.line,
            params.text_document_position_params.position.character,
        );
        let workspace = self.workspace_documents_for(&document);
        let Some((target_uri, span)) =
            definition_at_with_workspace(&document, offset, workspace.clone())
        else {
            return Ok(None);
        };
        let uri = match target_uri.parse::<Uri>() {
            Ok(parsed) => parsed,
            Err(_) => uri,
        };
        let target_document = workspace
            .into_iter()
            .find(|candidate| candidate.uri == uri.as_str())
            .unwrap_or_else(|| document.clone());
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: to_range(&target_document, span),
        })))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let Some(document) = self.document(&uri) else {
            return Ok(None);
        };
        let offset = document.offset(
            params.text_document_position.position.line,
            params.text_document_position.position.character,
        );
        let workspace = self.workspace_documents_for(&document);
        let documents_by_uri = workspace
            .iter()
            .map(|candidate| (candidate.uri.as_str(), candidate))
            .collect::<HashMap<_, _>>();
        let locations = references_at_with_workspace(&document, offset, workspace.clone())
            .into_iter()
            .filter(|reference| params.context.include_declaration || !reference.declaration)
            .filter_map(|reference| {
                let target = documents_by_uri.get(reference.uri.as_str())?;
                let uri = reference.uri.parse::<Uri>().ok()?;
                Some(Location {
                    uri,
                    range: to_range(target, reference.span),
                })
            })
            .collect();
        Ok(Some(locations))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        let symbols = document_symbols(&document)
            .into_iter()
            .map(|symbol| SymbolInformation {
                name: symbol.name,
                kind: symbol_kind(&symbol.detail),
                tags: None,
                deprecated: None,
                location: Location {
                    uri: params.text_document.uri.clone(),
                    range: to_range(&document, symbol.span),
                },
                container_name: None,
            })
            .collect();
        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: encode_semantic_tokens(semantic_tokens(&document)),
        })))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let mut symbols = Vec::new();
        for document in self.all_workspace_documents() {
            let Ok(uri) = document.uri.parse::<Uri>() else {
                continue;
            };
            symbols.extend(
                document_symbols(&document)
                    .into_iter()
                    .filter_map(|symbol| {
                        symbol
                            .name
                            .to_lowercase()
                            .contains(&query)
                            .then(|| SymbolInformation {
                                name: symbol.name,
                                kind: symbol_kind(&symbol.detail),
                                tags: None,
                                deprecated: None,
                                location: Location {
                                    uri: uri.clone(),
                                    range: to_range(&document, symbol.span),
                                },
                                container_name: Some(symbol.detail),
                            })
                    }),
            );
        }
        Ok(Some(symbols))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(Some(Vec::new()));
        };
        let actions = analyzer_quick_fix_actions(
            &params.text_document.uri,
            &document,
            params.range,
            self.cached_diagnostics_for(&params.text_document.uri, &document),
        );
        Ok(Some(actions))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        if !document.is_within_safety_limits() {
            return Err(Error::invalid_params(
                "document exceeds Linguini language-server safety limits",
            ));
        }
        if !params.options.insert_spaces || params.options.tab_size == 0 {
            return Err(Error::invalid_params(
                "Linguini formatter requires a positive space indentation width",
            ));
        }
        let options = FormatOptions {
            indent_width: usize::try_from(params.options.tab_size.min(16)).unwrap_or(2),
            ..FormatOptions::default()
        };
        let mut edit = match format_document_with_options(&document, &options) {
            Ok(edit) => edit,
            Err(error) => {
                return Err(Error::invalid_params(format!(
                    "cannot format Linguini document: {error}"
                )))
            }
        };
        apply_client_formatting_options(&mut edit.new_text, &params.options);
        let Some(edit) = minimal_formatting_edit(&document, &edit.new_text) else {
            return Ok(Some(Vec::new()));
        };
        Ok(Some(vec![TextEdit {
            range: to_range(&document, edit.span),
            new_text: edit.new_text,
        }]))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        let offset = document.offset(params.position.line, params.position.character);
        let Some(span) = prepare_rename_at(&document, offset) else {
            return Ok(None);
        };
        Ok(Some(PrepareRenameResponse::Range(to_range(
            &document, span,
        ))))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let Some(document) = self.document(&uri) else {
            return Ok(None);
        };
        let offset = document.offset(
            params.text_document_position.position.line,
            params.text_document_position.position.character,
        );
        let workspace_documents = self.workspace_documents_for(&document);
        let documents_by_uri = workspace_documents
            .iter()
            .filter_map(|document| {
                document
                    .uri
                    .parse::<Uri>()
                    .ok()
                    .map(|uri| (uri, (*document).clone()))
            })
            .collect::<HashMap<_, _>>();
        let workspace_edits =
            rename_workspace_edits(workspace_documents, &document, offset, &params.new_name);
        if workspace_edits.is_empty() {
            return Err(Error::invalid_params(
                "rename target, new name, or collision is invalid",
            ));
        }
        let mut changes = HashMap::new();
        for workspace_edit in workspace_edits {
            let Ok(edit_uri) = workspace_edit.uri.parse::<Uri>() else {
                continue;
            };
            let Some(edit_document) = documents_by_uri.get(&edit_uri) else {
                continue;
            };
            changes
                .entry(edit_uri)
                .or_insert_with(Vec::new)
                .push(TextEdit {
                    range: to_range(edit_document, workspace_edit.edit.span),
                    new_text: workspace_edit.edit.new_text,
                });
        }
        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }))
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let mut roots = self.workspace_roots();
        let removed = params
            .event
            .removed
            .into_iter()
            .filter_map(|folder| uri_to_file_path(&folder.uri))
            .collect::<HashSet<_>>();
        roots.retain(|root| !removed.contains(root));
        roots.extend(
            params
                .event
                .added
                .into_iter()
                .filter_map(|folder| uri_to_file_path(&folder.uri)),
        );
        roots.sort();
        roots.dedup();
        self.set_workspace_roots(roots);
        self.refresh_and_publish_open().await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.refresh_and_publish_open().await;
    }

    async fn did_create_files(&self, _: CreateFilesParams) {
        self.refresh_and_publish_open().await;
    }

    async fn did_rename_files(&self, _: RenameFilesParams) {
        self.refresh_and_publish_open().await;
    }

    async fn did_delete_files(&self, _: DeleteFilesParams) {
        self.refresh_and_publish_open().await;
    }
}

pub async fn run_stdio() {
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::build(Backend::new)
        .custom_method("linguini/protocolVersion", Backend::protocol_version)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}

pub fn run_stdio_blocking() {
    if let Err(error) = try_run_stdio_blocking() {
        eprintln!("failed to start Linguini language server: {error}");
    }
}

pub fn try_run_stdio_blocking() -> io::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run_stdio());
    Ok(())
}

fn workspace_roots_from_initialize(params: &InitializeParams) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(workspace_folders) = &params.workspace_folders {
        for folder in workspace_folders {
            if let Some(path) = uri_to_file_path(&folder.uri) {
                roots.push(path);
            }
        }
    }

    if roots.is_empty() {
        if let Some(uri) = &params.root_uri {
            if let Some(path) = uri_to_file_path(uri) {
                roots.push(path);
            }
        }
    }

    if roots.is_empty() {
        if let Some(root_path) = &params.root_path {
            roots.push(PathBuf::from(root_path));
        }
    }

    if roots.is_empty() {
        roots.extend(std::env::current_dir().ok());
    }

    roots.sort();
    roots.dedup();
    roots
}

fn build_workspace_index(mut roots: Vec<PathBuf>) -> WorkspaceIndex {
    let mut index = WorkspaceIndex::default();
    let mut contexts = Vec::new();
    let mut seen_contexts = HashSet::new();
    let mut remaining_entries = MAX_WORKSPACE_ENTRIES;

    roots.sort();
    roots.dedup();
    if roots.len() > MAX_WORKSPACE_ROOTS {
        push_index_error(
            &mut index.errors,
            format!(
                "workspace exposes {} roots; indexing first {MAX_WORKSPACE_ROOTS}",
                roots.len()
            ),
        );
        roots.truncate(MAX_WORKSPACE_ROOTS);
    }
    for root in roots {
        if remaining_entries == 0 || contexts.len() >= MAX_WORKSPACE_PROJECTS {
            break;
        }
        let root = match fs::canonicalize(&root) {
            Ok(root) => root,
            Err(error) => {
                push_index_error(
                    &mut index.errors,
                    format!("cannot index workspace root {}: {error}", root.display()),
                );
                continue;
            }
        };
        let mut state = WorkspaceDiscovery {
            root: root.clone(),
            visited: HashSet::new(),
            entries: 0,
            entry_limit: remaining_entries,
        };
        discover_project_contexts_recursively(
            &root,
            0,
            &mut state,
            &mut seen_contexts,
            &mut contexts,
            &mut index.errors,
        );
        remaining_entries = remaining_entries.saturating_sub(state.entries);
    }

    contexts.sort_by(|left, right| {
        right
            .root
            .components()
            .count()
            .cmp(&left.root.components().count())
            .then_with(|| left.root.cmp(&right.root))
    });
    if contexts.len() > MAX_WORKSPACE_PROJECTS {
        push_index_error(
            &mut index.errors,
            format!(
                "workspace exposes {} Linguini projects; indexing first {MAX_WORKSPACE_PROJECTS}",
                contexts.len()
            ),
        );
        contexts.truncate(MAX_WORKSPACE_PROJECTS);
    }
    let mut next_source_id = 1u32;
    let mut remaining_files = MAX_INDEXED_FILES;
    let mut remaining_bytes = MAX_INDEXED_BYTES;
    let mut seen_sources = HashSet::new();
    for context in contexts {
        if remaining_files == 0 || remaining_bytes == 0 {
            push_index_error(
                &mut index.errors,
                format!(
                    "workspace index reached its global limit of {MAX_INDEXED_FILES} files or {MAX_INDEXED_BYTES} bytes"
                ),
            );
            break;
        }
        index.projects.push(load_project_snapshot(
            context,
            &mut index.errors,
            &mut next_source_id,
            &mut remaining_files,
            &mut remaining_bytes,
            &mut seen_sources,
        ));
    }
    index
        .projects
        .sort_by(|left, right| left.context.root.cmp(&right.context.root));
    index
}

struct WorkspaceDiscovery {
    root: PathBuf,
    visited: HashSet<PathBuf>,
    entries: usize,
    entry_limit: usize,
}

fn discover_project_contexts_recursively(
    directory: &Path,
    depth: usize,
    state: &mut WorkspaceDiscovery,
    seen_contexts: &mut HashSet<PathBuf>,
    contexts: &mut Vec<ProjectContext>,
    errors: &mut Vec<String>,
) {
    if depth > MAX_WORKSPACE_DEPTH {
        push_index_error(
            errors,
            format!(
                "workspace discovery depth exceeds {MAX_WORKSPACE_DEPTH} under {}",
                state.root.display()
            ),
        );
        return;
    }
    let metadata = match fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) => {
            push_index_error(
                errors,
                format!("cannot inspect {}: {error}", directory.display()),
            );
            return;
        }
    };
    if metadata.file_type().is_symlink() {
        push_index_error(
            errors,
            format!(
                "workspace discovery skipped symlink {}",
                directory.display()
            ),
        );
        return;
    }
    let canonical = match fs::canonicalize(directory) {
        Ok(canonical) => canonical,
        Err(error) => {
            push_index_error(
                errors,
                format!("cannot canonicalize {}: {error}", directory.display()),
            );
            return;
        }
    };
    if !canonical.starts_with(&state.root) || !state.visited.insert(canonical) {
        return;
    }
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            push_index_error(
                errors,
                format!("cannot read directory {}: {error}", directory.display()),
            );
            return;
        }
    };

    for entry in entries {
        if state.entries >= state.entry_limit {
            push_index_error(
                errors,
                format!(
                    "workspace discovery reached its global limit of {MAX_WORKSPACE_ENTRIES} entries"
                ),
            );
            return;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                push_index_error(errors, format!("workspace directory entry failed: {error}"));
                continue;
            }
        };
        state.entries += 1;
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                push_index_error(
                    errors,
                    format!("cannot inspect workspace entry {}: {error}", path.display()),
                );
                continue;
            }
        };
        if file_type.is_symlink() {
            push_index_error(
                errors,
                format!("workspace discovery skipped symlink {}", path.display()),
            );
            continue;
        }
        if file_type.is_dir() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                push_index_error(
                    errors,
                    format!(
                        "workspace discovery skipped non-UTF-8 directory {}",
                        path.display()
                    ),
                );
                continue;
            };
            if should_skip_discovery_directory(name) {
                continue;
            }
            discover_project_contexts_recursively(
                &path,
                depth + 1,
                state,
                seen_contexts,
                contexts,
                errors,
            );
            continue;
        }

        if !file_type.is_file()
            || path.file_name().and_then(|name| name.to_str()) != Some(DEFAULT_CONFIG_FILE)
        {
            continue;
        }
        let Some(root) = path.parent() else {
            continue;
        };
        let root = normalize_existing_path(root);
        if !seen_contexts.insert(root.clone()) {
            continue;
        }
        if contexts.len() >= MAX_WORKSPACE_PROJECTS {
            push_index_error(
                errors,
                format!(
                    "workspace discovery reached its global limit of {MAX_WORKSPACE_PROJECTS} projects"
                ),
            );
            return;
        }
        match load_project_context(&root) {
            Ok(context) => contexts.push(context),
            Err(error) => push_index_error(errors, error),
        }
    }
}

fn should_skip_discovery_directory(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | ".svelte-kit"
            | ".next"
            | ".nuxt"
            | ".turbo"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | "vendor"
    )
}

fn load_project_context(root: &Path) -> std::result::Result<ProjectContext, String> {
    let config_path = root.join(DEFAULT_CONFIG_FILE);
    let source = fs::read_to_string(&config_path)
        .map_err(|error| format!("cannot read {}: {error}", config_path.display()))?;
    let config = parse_config(&source)
        .map_err(|error| format!("invalid {}: {error}", config_path.display()))?;
    Ok(ProjectContext {
        root: normalize_existing_path(root),
        schema_root: normalize_existing_path(root.join(config.paths().schema())),
        locale_root: normalize_existing_path(root.join(config.paths().locale())),
    })
}

fn load_project_snapshot(
    context: ProjectContext,
    errors: &mut Vec<String>,
    next_source_id: &mut u32,
    remaining_files: &mut usize,
    remaining_bytes: &mut u64,
    seen_sources: &mut HashSet<PathBuf>,
) -> ProjectSnapshot {
    let mut sources = Vec::new();
    match discover_schema_files(&context.schema_root) {
        Ok(files) => {
            sources.extend(
                files
                    .into_iter()
                    .map(|file| (file.path, file.namespace, None, "linguini-schema")),
            );
        }
        Err(error) => push_index_error(
            errors,
            format!(
                "cannot discover schema sources for {}: {error}",
                context.root.display()
            ),
        ),
    }
    match discover_locale_files(&context.locale_root) {
        Ok(files) => {
            sources.extend(files.into_iter().map(|file| {
                (
                    file.path,
                    file.namespace,
                    Some(file.locale),
                    "linguini-locale",
                )
            }));
        }
        Err(error) => push_index_error(
            errors,
            format!(
                "cannot discover locale sources for {}: {error}",
                context.root.display()
            ),
        ),
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    sources.dedup_by(|left, right| left.0 == right.0);
    let file_limit = MAX_PROJECT_FILES.min(*remaining_files);
    if sources.len() > file_limit {
        push_index_error(
            errors,
            format!(
                "project {} exceeds its remaining index budget of {file_limit} source files",
                context.root.display(),
            ),
        );
        sources.truncate(file_limit);
    }

    let mut bytes = 0u64;
    let mut documents = Vec::new();
    for (path, namespace, locale, language_id) in sources {
        let normalized_path = normalize_existing_path(&path);
        if !seen_sources.insert(normalized_path) {
            continue;
        }
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                push_index_error(
                    errors,
                    format!("cannot inspect source {}: {error}", path.display()),
                );
                continue;
            }
        };
        if metadata.len() > MAX_DOCUMENT_BYTES {
            push_index_error(
                errors,
                format!(
                    "source {} exceeds {} bytes",
                    path.display(),
                    MAX_DOCUMENT_BYTES
                ),
            );
            continue;
        }
        let mut source = String::new();
        let source_result = fs::File::open(&path).and_then(|file| {
            file.take(MAX_DOCUMENT_BYTES + 1)
                .read_to_string(&mut source)
        });
        match source_result {
            Ok(_) => {}
            Err(error) => {
                push_index_error(
                    errors,
                    format!("cannot read source {}: {error}", path.display()),
                );
                continue;
            }
        }
        if source.len() as u64 > MAX_DOCUMENT_BYTES {
            push_index_error(
                errors,
                format!(
                    "source {} grew beyond {} bytes while indexing",
                    path.display(),
                    MAX_DOCUMENT_BYTES
                ),
            );
            continue;
        }
        let source_bytes = source.len() as u64;
        if source_bytes > MAX_PROJECT_BYTES.saturating_sub(bytes) {
            push_index_error(
                errors,
                format!(
                    "project {} exceeds {} indexed source bytes",
                    context.root.display(),
                    MAX_PROJECT_BYTES
                ),
            );
            break;
        }
        if source_bytes > *remaining_bytes {
            push_index_error(
                errors,
                format!(
                    "workspace exceeds {MAX_INDEXED_BYTES} indexed source bytes while loading {}",
                    path.display()
                ),
            );
            break;
        }
        let Some(uri) = path_to_uri(&path) else {
            push_index_error(
                errors,
                format!("cannot convert source path to URI: {}", path.display()),
            );
            continue;
        };
        let source_id = SourceId(*next_source_id);
        *next_source_id = next_source_id.saturating_add(1);
        let document = LinguiniDocument::new(uri.to_string(), language_id, source)
            .with_source_identity(namespace, locale)
            .with_source_id(source_id);
        if document.is_within_safety_limits() {
            prime_semantic_cache(&document);
        }
        bytes += source_bytes;
        *remaining_bytes -= source_bytes;
        *remaining_files -= 1;
        documents.push(document);
    }

    ProjectSnapshot { context, documents }
}

fn path_matches_context(path: &Path, context: &ProjectContext) -> bool {
    path.starts_with(&context.schema_root) || path.starts_with(&context.locale_root)
}

fn document_file_path(document: &LinguiniDocument) -> Option<PathBuf> {
    let uri = document.uri.parse::<Uri>().ok()?;
    uri_to_file_path(&uri)
}

fn normalize_existing_path(path: impl AsRef<Path>) -> PathBuf {
    fs::canonicalize(path.as_ref()).unwrap_or_else(|_| path.as_ref().to_path_buf())
}

fn uri_to_file_path(uri: &Uri) -> Option<PathBuf> {
    if uri.scheme() != "file" || uri.query().is_some() || uri.fragment().is_some() {
        return None;
    }
    uri.to_file_path().ok()
}

fn path_to_uri(path: &Path) -> Option<Uri> {
    Uri::from_file_path(path).ok()
}

fn push_index_error(errors: &mut Vec<String>, error: String) {
    if errors.len() < 100 {
        errors.push(error);
    }
}

fn workspace_file_operation_capabilities() -> WorkspaceFileOperationsServerCapabilities {
    let registration = FileOperationRegistrationOptions {
        filters: vec![FileOperationFilter {
            scheme: Some("file".to_owned()),
            pattern: FileOperationPattern {
                glob: "**/{linguini.toml,*.lgs,*.lgl}".to_owned(),
                matches: Some(FileOperationPatternKind::File),
                options: None,
            },
        }],
    };
    WorkspaceFileOperationsServerCapabilities {
        did_create: Some(registration.clone()),
        did_rename: Some(registration.clone()),
        did_delete: Some(registration),
        ..Default::default()
    }
}

fn apply_client_formatting_options(text: &mut String, options: &FormattingOptions) {
    if options.trim_trailing_whitespace == Some(true) {
        let had_final_newline = text.ends_with('\n');
        *text = text
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n");
        if had_final_newline {
            text.push('\n');
        }
    }
    if options.trim_final_newlines == Some(true) {
        let had_final_newline = text.ends_with('\n');
        if had_final_newline {
            let trimmed = text.trim_end_matches(['\r', '\n']).len();
            text.truncate(trimmed);
            text.push('\n');
        }
    }
    if options.insert_final_newline == Some(true) && !text.ends_with('\n') {
        text.push('\n');
    }
}

fn minimal_formatting_edit(
    document: &LinguiniDocument,
    formatted: &str,
) -> Option<crate::document::TextEdit> {
    if document.text == formatted {
        return None;
    }

    let mut prefix = document
        .text
        .bytes()
        .zip(formatted.bytes())
        .take_while(|(left, right)| left == right)
        .count();
    while !document.text.is_char_boundary(prefix) || !formatted.is_char_boundary(prefix) {
        prefix = prefix.saturating_sub(1);
    }

    let max_suffix = document
        .text
        .len()
        .min(formatted.len())
        .saturating_sub(prefix);
    let mut suffix = document
        .text
        .bytes()
        .rev()
        .zip(formatted.bytes().rev())
        .take(max_suffix)
        .take_while(|(left, right)| left == right)
        .count();
    while !document.text.is_char_boundary(document.text.len() - suffix)
        || !formatted.is_char_boundary(formatted.len() - suffix)
    {
        suffix = suffix.saturating_sub(1);
    }

    Some(crate::document::TextEdit {
        span: linguini_syntax::Span::in_source(
            document.source_id,
            prefix,
            document.text.len() - suffix,
        ),
        new_text: formatted[prefix..formatted.len() - suffix].to_owned(),
    })
}

fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("linguini-lsp: recovering poisoned shared state for reading");
            poisoned.into_inner()
        }
    }
}

fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("linguini-lsp: recovering poisoned shared state for writing");
            poisoned.into_inner()
        }
    }
}

fn to_range(document: &LinguiniDocument, span: linguini_syntax::Span) -> Range {
    let ((start_line, start_character), (end_line, end_character)) = document.range(span);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

fn encode_semantic_tokens(tokens: Vec<crate::LinguiniSemanticToken>) -> Vec<SemanticToken> {
    let mut previous_line = 0;
    let mut previous_start = 0;
    tokens
        .into_iter()
        .map(|token| {
            let delta_line = token.line - previous_line;
            let delta_start = if delta_line == 0 {
                token.start - previous_start
            } else {
                token.start
            };
            previous_line = token.line;
            previous_start = token.start;
            SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.modifiers,
            }
        })
        .collect()
}

fn symbol_kind(detail: &str) -> SymbolKind {
    match detail {
        "enum" => SymbolKind::ENUM,
        "type" => SymbolKind::TYPE_PARAMETER,
        "variable" => SymbolKind::VARIABLE,
        "function" => SymbolKind::FUNCTION,
        "message group" => SymbolKind::NAMESPACE,
        _ => SymbolKind::STRING,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_client_formatting_options, minimal_formatting_edit, path_to_uri,
        protocol_version_response, uri_to_file_path,
    };
    use std::path::Path;
    use tower_lsp_server::lsp_types::{FormattingOptions, Url as Uri};

    #[test]
    fn protocol_handshake_is_stable_and_versioned() {
        let response = protocol_version_response();

        assert_eq!(response["protocolVersion"], "1");
        assert_eq!(response["compilerVersion"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn formatting_options_preserve_or_normalize_final_newlines() {
        let mut preserved = "one  \ntwo  \n".to_owned();
        let mut trim_only = FormattingOptions {
            tab_size: 2,
            insert_spaces: true,
            ..FormattingOptions::default()
        };
        trim_only.trim_trailing_whitespace = Some(true);
        apply_client_formatting_options(&mut preserved, &trim_only);
        assert_eq!(preserved, "one\ntwo\n");

        let mut normalized = "one\n\n\n".to_owned();
        let mut trim_final = FormattingOptions {
            tab_size: 2,
            insert_spaces: true,
            ..FormattingOptions::default()
        };
        trim_final.trim_final_newlines = Some(true);
        apply_client_formatting_options(&mut normalized, &trim_final);
        assert_eq!(normalized, "one\n");

        let mut absent = "one".to_owned();
        apply_client_formatting_options(&mut absent, &trim_final);
        assert_eq!(absent, "one");
    }

    #[test]
    fn formatting_uses_a_minimal_unicode_safe_edit() {
        let document = crate::LinguiniDocument::new(
            "file:///tmp/shop.lgl",
            "linguini-locale",
            "hello = Привет  \nnext = Мир\n",
        );

        let edit = minimal_formatting_edit(&document, "hello = Привет\nnext = Мир\n")
            .expect("formatting edit");

        assert_eq!(&document.text[edit.span.start..edit.span.end], "  ");
        assert!(document.text.is_char_boundary(edit.span.start));
        assert!(document.text.is_char_boundary(edit.span.end));
        assert!(edit.new_text.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn file_uri_conversion_preserves_non_utf8_paths_and_rejects_authorities() {
        use std::os::unix::ffi::OsStrExt;

        let uri = "file:///tmp/%FF.lgl".parse::<Uri>().expect("valid URI");
        let path = uri_to_file_path(&uri).expect("local file URI");
        assert_eq!(path.as_os_str().as_bytes(), b"/tmp/\xff.lgl");
        assert_eq!(
            path_to_uri(&path).expect("path URI").as_str(),
            "file:///tmp/%FF.lgl"
        );

        let remote = "file://example.com/tmp/shop.lgl"
            .parse::<Uri>()
            .expect("valid URI");
        assert!(uri_to_file_path(&remote).is_none());
        assert!(path_to_uri(Path::new("relative.lgl")).is_none());
    }
}
