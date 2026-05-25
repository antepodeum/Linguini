#![allow(deprecated)]

use crate::{
    code_action::{analyzer_quick_fix_actions, diagnostic_display_actions, to_lsp_diagnostic},
    completion_items, definition_at_with_workspace, diagnostics_with_workspace, document_symbols,
    format_document, hover_at_with_workspace, prepare_rename_at, references_at,
    rename_workspace_edits, semantic_tokens, LinguiniDocument, SemanticLegend,
};
use linguini_config::{
    discover_locale_files, discover_schema_files, parse_config, LinguiniConfig, DEFAULT_CONFIG_FILE,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tower_lsp_server::{jsonrpc::Result, ls_types::*, Client, LanguageServer, LspService, Server};

#[derive(Debug, Clone, Eq, PartialEq)]
struct ProjectContext {
    root: PathBuf,
    schema_root: PathBuf,
    locale_root: PathBuf,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Uri, LinguiniDocument>>>,
    workspace_roots: Arc<RwLock<Vec<PathBuf>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            workspace_roots: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn publish(&self, document: &LinguiniDocument) {
        let diagnostics = self
            .diagnostics_for(document)
            .into_iter()
            .map(|diagnostic| to_lsp_diagnostic(document, &diagnostic))
            .collect();
        let Ok(uri) = document.uri.parse::<Uri>() else {
            return;
        };
        self.client
            .publish_diagnostics(uri, diagnostics, None)
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

    fn diagnostics_for(&self, document: &LinguiniDocument) -> Vec<linguini_analyzer::Diagnostic> {
        diagnostics_with_workspace(document, self.workspace_documents_for(document))
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

        self.config_workspace_documents(&context, &open_documents)
    }

    fn config_workspace_documents(
        &self,
        context: &ProjectContext,
        open_documents: &[LinguiniDocument],
    ) -> Vec<LinguiniDocument> {
        let mut documents = Vec::new();
        let mut seen_paths = HashSet::new();
        let open_by_path = open_documents
            .iter()
            .filter_map(|document| document_file_path(document).map(|path| (path, document)))
            .collect::<HashMap<_, _>>();

        for source_path in discover_config_source_paths(context) {
            let path = normalize_existing_path(&source_path);
            if !seen_paths.insert(path.clone()) {
                continue;
            }

            if let Some(open_document) = open_by_path.get(&path) {
                documents.push((**open_document).clone());
                continue;
            }

            let Some(document) = read_document_from_path(&path) else {
                continue;
            };
            documents.push(document);
        }

        for document in open_documents {
            let Some(path) = document_file_path(document) else {
                continue;
            };
            if !path_matches_context(&path, context) || !seen_paths.insert(path) {
                continue;
            }
            documents.push(document.clone());
        }

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
        self.documents
            .read()
            .ok()
            .map(|documents| documents.values().cloned().collect())
            .unwrap_or_default()
    }

    fn document(&self, uri: &Uri) -> Option<LinguiniDocument> {
        self.documents.read().ok()?.get(uri).cloned()
    }

    fn set_workspace_roots(&self, roots: Vec<PathBuf>) {
        if let Ok(mut workspace_roots) = self.workspace_roots.write() {
            *workspace_roots = roots;
        }
    }

    fn workspace_roots(&self) -> Vec<PathBuf> {
        self.workspace_roots
            .read()
            .ok()
            .filter(|roots| !roots.is_empty())
            .map(|roots| roots.clone())
            .unwrap_or_else(|| std::env::current_dir().into_iter().collect())
    }

    fn project_context_for_path(&self, path: &Path) -> Option<ProjectContext> {
        let path = normalize_existing_path(path);
        let mut nearest_config_context = None;

        for root in path.ancestors() {
            let config_path = root.join(DEFAULT_CONFIG_FILE);
            if !config_path.is_file() {
                continue;
            }
            let Some(context) = load_project_context(root) else {
                continue;
            };
            if nearest_config_context.is_none() {
                nearest_config_context = Some(context.clone());
            }
            if path_matches_context(&path, &context) {
                return Some(context);
            }
        }

        if nearest_config_context.is_some() {
            return nearest_config_context;
        }

        discover_workspace_project_contexts(self.workspace_roots())
            .into_iter()
            .filter(|context| path_matches_context(&path, context))
            .max_by_key(|context| context.root.components().count())
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.set_workspace_roots(workspace_roots_from_initialize(&params));

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
                rename_provider: Some(OneOf::Left(true)),
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
                ..Default::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Linguini language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = LinguiniDocument::new(
            params.text_document.uri.to_string(),
            params.text_document.language_id,
            params.text_document.text,
        );
        if let Ok(uri) = document.uri.parse::<Uri>() {
            if let Ok(mut documents) = self.documents.write() {
                documents.insert(uri, document.clone());
            }
        }
        self.publish_related(&document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };
        let language_id = self
            .document(&params.text_document.uri)
            .map(|document| document.language_id)
            .unwrap_or_else(|| "linguini-locale".to_owned());
        let document = LinguiniDocument::new(
            params.text_document.uri.to_string(),
            language_id,
            change.text,
        );
        if let Ok(mut documents) = self.documents.write() {
            documents.insert(params.text_document.uri, document.clone());
        }
        self.publish_related(&document).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Ok(mut documents) = self.documents.write() {
            documents.remove(&params.text_document.uri);
        }
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let Some(document) = self.document(&params.text_document_position.text_document.uri) else {
            return Ok(None);
        };
        let offset = document.offset(
            params.text_document_position.position.line,
            params.text_document_position.position.character,
        );
        let items = completion_items(&document, offset)
            .into_iter()
            .map(|label| CompletionItem {
                label,
                kind: Some(CompletionItemKind::KEYWORD),
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
        let Some((target_uri, span)) = definition_at_with_workspace(
            &document,
            offset,
            self.workspace_documents_for(&document),
        ) else {
            return Ok(None);
        };
        let uri = match target_uri.parse::<Uri>() {
            Ok(parsed) => parsed,
            Err(_) => uri,
        };
        let target_document = self
            .document(&uri)
            .or_else(|| {
                uri_to_file_path(&uri.to_string()).and_then(|path| read_document_from_path(&path))
            })
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
        let locations = references_at(&document, offset)
            .into_iter()
            .map(|span| Location {
                uri: uri.clone(),
                range: to_range(&document, span),
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
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        let query = params.query.to_lowercase();
        let mut symbols = Vec::new();
        if let Ok(documents) = self.documents.read() {
            for (uri, document) in documents.iter() {
                symbols.extend(document_symbols(document).into_iter().filter_map(|symbol| {
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
                                range: to_range(document, symbol.span),
                            },
                            container_name: Some(symbol.detail),
                        })
                }));
            }
        }
        Ok(Some(WorkspaceSymbolResponse::Flat(symbols)))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(Some(Vec::new()));
        };
        let mut actions: CodeActionResponse =
            diagnostic_display_actions(params.context.diagnostics);
        actions.extend(analyzer_quick_fix_actions(
            &params.text_document.uri,
            &document,
            params.range,
            self.diagnostics_for(&document),
        ));

        if params.range.start == params.range.end {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Rename Linguini symbol".to_owned(),
                kind: Some(CodeActionKind::REFACTOR),
                command: Some(Command {
                    title: "Rename Linguini symbol".to_owned(),
                    command: "editor.action.rename".to_owned(),
                    arguments: None,
                }),
                ..Default::default()
            }));
        }
        Ok(Some(actions))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        let edit = match format_document(&document) {
            Ok(edit) => edit,
            Err(_) => return Ok(None),
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
        let mut changes = HashMap::new();
        for workspace_edit in
            rename_workspace_edits(workspace_documents, &document, offset, &params.new_name)
        {
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
}

pub async fn run_stdio() {
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

pub fn run_stdio_blocking() {
    let runtime = tokio::runtime::Runtime::new().expect("create Tokio runtime for Linguini LSP");
    runtime.block_on(run_stdio());
}

fn workspace_roots_from_initialize(params: &InitializeParams) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(workspace_folders) = &params.workspace_folders {
        for folder in workspace_folders {
            if let Some(path) = uri_to_file_path(&folder.uri.to_string()) {
                roots.push(normalize_existing_path(path));
            }
        }
    }

    if roots.is_empty() {
        if let Some(uri) = &params.root_uri {
            if let Some(path) = uri_to_file_path(&uri.to_string()) {
                roots.push(normalize_existing_path(path));
            }
        }
    }

    if roots.is_empty() {
        if let Some(root_path) = &params.root_path {
            roots.push(normalize_existing_path(root_path.as_str()));
        }
    }

    if roots.is_empty() {
        roots.extend(std::env::current_dir().ok());
    }

    roots.sort();
    roots.dedup();
    roots
}

fn discover_workspace_project_contexts(roots: Vec<PathBuf>) -> Vec<ProjectContext> {
    let mut contexts = Vec::new();
    let mut seen = HashSet::new();

    for root in roots {
        discover_project_contexts_recursively(&root, &mut seen, &mut contexts);
    }

    contexts
}

fn discover_project_contexts_recursively(
    directory: &Path,
    seen: &mut HashSet<PathBuf>,
    contexts: &mut Vec<ProjectContext>,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if should_skip_discovery_directory(name) {
                continue;
            }
            discover_project_contexts_recursively(&path, seen, contexts);
            continue;
        }

        if path.file_name().and_then(|name| name.to_str()) != Some(DEFAULT_CONFIG_FILE) {
            continue;
        }
        let Some(root) = path.parent() else {
            continue;
        };
        let root = normalize_existing_path(root);
        if !seen.insert(root.clone()) {
            continue;
        }
        if let Some(context) = load_project_context(&root) {
            contexts.push(context);
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

fn load_project_context(root: &Path) -> Option<ProjectContext> {
    let config = load_config(root)?;
    Some(ProjectContext {
        root: normalize_existing_path(root),
        schema_root: normalize_existing_path(root.join(&config.paths.schema)),
        locale_root: normalize_existing_path(root.join(&config.paths.locale)),
    })
}

fn load_config(root: &Path) -> Option<LinguiniConfig> {
    let source = fs::read_to_string(root.join(DEFAULT_CONFIG_FILE)).ok()?;
    parse_config(&source).ok()
}

fn discover_config_source_paths(context: &ProjectContext) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(schema_files) = discover_schema_files(&context.schema_root) {
        paths.extend(schema_files.into_iter().map(|file| file.path));
    }

    if let Ok(locale_files) = discover_locale_files(&context.locale_root) {
        paths.extend(locale_files.into_iter().map(|file| file.path));
    }

    paths.sort();
    paths.dedup();
    paths
}

fn path_matches_context(path: &Path, context: &ProjectContext) -> bool {
    path.starts_with(&context.schema_root) || path.starts_with(&context.locale_root)
}

fn document_file_path(document: &LinguiniDocument) -> Option<PathBuf> {
    uri_to_file_path(&document.uri).map(normalize_existing_path)
}

fn read_document_from_path(path: &Path) -> Option<LinguiniDocument> {
    let source = fs::read_to_string(path).ok()?;
    let uri = path_to_file_uri(path);
    let language_id = match path.extension().and_then(|extension| extension.to_str()) {
        Some("lgs") => "linguini-schema",
        Some("lgl") => "linguini-locale",
        _ => return None,
    };

    Some(LinguiniDocument::new(uri, language_id, source))
}

fn normalize_existing_path(path: impl AsRef<Path>) -> PathBuf {
    fs::canonicalize(path.as_ref()).unwrap_or_else(|_| path.as_ref().to_path_buf())
}

fn uri_to_file_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let path_part = if rest.starts_with('/') {
        rest
    } else {
        let slash = rest.find('/')?;
        &rest[slash..]
    };
    let decoded = percent_decode(path_part)?;

    #[cfg(windows)]
    {
        let decoded = decoded.trim_start_matches('/');
        return Some(PathBuf::from(decoded));
    }

    #[cfg(not(windows))]
    {
        Some(PathBuf::from(decoded))
    }
}

fn path_to_file_uri(path: &Path) -> String {
    let path = normalize_existing_path(path);
    let path = path.to_string_lossy().replace('\\', "/");

    #[cfg(windows)]
    let path = format!("/{path}");

    format!("file://{}", percent_encode_path(&path))
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).copied()?;
            let low = bytes.get(index + 2).copied()?;
            output.push((hex_value(high)? << 4) | hex_value(low)?);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(output).ok()
}

fn percent_encode_path(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' | b':' => {
                encoded.push(byte as char)
            }
            byte => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
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
