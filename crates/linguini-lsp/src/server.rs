#![allow(deprecated)]

use crate::{
    completion_items, diagnostics, document_symbols, format_document, hover_at, references_at,
    semantic_tokens, LinguiniDocument, SemanticLegend,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_lsp_server::{jsonrpc::Result, ls_types::*, Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Uri, LinguiniDocument>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn publish(&self, document: &LinguiniDocument) {
        let diagnostics = diagnostics(document)
            .into_iter()
            .map(|diagnostic| Diagnostic {
                range: to_range(document, diagnostic.span),
                severity: Some(match diagnostic.severity {
                    linguini_analyzer::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                    linguini_analyzer::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                    linguini_analyzer::DiagnosticSeverity::Advice => DiagnosticSeverity::HINT,
                }),
                source: Some("linguini".to_owned()),
                message: diagnostic.message,
                ..Default::default()
            })
            .collect();
        let Ok(uri) = document.uri.parse::<Uri>() else {
            return;
        };
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    fn document(&self, uri: &Uri) -> Option<LinguiniDocument> {
        self.documents.read().ok()?.get(uri).cloned()
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
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
        self.publish(&document).await;
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
        self.publish(&document).await;
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
        Ok(hover_at(&document, offset).map(|value| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        }))
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
        let Some(span) = references_at(&document, offset).into_iter().next() else {
            return Ok(None);
        };
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: to_range(&document, span),
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
        let actions = params
            .context
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Show Linguini diagnostic: {}", diagnostic.message),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic]),
                    ..Default::default()
                })
            })
            .collect();
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
        let Some(span) = references_at(&document, offset).into_iter().next() else {
            return Ok(None);
        };
        Ok(Some(PrepareRenameResponse::Range(to_range(&document, span))))
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
        let edits = references_at(&document, offset)
            .into_iter()
            .map(|span| TextEdit {
                range: to_range(&document, span),
                new_text: params.new_name.clone(),
            })
            .collect();
        let mut changes = HashMap::new();
        changes.insert(uri, edits);
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
        "function" => SymbolKind::FUNCTION,
        "message group" => SymbolKind::NAMESPACE,
        _ => SymbolKind::STRING,
    }
}
