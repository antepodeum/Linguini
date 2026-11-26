use futures::{channel::mpsc, future::poll_fn, SinkExt, StreamExt};
use js_sys::Function;
use linguini_lsp::{
    LspScheduler, LspTask, MemoryWorkspace, WorkspaceFile, WORKSPACE_IGNORED_DIRECTORIES,
    WORKSPACE_LIMITS,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_service::Service;
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(callback: &JsValue, milliseconds: i32);
}

#[derive(Debug)]
struct WorkerScheduler;
impl LspScheduler for WorkerScheduler {
    fn spawn_after(&self, delay: Duration, task: LspTask) {
        let callback = Closure::once_into_js(move || spawn_local(task));
        set_timeout(&callback, delay.as_millis().min(i32::MAX as u128) as i32);
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedFile {
    uri: String,
    text: String,
}

/// Transport for the shared Tower LSP service. Own one instance per worker.
#[wasm_bindgen]
pub struct WorkerServer {
    workspace: MemoryWorkspace,
    requests: mpsc::UnboundedSender<Request>,
    responses: mpsc::UnboundedSender<Response>,
}

#[wasm_bindgen]
impl WorkerServer {
    #[wasm_bindgen(constructor)]
    pub fn new(send: Function) -> Self {
        let workspace = MemoryWorkspace::default();
        let (mut service, socket) = workspace.service(Arc::new(WorkerScheduler));
        let (requests, mut request_stream) = mpsc::unbounded();
        let (responses, mut response_stream) = mpsc::unbounded();
        let (mut outgoing, mut incoming) = socket.split();
        let sender = send.clone();
        spawn_local(async move {
            while let Some(message) = outgoing.next().await {
                emit(&sender, &message);
            }
        });
        spawn_local(async move {
            while let Some(response) = response_stream.next().await {
                if incoming.send(response).await.is_err() {
                    break;
                }
            }
        });
        spawn_local(async move {
            while let Some(request) = request_stream.next().await {
                if poll_fn(|cx| service.poll_ready(cx)).await.is_err() {
                    break;
                }
                let response = service.call(request);
                let sender = send.clone();
                // A server request can await a client response. Keep accepting that response
                // and subsequent messages instead of blocking the worker's receive loop.
                spawn_local(async move {
                    if let Ok(Some(response)) = response.await {
                        emit(&sender, &response);
                    }
                });
            }
        });
        Self {
            workspace,
            requests,
            responses,
        }
    }

    /// Route a standard JSON-RPC request, notification, or client response.
    pub fn receive(&self, message: &str) -> Result<(), JsError> {
        let value: Value = serde_json::from_str(message)?;
        if value.get("method").is_some() {
            self.requests
                .unbounded_send(serde_json::from_value(value)?)
                .map_err(|_| JsError::new("language server exited"))
        } else {
            self.responses
                .unbounded_send(serde_json::from_value(value)?)
                .map_err(|_| JsError::new("language server exited"))
        }
    }

    /// Replace the saved workspace before initialization or after workspace discovery changes.
    pub fn replace_files(&self, files: &str) -> Result<(), JsError> {
        let files: Vec<SavedFile> = serde_json::from_str(files)?;
        self.workspace
            .replace(files.into_iter().map(|file| WorkspaceFile {
                uri: file.uri,
                text: file.text,
            }))
            .map_err(|error| JsError::new(&error))
    }

    /// Update or remove one saved file before a standard watched-files notification.
    pub fn update_file(&self, uri: &str, text: Option<String>) -> Result<(), JsError> {
        self.workspace
            .update(uri, text)
            .map_err(|error| JsError::new(&error))
    }

    /// Ask the Rust config parser which source roots the embedding host needs to read.
    pub fn source_plan(&self, roots: &str) -> Result<String, JsError> {
        let roots: Vec<String> = serde_json::from_str(roots)?;
        let plan = self
            .workspace
            .source_plan(&roots)
            .map_err(|error| JsError::new(&error))?;
        let roots = plan.roots.into_iter().map(|root| json!({"configUri":root.config_uri,"schemaUri":root.schema_uri,"localeUri":root.locale_uri})).collect::<Vec<_>>();
        Ok(serde_json::to_string(
            &json!({"roots": roots, "errors": plan.errors}),
        )?)
    }
}

impl Drop for WorkerServer {
    fn drop(&mut self) {
        self.requests.close_channel();
        self.responses.close_channel();
    }
}

fn emit(send: &Function, message: &impl serde::Serialize) {
    if let Ok(json) = serde_json::to_string(message) {
        let _ = send.call1(&JsValue::UNDEFINED, &JsValue::from_str(&json));
    }
}

/// Discovery exclusions are owned by the shared Rust host policy.
#[wasm_bindgen]
pub fn discovery_options() -> String {
    json!({"ignoredDirectories": WORKSPACE_IGNORED_DIRECTORIES, "maxEntries": WORKSPACE_LIMITS.max_entries,
        "maxDepth": WORKSPACE_LIMITS.max_depth, "maxRoots": WORKSPACE_LIMITS.max_roots,
        "maxFileBytes": WORKSPACE_LIMITS.max_file_bytes, "maxBytes": WORKSPACE_LIMITS.max_bytes}).to_string()
}
