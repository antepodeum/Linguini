//! Framework-selected browser effects with closed, guarded capability reads.

use super::{EcmaLinkMode, EcmaWebOptions};
use linguini_codegen_ecmascript::render_web_options as web_options_literal;
use linguini_codegen_ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement, RenderedEcmaModule,
};

pub(super) fn generate_svelte_effects_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "svelte-effects.svelte.d.ts",
        None,
    ));
    for (path, name) in [("./locale.js", "Locale"), ("./web.js", "LinguiniWebLocale")] {
        module.push_import(EcmaImport {
            specifier: path.to_owned(),
            bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(name, name)]),
        });
    }
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render("svelte/effects_declarations", minijinja::context! {}),
        None,
    ));
    module.render_code()
}

/// Compile browser effects with selected framework imports as checked JavaScript.
pub fn compile_svelte_effects_module(
    options: &EcmaWebOptions,
    host: &crate::SvelteHost,
) -> RenderedEcmaModule {
    effects_module(options, host, output(EcmaModuleKind::JavaScript)).render(&[])
}

fn output(target: EcmaModuleKind) -> EcmaModuleOutput {
    EcmaModuleOutput::new(target, "svelte-effects.svelte.js", None)
}

fn effects_module(
    options: &EcmaWebOptions,
    host: &crate::SvelteHost,
    output: EcmaModuleOutput,
) -> EcmaModule {
    let features = options.features();
    let runtime_links = features.link_mode == EcmaLinkMode::Runtime;
    let needs_browser = runtime_links || !features.sources.is_empty();
    let mut module = EcmaModule::new(output);
    if needs_browser {
        if let Some(binding) = &host.browser {
            module.push_import(binding.value_import("browser"));
        }
    }
    if let Some(binding) = &host.base_path {
        module.push_import(binding.value_import("base"));
    }
    module.push_import(EcmaImport {
        specifier: "./locale.js".to_owned(),
        bindings: EcmaImportBindings::Namespace("locale".to_owned()),
    });
    module.push_import(EcmaImport::named(
        "./web.js",
        vec![EcmaNamedImport::new(
            "createWebLocaleI18n",
            "createWebLocaleI18n",
        )],
    ));
    let mut locale_imports = Vec::new();
    if runtime_links {
        locale_imports.push(EcmaNamedImport::new("getCurrentLocale", "getCurrentLocale"));
    }
    locale_imports.push(EcmaNamedImport::new(
        "initializeCurrentLocale",
        "initializeCurrentLocale",
    ));
    module.push_import(EcmaImport::named(
        "./svelte-locale.svelte.js",
        locale_imports,
    ));
    if runtime_links {
        module.push_import(EcmaImport::named(
            "./web/runtime-links.js",
            vec![EcmaNamedImport::new(
                "startRuntimeLinkLocalization",
                "startRuntimeLinkLocalization",
            )],
        ));
    }
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "svelte/effects",
        minijinja::context! {
            options => web_options_literal(options), external_browser => host.browser.is_some(), base_path => host.base_path.is_some(),
            runtime_links, needs_browser,
            has_path => features.has_path, has_cookie => features.has_cookie,
            has_local_storage => features.has_local_storage, has_accept_language => features.has_accept_language,
            no_sources => features.sources.is_empty(),
        },
    )));
    module
}

#[cfg(test)]
mod tests {
    use super::super::EcmaLocaleSource;
    use super::*;
    use std::path::Path;

