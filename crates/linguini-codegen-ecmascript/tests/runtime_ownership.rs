//! Canonical JS templates own runtime syntax; declarations share the semantic model.

use std::path::{Path, PathBuf};

fn source_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("read codegen source directory") {
        let path = entry.expect("read codegen source entry").path();
        if path.is_dir() {
            source_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

#[test]
fn production_runtime_ownership_excludes_paired_sources_and_retired_emitters() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates");
    let owners = [
        "linguini-codegen-ecmascript",
        "linguini-codegen-ecmascript-web",
        "linguini-codegen-svelte",
        "linguini-codegen-sveltekit",
        "linguini-codegen-vite",
    ];
    let mut files = Vec::new();
    assert!(
        !crates
            .join("linguini-codegen-ecmascript/templates/web")
            .exists(),
        "web templates belong to the web integration crate"
    );
    for owner in owners {
        source_files(&crates.join(owner).join("src"), &mut files);
    }
    files.sort();
    for path in files {
        assert!(
            !matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("ts" | "js")
            ),
            "runtime syntax belongs in shared target templates outside Rust source: {}",
            path.display()
        );
        let source = std::fs::read_to_string(&path).expect("read codegen source");
        if path.starts_with(crates.join("linguini-codegen-ecmascript/src")) {
            let production = source.split("#[cfg(test)]").next().unwrap();
            for web_api in ["EcmaWebOptions", "compile_web_", "web_options", "web/"] {
                assert!(
                    !production.contains(web_api),
                    "web generation belongs to its integration crate: {} ({web_api})",
                    path.display()
                );
            }
        }
        if path.parent()
            == Some(
                crates
                    .join("linguini-codegen-ecmascript/src/module")
                    .as_path(),
            )
            && matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some(
                    "message.rs"
                        | "locale.rs"
                        | "globals.rs"
                        | "semantic.rs"
                        | "messages.rs"
                        | "decl.rs"
                        | "signature.rs"
                        | "names.rs"
                        | "emit.rs"
                        | "type_tree.rs"
                )
            )
        {
            let production = source.split("#[cfg(test)]").next().unwrap();
            for inline in [
                "export function message(",
                "export const message =",
                "\"export {};\\n\"",
                "export {{ {} }};",
                "@typedef {{import(",
                "export declare const {",
                "export declare function {",
                "export type {",
                "readonly {}:",
                "@param {{{",
                "@returns {{{",
                "@typedef {{{",
            ] {
                assert!(
                    !production.contains(inline),
                    "module output belongs in shared templates: {} ({inline})",
                    path.display()
                );
            }
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("messages.rs" | "decl.rs")
            ) {
                assert!(
                    !production.contains("fn emit_object_type("),
                    "namespace types must share the provenance-aware renderer: {}",
                    path.display()
                );
            }
        }
        if path.starts_with(crates.join("linguini-codegen-svelte")) {
            let production = source.split("#[cfg(test)]").next().unwrap();
            for framework_api in ["$app/", "App.PageState", "SvelteKit", "sveltekit: bool"] {
                assert!(
                    !production.contains(framework_api),
                    "Svelte host behavior belongs in the framework adapter: {} ({framework_api})",
                    path.display()
                );
            }
        }
        for retired in [
            "pub fn compile_typescript_",
            "pub fn compile_javascript_",
            "pub fn generate_typescript_",
            "fn implementation_rest_params_for_target(",
            "fn normalized_bindings_statement_for_target(",
            "fn javascript_shared_runtime(",
            "fn typescript_index_runtime(",
            "fn javascript_index_runtime(",
            "fn typescript_index(",
            "fn javascript_index(",
            "fn typescript_locale(",
            "fn javascript_locale(",
            "SHARED_RUNTIME",
            "SINGLE_INDEX_RUNTIME",
        ] {
            assert!(
                !source.contains(retired),
                "retired assembly `{retired}` reintroduced in {}",
                path.display()
            );
        }
        if path.starts_with(crates.join("linguini-codegen-ecmascript-web/src")) {
            assert!(
                !source.contains("RuntimeCode"),
                "web runtime bodies must be templates: {}",
                path.display()
            );
            assert!(
                !source.contains("_BODY"),
                "inline web runtime fragments reintroduced: {}",
                path.display()
            );
        }
    }
    let mut templates = Vec::new();
    for owner in owners {
        let directory = crates.join(owner).join("templates");
        if directory.is_dir() {
            source_files(&directory, &mut templates);
        }
    }
    for path in templates {
        let source = std::fs::read_to_string(&path).expect("read codegen template");
        if path.starts_with(crates.join("linguini-codegen-svelte")) {
            for framework_api in ["$app/", "App.PageState", "page.data", "goto(", "sveltekit"] {
                assert!(
                    !source.contains(framework_api),
                    "framework behavior in Svelte template {}: {framework_api}",
                    path.display()
                );
            }
        }
        for branch in ["{% if ts", "{% if not ts", "{% elif ts"] {
            assert!(
                !source.contains(branch),
                "paired TS runtime branch reintroduced: {}",
                path.display()
            );
        }
        assert_eq!(
            path.extension().and_then(|value| value.to_str()),
            Some("jinja"),
            "production templates must share target bodies rather than store generated runtimes: {}",
            path.display()
        );
    }
}
