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
    let response = request_response(service, method, params).await;
    assert!(response.error().is_none(), "{method}: {response:?}");
    response.result().unwrap().clone()
}

async fn request_response<S: LanguageServer>(
    service: &mut LspService<S>,
    method: &str,
    params: Value,
) -> Response {
    let mut message = json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params});
    if params.is_null() {
        message.as_object_mut().unwrap().remove("params");
    }
    let request: Request = serde_json::from_value(message).unwrap();
    poll_fn(|cx| service.poll_ready(cx)).await.unwrap();
    service.call(request).await.unwrap().unwrap()
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
    next_diagnostics_matching(notifications, uri, version, |_| true).await
}

async fn next_diagnostics_matching(
    notifications: &mut mpsc::UnboundedReceiver<Value>,
    uri: &str,
    version: i32,
    predicate: impl Fn(&Value) -> bool,
) -> Value {
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(event) = notifications.next().await {
            if event["method"] == "textDocument/publishDiagnostics"
                && event["params"]["uri"] == uri
                && event["params"]["version"] == version
                && predicate(&event)
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

#[tokio::test]
async fn configured_fallback_globals_match_native_and_memory_editor_diagnostics() {
    let temp = linguini_test_support::temp_project_dir("lsp-fallbacks").unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let config = CONFIG.replace(
        "locales = [\"en\"]",
        "locales = [\"en\", \"en-001\", \"en-AU\", \"fr\"]",
    );
    let french = "form Word(Tone) { formal => Oui\ncasual => Non }\nsummary = {render(count)}|{fruit.label}|{suffix}|{Word(fruit.Tone)}\n";
    let australian = "summary = {render(name,count)}|{fruit.label}|{suffix}\n";
    let files = vec![
        workspace_file(&root, "linguini.toml", &config),
        workspace_file(&root, "build/schema/shop.lgs", "enum Fruit { apple }\nsummary(fruit: Fruit, count: Number, name: String)\nother\n"),
        workspace_file(&root, "locale/shop/en.lgl", "let suffix = BASE\nenum Tone { formal, casual }\nfn render(count: Number) { _ => {count}|{suffix} }\nimpl Fruit { apple { label = Apple\nTone = formal } }\nsummary = BASE\nother = OTHER\n"),
        workspace_file(&root, "locale/shop/en-001.lgl", "fn render(name: String, count: Number) { _ => {name}:{count} }\nimpl Fruit { apple { label = Parent } }\n"),
        workspace_file(&root, "locale/shop/en-AU.lgl", australian),
        workspace_file(&root, "locale/shop/fr.lgl", french),
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
    let documents = &index.projects[0].documents;
    for locale in ["en-AU", "fr"] {
        let document = documents
            .iter()
            .find(|document| document.locale.as_deref() == Some(locale))
            .unwrap();
        let diagnostics = crate::diagnostics_with_workspace(document, documents.clone());
        assert!(
            diagnostics
                .iter()
                .all(|item| item.severity != linguini_analyzer::DiagnosticSeverity::Error),
            "{locale}: {diagnostics:?}"
        );
        assert!(
            diagnostics
                .iter()
                .any(|item| item.message.contains("missing 1 schema message: `other`")),
            "{locale}: {diagnostics:?}"
        );
    }
    let (native_service, native_socket) = crate::server::native_service();
    let native =
        exercise_fallback_diagnostics(native_service, native_socket, &root, &files, |source| {
            fs::write(root.join("linguini.toml"), source).unwrap()
        })
        .await;
    let memory = MemoryWorkspace::default();
    memory.replace(files.clone()).unwrap();
    let config_uri = path_to_uri(&root.join("linguini.toml"))
        .unwrap()
        .to_string();
    let (memory_service, memory_socket) = memory.service(Arc::new(DelayedScheduler));
    let browser =
        exercise_fallback_diagnostics(memory_service, memory_socket, &root, &files, |source| {
            memory.update(&config_uri, Some(source.to_owned())).unwrap()
        })
        .await;
    assert_eq!(native, browser);
}

fn has_diagnostic_errors(event: &Value) -> bool {
    event["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["severity"] == 1)
}

async fn exercise_fallback_diagnostics<S: LanguageServer>(
    mut service: LspService<S>,
    socket: tower_lsp_server::ClientSocket,
    root: &Path,
    files: &[WorkspaceFile],
    update_config: impl Fn(&str),
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
    let uri = |relative: &str| path_to_uri(&root.join(relative)).unwrap();
    let source = |relative: &str| {
        let uri = uri(relative).to_string();
        files
            .iter()
            .find(|file| file.uri == uri)
            .unwrap()
            .text
            .as_str()
    };
    let french_uri = uri("locale/shop/fr.lgl");
    let australian_uri = uri("locale/shop/en-AU.lgl");
    let english_uri = uri("locale/shop/en.lgl");
    let config_uri = uri("linguini.toml");
    request(&mut service, "initialize", json!({"capabilities":{},"workspaceFolders":[{"uri":path_to_uri(root).unwrap(),"name":"fallbacks"}]})).await;
    notify(&mut service, "initialized", json!({})).await;
    let mut results = Vec::new();
    for (document_uri, text) in [
        (&french_uri, source("locale/shop/fr.lgl")),
        (&australian_uri, source("locale/shop/en-AU.lgl")),
    ] {
        notify(&mut service, "textDocument/didOpen", json!({"textDocument":{"uri":document_uri,"languageId":"linguini-locale","version":1,"text":text}})).await;
        let initial = next_diagnostics(&mut notifications, document_uri.as_str(), 1).await;
        assert!(!has_diagnostic_errors(&initial), "{initial:#?}");
        results.push(initial);
    }
    for (relative, document_uri) in [
        ("locale/shop/fr.lgl", &french_uri),
        ("locale/shop/en-AU.lgl", &australian_uri),
    ] {
        let document =
            crate::LinguiniDocument::new(document_uri.to_string(), "lgl", source(relative));
        for name in ["render", "suffix"] {
            let target_relative = if name == "render" && relative.ends_with("en-AU.lgl") {
                "locale/shop/en-001.lgl"
            } else {
                "locale/shop/en.lgl"
            };
            let target_uri = uri(target_relative);
            let target = crate::LinguiniDocument::new(
                target_uri.to_string(),
                "lgl",
                source(target_relative),
            );
            let (line, character) = document.position(document.text.find(name).unwrap());
            let at = json!({"textDocument":{"uri":document_uri},"position":{"line":line,"character":character}});
            let definition = request(&mut service, "textDocument/definition", at.clone()).await;
            let start = target.text.find(name).unwrap();
            let (line, character) = target.position(start);
            let (end_line, end_character) = target.position(start + name.len());
            assert_eq!(
                definition,
                json!({"uri":target_uri,"range":{"start":{"line":line,"character":character},"end":{"line":end_line,"character":end_character}}})
            );
            let mut params = at.clone();
            params["context"] = json!({"includeDeclaration":true});
            let references = request(&mut service, "textDocument/references", params).await;
            let references = references.as_array().unwrap();
            assert_eq!(references.len(), if name == "render" { 2 } else { 4 });
            assert!(references
                .iter()
                .all(|item| item["uri"] == target_uri.as_str()
                    || item["uri"] == document_uri.as_str()
                    || name == "suffix"
                        && (item["uri"] == australian_uri.as_str()
                            || item["uri"] == french_uri.as_str())));
            let mut params = at;
            params["newName"] = json!("renamed");
            let renamed = request(&mut service, "textDocument/rename", params).await;
            let mut expected = std::collections::BTreeMap::<String, Vec<Value>>::new();
            for reference in references {
                expected
                    .entry(reference["uri"].as_str().unwrap().into())
                    .or_default()
                    .push(json!({"range":reference["range"],"newText":"renamed"}));
            }
            assert_eq!(renamed["changes"], json!(expected));
            results.extend([definition, json!(references), renamed]);
        }
    }
    let english = source("locale/shop/en.lgl").replacen("count: Number", "count: String", 1);
    notify(&mut service, "textDocument/didOpen", json!({"textDocument":{"uri":english_uri,"languageId":"linguini-locale","version":1,"text":english}})).await;
    let broken = next_diagnostics_matching(
        &mut notifications,
        french_uri.as_str(),
        1,
        has_diagnostic_errors,
    )
    .await;
    assert!(
        broken["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["message"]
                .as_str()
                .unwrap()
                .contains("expects `String`, got `Number`")
                && item["relatedInformation"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|related| related["location"]["uri"] == english_uri.as_str())),
        "{broken:#?}"
    );
    results.push(broken);
    let australian = source("locale/shop/en-AU.lgl");
    notify(&mut service, "textDocument/didChange", json!({"textDocument":{"uri":australian_uri,"version":2},"contentChanges":[{"text":australian}]})).await;
    let parent = next_diagnostics(&mut notifications, australian_uri.as_str(), 2).await;
    assert!(!has_diagnostic_errors(&parent), "{parent:#?}");
    results.push(parent);
    let french = source("locale/shop/fr.lgl").replace("render(count)", "render(name)");
    notify(
        &mut service,
        "textDocument/didChange",
        json!({"textDocument":{"uri":french_uri,"version":2},"contentChanges":[{"text":french}]}),
    )
    .await;
    let recovered = next_diagnostics(&mut notifications, french_uri.as_str(), 2).await;
    assert!(!has_diagnostic_errors(&recovered), "{recovered:#?}");
    results.push(recovered);
    notify(
        &mut service,
        "textDocument/didClose",
        json!({"textDocument":{"uri":english_uri}}),
    )
    .await;
    let saved = next_diagnostics_matching(
        &mut notifications,
        french_uri.as_str(),
        2,
        has_diagnostic_errors,
    )
    .await;
    assert!(
        saved["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["message"]
                .as_str()
                .unwrap()
                .contains("expects `Number`, got `String`")),
        "{saved:#?}"
    );
    results.push(saved);
    notify(&mut service, "textDocument/didChange", json!({"textDocument":{"uri":french_uri,"version":3},"contentChanges":[{"text":source("locale/shop/fr.lgl")}]})).await;
    let recovered = next_diagnostics(&mut notifications, french_uri.as_str(), 3).await;
    assert!(!has_diagnostic_errors(&recovered), "{recovered:#?}");
    results.push(recovered);
    let config = source("linguini.toml");
    update_config(&config.replace("\"en-001\", ", ""));
    notify(
        &mut service,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":config_uri,"type":2}]}),
    )
    .await;
    let changed =
        next_diagnostics_matching(&mut notifications, australian_uri.as_str(), 2, |event| {
            event["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| {
                    item["message"]
                        .as_str()
                        .unwrap()
                        .contains("expects 1 argument(s), got 2")
                })
        })
        .await;
    results.push(changed);
    let offset = australian.find("render").unwrap();
    let (line, character) =
        crate::LinguiniDocument::new("linguini:au", "lgl", australian).position(offset);
    let changed_definition = request(&mut service, "textDocument/definition", json!({"textDocument":{"uri":australian_uri},"position":{"line":line,"character":character}})).await;
    assert_eq!(changed_definition["uri"], english_uri.as_str());
    results.push(changed_definition);
    update_config(config);
    notify(
        &mut service,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":config_uri,"type":2}]}),
    )
    .await;
    notify(&mut service, "textDocument/didChange", json!({"textDocument":{"uri":australian_uri,"version":3},"contentChanges":[{"text":australian}]})).await;
    let restored = next_diagnostics(&mut notifications, australian_uri.as_str(), 3).await;
    assert!(!has_diagnostic_errors(&restored), "{restored:#?}");
    results.push(restored);
    let overridden = format!("let suffix = LOCAL\n{}", source("locale/shop/fr.lgl"));
    notify(&mut service, "textDocument/didChange", json!({"textDocument":{"uri":french_uri,"version":4},"contentChanges":[{"text":overridden}]})).await;
    let overridden_diagnostics = next_diagnostics(&mut notifications, french_uri.as_str(), 4).await;
    assert!(
        !has_diagnostic_errors(&overridden_diagnostics),
        "{overridden_diagnostics:#?}"
    );
    results.push(overridden_diagnostics);
    notify(&mut service, "textDocument/didOpen", json!({"textDocument":{"uri":english_uri,"languageId":"linguini-locale","version":2,"text":source("locale/shop/en.lgl")}})).await;
    let opened = next_diagnostics(&mut notifications, english_uri.as_str(), 2).await;
    assert!(!has_diagnostic_errors(&opened), "{opened:#?}");
    results.push(opened);
    for document_uri in [&english_uri, &french_uri] {
        let refused = request_response(&mut service, "textDocument/rename", json!({"textDocument":{"uri":document_uri},"position":{"line":0,"character":4},"newName":"renamed"})).await;
        assert!(refused.result().is_none());
        let refused = serde_json::to_value(refused).unwrap();
        assert_eq!(refused["error"]["code"], -32602);
        results.push(refused);
    }
    request(&mut service, "shutdown", json!(null)).await;
    notify(&mut service, "exit", json!(null)).await;
    pump.abort();
    results
}
