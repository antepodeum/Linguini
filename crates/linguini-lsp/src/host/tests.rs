use super::*;
use futures::{channel::mpsc, future::poll_fn, SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{fs, sync::Arc};
use tower_lsp_server::{
    jsonrpc::{Request, Response},
    LanguageServer, LspService,
};
use tower_service::Service;

const CONFIG: &str = "[project]\nname = \"shop\"\ndefault_locale = \"en\"\nlocales = [\"en\"]\n[paths]\nschema = \"build/schema\"\nlocale = \"locale\"\n";
const SCHEMA: &str = "/// Delivery label\ndelivery(count: Number)\n";
const LOCALE: &str = "// 😀 Unicode\ndelivery = Delivered {count}\n";

fn workspace_file(root: &Path, relative: &str, text: &str) -> WorkspaceFile {
    WorkspaceFile {
        uri: path_to_uri(&root.join(relative)).unwrap().to_string(),
        text: text.into(),
    }
}

async fn request<S: LanguageServer>(
    service: &mut LspService<S>,
    method: &str,
    params: Value,
) -> Value {
    let mut message = json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params});
    if params.is_null() {
        message.as_object_mut().unwrap().remove("params");
    }
    let request: Request = serde_json::from_value(message).unwrap();
    poll_fn(|cx| service.poll_ready(cx)).await.unwrap();
    let response = service.call(request).await.unwrap().unwrap();
    assert!(response.error().is_none(), "{method}: {response:?}");
    response.result().unwrap().clone()
}

async fn notify<S: LanguageServer>(service: &mut LspService<S>, method: &str, params: Value) {
    let mut message = json!({"jsonrpc":"2.0", "method":method, "params":params});
    if params.is_null() {
        message.as_object_mut().unwrap().remove("params");
    }
    let request: Request = serde_json::from_value(message).unwrap();
    poll_fn(|cx| service.poll_ready(cx)).await.unwrap();
    assert!(service.call(request).await.unwrap().is_none());
}

async fn next_diagnostics(
    notifications: &mut mpsc::UnboundedReceiver<Value>,
    uri: &str,
    version: i32,
) -> Value {
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(event) = notifications.next().await {
            if event["method"] == "textDocument/publishDiagnostics"
                && event["params"]["uri"] == uri
                && event["params"]["version"] == version
            {
                return event;
            }
        }
        panic!("notification stream closed before diagnostics for {uri} version {version}");
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for diagnostics for {uri} version {version}"))
}

#[derive(Debug)]
struct DelayedScheduler;

impl LspScheduler for DelayedScheduler {
    fn spawn_after(&self, delay: Duration, task: LspTask) {
        NativeScheduler.spawn_after(delay + Duration::from_millis(150), task);
    }
}

async fn exercise<S: LanguageServer>(
    mut service: LspService<S>,
    socket: tower_lsp_server::ClientSocket,
    root: &Path,
) -> Vec<Value> {
    let (captured, mut notifications) = mpsc::unbounded();
    let pump = tokio::spawn(async move {
        let (mut stream, mut sink) = socket.split();
        while let Some(message) = stream.next().await {
            if let Some(id) = message.id() {
                sink.send(Response::from_ok(id.clone(), Value::Null))
                    .await
                    .unwrap();
            } else {
                captured
                    .unbounded_send(serde_json::to_value(message).unwrap())
                    .unwrap();
            }
        }
    });
    let root_uri = path_to_uri(root).unwrap();
    let locale_uri = path_to_uri(&root.join("locale/shop/en.lgl")).unwrap();
    let schema_uri = path_to_uri(&root.join("build/schema/shop.lgs")).unwrap();
    let mut results = vec![
        request(
            &mut service,
            "initialize",
            json!({"capabilities":{}, "workspaceFolders":[{"uri":root_uri,"name":"shop"}]}),
        )
        .await,
    ];
    notify(&mut service, "initialized", json!({})).await;
    results.push(request(&mut service, "linguini/protocolVersion", json!(null)).await);
    notify(&mut service, "textDocument/didOpen", json!({"textDocument":{"uri":locale_uri,"languageId":"linguini-locale", "version":1,"text":LOCALE}})).await;
    let initial = next_diagnostics(&mut notifications, locale_uri.as_str(), 1).await;
    assert_eq!(initial["params"]["diagnostics"], json!([]));
    let document = json!({"uri":locale_uri});
    let at = json!({"textDocument":document,"position":{"line":1,"character":2}});
    for method in [
        "textDocument/hover",
        "textDocument/definition",
        "textDocument/prepareRename",
    ] {
        results.push(request(&mut service, method, at.clone()).await);
    }
    results.push(request(&mut service, "textDocument/references", json!({"textDocument":document,"position":{"line":1,"character":2},"context":{"includeDeclaration":true}})).await);
    results.push(request(&mut service, "textDocument/rename", json!({"textDocument":document,"position":{"line":1,"character":2},"newName":"shipping"})).await);
    results.push(
        request(
            &mut service,
            "textDocument/completion",
            json!({"textDocument":document,"position":{"line":1,"character":22}}),
        )
        .await,
    );
    results.push(
        request(
            &mut service,
            "textDocument/documentSymbol",
            json!({"textDocument":document}),
        )
        .await,
    );
    results.push(
        request(
            &mut service,
            "workspace/symbol",
            json!({"query":"delivery"}),
        )
        .await,
    );
    results.push(
        request(
            &mut service,
            "textDocument/semanticTokens/full",
            json!({"textDocument":document}),
        )
        .await,
    );
    results.push(
        request(
            &mut service,
            "textDocument/formatting",
            json!({"textDocument":document,"options":{"tabSize":2,"insertSpaces":true}}),
        )
        .await,
    );
    results.push(
        request(
            &mut service,
            "linguini/highlight",
            json!({"language":"linguini-locale","source":LOCALE}),
        )
        .await,
    );
    // Keep one broken open buffer while the saved workspace is valid, then recover it.
    notify(&mut service, "textDocument/didChange", json!({"textDocument":{"uri":locale_uri,"version":2},"contentChanges":[{"text":"delivery = {unknown}\n"}]})).await;
    let broken = next_diagnostics(&mut notifications, locale_uri.as_str(), 2).await;
    assert!(
        !broken["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty(),
        "{broken:#?}"
    );
    results.push(request(&mut service, "textDocument/codeAction", json!({"textDocument":document,"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":20}},"context":{"diagnostics":[]}})).await);
    notify(
        &mut service,
        "textDocument/didChange",
        json!({"textDocument":{"uri":locale_uri,"version":3},"contentChanges":[{"text":LOCALE}]}),
    )
    .await;
    let recovered = next_diagnostics(&mut notifications, locale_uri.as_str(), 3).await;
    assert_eq!(recovered["params"]["diagnostics"], json!([]));
    results.push(json!([initial, broken, recovered]));
    assert!(results[2].to_string().contains("Delivery label"));
    assert!(results[3].to_string().contains(schema_uri.as_str()));
    request(&mut service, "shutdown", json!(null)).await;
    notify(&mut service, "exit", json!(null)).await;
    pump.abort();
    results
}