    #[test]
    fn effects_select_only_owned_capabilities_and_target_types() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-svelte-effects");
        let cases = [
            ("none", vec![], EcmaLinkMode::Manual),
            (
                "path",
                vec![EcmaLocaleSource::Path],
                EcmaLinkMode::Transform,
            ),
            (
                "cookie",
                vec![EcmaLocaleSource::Cookie],
                EcmaLinkMode::Manual,
            ),
            (
                "storage",
                vec![EcmaLocaleSource::LocalStorage],
                EcmaLinkMode::Manual,
            ),
            (
                "language",
                vec![EcmaLocaleSource::AcceptLanguage],
                EcmaLinkMode::Manual,
            ),
            (
                "all",
                vec![
                    EcmaLocaleSource::Path,
                    EcmaLocaleSource::Cookie,
                    EcmaLocaleSource::LocalStorage,
                    EcmaLocaleSource::AcceptLanguage,
                ],
                EcmaLinkMode::Runtime,
            ),
        ];
        for (name, sources, link_mode) in cases {
            let options = EcmaWebOptions {
                sources,
                link_mode,
                ..Default::default()
            };
            for sveltekit in [false, true] {
                let name = format!("{}-{name}", if sveltekit { "kit" } else { "plain" });
                let host = if sveltekit {
                    crate::host::test_host()
                } else {
                    Default::default()
                };
                let js = compile_svelte_effects_module(&options, &host);
                assert_eq!(
                    generate_svelte_effects_declaration(),
                    super::super::test_templates::declaration_with_runtime_imports(
                        std::fs::read_to_string(root.join("legacy/svelte-effects.runtime.d.ts"))
                            .unwrap()
                    )
                );
                for (selected, capability) in [
                    (options.features().has_path, "window.location"),
                    (options.features().has_cookie, "document.cookie"),
                    (options.features().has_local_storage, "window.localStorage"),
                    (options.features().has_accept_language, "window.navigator"),
                ] {
                    assert_eq!(js.code.contains(capability), selected);
                }
                assert_eq!(
                    js.code.contains("startRuntimeLinkLocalization"),
                    link_mode == EcmaLinkMode::Runtime
                );
                let runtime_links = link_mode == EcmaLinkMode::Runtime;
                let has_sources = !options.sources.is_empty();
                assert_eq!(js.code.contains("readInitialLocale"), has_sources);
                assert_eq!(js.code.contains("readBrowserCapability"), has_sources);
                assert_eq!(js.code.contains("getCurrentLocale"), runtime_links);
                assert_eq!(js.code.contains("const linkEffects"), runtime_links);
                assert_eq!(js.code.contains("hot?.dispose("), runtime_links);
                assert_eq!(
                    js.code.contains("const browser") || js.code.contains("{ browser }"),
                    runtime_links || has_sources
                );
                assert!(!js.code.contains("const linkEffects = undefined"));
                assert!(!js.code.contains(" as const"));
                let snapshots = [
                    (
                        root.join(format!("{name}/svelte-effects.svelte.js")),
                        js.code,
                    ),
                    (
                        root.join(format!("{name}/svelte-effects.svelte.js.map")),
                        js.source_map,
                    ),
                    (
                        root.join(format!("{name}/typescript/svelte-effects.svelte.d.ts")),
                        generate_svelte_effects_declaration(),
                    ),
                ];
                if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
                    for (path, code) in &snapshots {
                        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                        std::fs::write(path, code).unwrap();
                    }
                }
                for (path, code) in snapshots {
                    assert_eq!(code, std::fs::read_to_string(path).unwrap());
                }
            }
        }
        // Runtime links need a browser guard even when locale resolution has no sources.
        for link_mode in [
            EcmaLinkMode::Manual,
            EcmaLinkMode::Transform,
            EcmaLinkMode::Runtime,
        ] {
            let options = EcmaWebOptions {
                sources: vec![],
                link_mode,
                ..Default::default()
            };
            for host in [crate::SvelteHost::default(), crate::host::test_host()] {
                let js = compile_svelte_effects_module(&options, &host);
                assert!(js.code.contains("initializeCurrentLocale(web.baseLocale)"));
                assert!(!js.code.contains("readInitialLocale"));
                assert!(!js.code.contains("readBrowserCapability"));
                assert_eq!(
                    js.code.contains("startRuntimeLinkLocalization"),
                    link_mode == EcmaLinkMode::Runtime
                );
                assert_eq!(
                    js.code.contains("const browser") || js.code.contains("{ browser }"),
                    link_mode == EcmaLinkMode::Runtime
                );
            }
        }
    }
}
