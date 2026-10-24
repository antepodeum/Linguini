//! Project-specific Svelte import and type bindings. Runtime behavior belongs to npm adapters.
mod svelte_facade;
mod svelte_locale;
mod templates;
pub use svelte_facade::compile_svelte_module;
pub use svelte_locale::compile_svelte_locale_module;

pub fn runtime_helper_descriptors() -> std::collections::BTreeMap<String, serde_json::Value> {
    std::collections::BTreeMap::from([(
        "svelte_locale".to_owned(),
        serde_json::json!({"import":"./svelte-locale.svelte.js", "file":"svelte-locale.svelte.js"}),
    )])
}
pub struct SvelteProjectOptions {
    pub declaration: bool,
    pub host_module: Option<String>,
}
fn runtime_module(host: Option<&str>) -> &str {
    host.unwrap_or("@antepodeum/linguini-svelte")
}
pub fn generate_project_files(
    options: &SvelteProjectOptions,
    source_maps: bool,
) -> Vec<linguini_codegen_ecmascript::EcmaGeneratedFile> {
    use linguini_codegen_ecmascript::{generated_module_files, EcmaGeneratedFile};
    let host = options.host_module.as_deref();
    let mut files = generated_module_files(compile_svelte_locale_module(host), source_maps);
    files.extend(generated_module_files(
        compile_svelte_module(host),
        source_maps,
    ));
    if options.declaration {
        files.push(EcmaGeneratedFile {
            path: "svelte-locale.svelte.d.ts".into(),
            contents: svelte_locale::declaration(host),
        });
        files.push(EcmaGeneratedFile {
            path: "svelte.d.ts".into(),
            contents: svelte_facade::declaration(host),
        });
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn project_bindings_only_import_runtime_implementations_and_preserve_artifact_ownership() {
        for host in [None, Some("@antepodeum/linguini-sveltekit/client")] {
            for declaration in [false, true] {
                for source_maps in [false, true] {
                    let options = SvelteProjectOptions {
                        declaration,
                        host_module: host.map(str::to_owned),
                    };
                    let files = generate_project_files(&options, source_maps);
                    let files: BTreeMap<_, _> = files
                        .iter()
                        .map(|file| (file.path.as_str(), file.contents.as_str()))
                        .collect();
                    assert_eq!(
                        files.len(),
                        2 + usize::from(source_maps) * 2 + usize::from(declaration) * 2
                    );
                    for name in ["svelte-locale.svelte.js", "svelte.js"] {
                        let code = files[name];
                        assert!(code.contains(crate::runtime_module(host)));
                        for implementation in [
                            "$state(",
                            "function createLocaleState",
                            "function createLinguiniRune",
                            "createWebLocaleI18n",
                            "new Response",
                            "document.cookie",
                            "window.location",
                        ] {
                            assert!(
                                !code.contains(implementation),
                                "unexpected runtime body in {name}: {implementation}"
                            );
                        }
                        if source_maps {
                            let map: serde_json::Value =
                                serde_json::from_str(files[format!("{name}.map").as_str()])
                                    .unwrap();
                            assert_eq!(map["file"], name);
                            assert!(code.ends_with(&format!("//# sourceMappingURL={name}.map\n")));
                        }
                    }
                    assert_eq!(
                        files["svelte-locale.svelte.js"].contains("{ getRequestLocale }"),
                        host.is_some()
                    );
                    if declaration {
                        assert!(files["svelte.d.ts"].contains("LinguiniRune<Locale, Linguini>"));
                        assert!(files["svelte-locale.svelte.d.ts"].contains("LocaleState<Locale>"));
                    }
                }
            }
        }
    }
}