#[tokio::test]
async fn native_and_memory_hosts_share_protocol_results_and_source_identity() {
    let temp = linguini_test_support::temp_project_dir("lsp-hosts").unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let files = vec![
        workspace_file(&root, "linguini.toml", CONFIG),
        workspace_file(&root, "build/schema/shop.lgs", SCHEMA),
        workspace_file(&root, "locale/shop/en.lgl", LOCALE),
        workspace_file(
            &root,
            "build/schema/account.lgs",
            "delivery(name: String)\n",
        ),
        workspace_file(&root, "locale/account/en.lgl", "delivery = Hello {name}\n"),
    ];
    for file in &files {
        let path = uri_to_file_path(&file.uri.parse().unwrap()).unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, &file.text).unwrap();
    }
    let index = NativeHost
        .workspace_index(vec![root.clone()])
        .await
        .unwrap();
    assert!(index.errors.is_empty(), "{:?}", index.errors);
    assert_eq!(index.projects.len(), 1);
    let context = &index.projects[0].context;
    let wire_path = |path: &Path| uri_to_file_path(&path_to_uri(path).unwrap()).unwrap();
    assert_eq!(context.root, wire_path(&root));
    assert_eq!(context.schema_root, wire_path(&root.join("build/schema")));
    assert_eq!(context.locale_root, wire_path(&root.join("locale")));
    let memory = MemoryWorkspace::default();
    memory.replace(files).unwrap();
    assert_eq!(
        memory
            .source_roots(&[path_to_uri(&root).unwrap().to_string()])
            .unwrap()[0]
            .schema_uri,
        path_to_uri(&root.join("build/schema")).unwrap().to_string()
    );
    let (native_service, native_socket) = crate::server::native_service();
    let native = exercise(native_service, native_socket, &root).await;
    let (memory_service, memory_socket) = memory.service(Arc::new(DelayedScheduler));
    let memory_results = exercise(memory_service, memory_socket, &root).await;
    assert_eq!(native, memory_results);
}

#[test]
fn saved_workspace_updates_are_atomic_and_configuration_is_parsed_in_rust() {
    let memory = MemoryWorkspace::default();
    let root = Path::new(if cfg!(windows) {
        "C:/workspace"
    } else {
        "/workspace"
    });
    let config = workspace_file(root, "linguini.toml", CONFIG);
    memory.replace([config.clone()]).unwrap();
    let roots = [path_to_uri(root).unwrap().to_string()];
    let initial = memory.source_roots(&roots).unwrap();
    assert!(memory.replace([config.clone(), config.clone()]).is_err());
    assert_eq!(memory.source_roots(&roots).unwrap(), initial);
    assert!(memory
        .update("https://example.com/a.lgs", Some(String::new()))
        .is_err());
    assert!(memory
        .update(
            &config.uri,
            Some("x".repeat(MAX_DOCUMENT_BYTES as usize + 1))
        )
        .is_err());
    assert_eq!(memory.source_roots(&roots).unwrap(), initial);
    memory.update(&config.uri, Some("[web]\n".into())).unwrap();
    assert!(memory.source_roots(&roots).unwrap_err().contains("invalid"));
    memory.update(&config.uri, None).unwrap();
    assert!(memory.source_roots(&roots).unwrap().is_empty());
}
