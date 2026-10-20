//! Lightweight browser controls, emitted from selected web capabilities.

use super::EcmaWebOptions;
use linguini_codegen_ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport, EcmaStatement,
    RenderedEcmaModule,
};

/// Compile lightweight, capability-selected Svelte browser controls as checked JavaScript.
pub fn compile_svelte_control_module(
    options: &EcmaWebOptions,
    host: &crate::SvelteHost,
) -> RenderedEcmaModule {
    control_module(options, host, EcmaModuleKind::JavaScript).render(&[])
}

fn control_module(
    options: &EcmaWebOptions,
    host: &crate::SvelteHost,
    target: EcmaModuleKind,
) -> EcmaModule {
    let features = options.features();
    let path = "svelte-control.js";
    let mut module = EcmaModule::new(EcmaModuleOutput::new(target, path, None));
    if let Some(binding) = &host.browser {
        module.push_import(binding.value_import("browser"));
    }
    if features.has_path {
        if let Some(binding) = &host.navigate {
            module.push_import(binding.value_import("navigate"));
        }
    }
    module.push_import(EcmaImport::named(
        "./svelte-effects.svelte.js",
        vec![
            EcmaNamedImport::new("destroyLinguiniEffects", "destroyLinguiniEffects"),
            EcmaNamedImport::new("refreshLinguiniEffects", "refreshLinguiniEffects"),
            EcmaNamedImport::new("web", "web"),
        ],
    ));
    let mut locale_imports = Vec::new();
    if host.navigate.is_some() && host.request_locale.is_some() && features.has_path {
        locale_imports.push(EcmaNamedImport::new(
            "clearCurrentLocaleOverride",
            "clearCurrentLocaleOverride",
        ));
    }
    for name in ["getCurrentLocale", "prepareLocale", "setCurrentLocale"] {
        locale_imports.push(EcmaNamedImport::new(name, name));
    }
    module.push_import(EcmaImport::named(
        "./svelte-locale.svelte.js",
        locale_imports,
    ));
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "svelte/control",
        minijinja::context! { external_browser => host.browser.is_some(), navigation => host.navigate.is_some(), request_locale => host.request_locale.is_some(), has_path => features.has_path, has_cookie => features.has_cookie, has_local_storage => features.has_local_storage },
    )));
    module
}

#[cfg(test)]
mod tests {
    use super::super::{
        svelte_control_types::generate_svelte_control_declaration, EcmaLocaleSource,
    };
    use super::*;
    use std::path::Path;

    #[test]
    fn controls_select_navigation_persistence_and_target_types() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-svelte-control");
        for (name, sources) in [
            ("none", vec![]),
            ("path", vec![EcmaLocaleSource::Path]),
            ("cookie", vec![EcmaLocaleSource::Cookie]),
            ("storage", vec![EcmaLocaleSource::LocalStorage]),
            (
                "all",
                vec![
                    EcmaLocaleSource::Path,
                    EcmaLocaleSource::Cookie,
                    EcmaLocaleSource::LocalStorage,
                    EcmaLocaleSource::AcceptLanguage,
                ],
            ),
        ] {
            let options = EcmaWebOptions {
                sources,
                ..Default::default()
            };
            for sveltekit in [false, true] {
                let name = format!("{}-{name}", if sveltekit { "kit" } else { "plain" });
                let host = if sveltekit {
                    crate::host::test_host()
                } else {
                    Default::default()
                };
                let js = compile_svelte_control_module(&options, &host);
                for (selected, capability) in [
                    (options.features().has_path, "window.location"),
                    (options.features().has_cookie, "document.cookie"),
                    (options.features().has_local_storage, "window.localStorage"),
                ] {
                    assert_eq!(js.code.contains(capability), selected);
                }
                assert_eq!(
                    js.code.contains("navigate } from"),
                    sveltekit && options.features().has_path
                );
                assert_eq!(
                    js.code.contains("clearCurrentLocaleOverride"),
                    sveltekit && options.features().has_path
                );
                assert!(!js.code.contains("./index"));
                let mut declaration = super::super::test_templates::render_template(
                    &super::super::test_templates::declaration_with_runtime_imports(
                        std::fs::read_to_string(root.join("legacy/svelte-control.runtime.d.ts"))
                            .unwrap(),
                    ),
                    &[(
                        "PAGE_STATE",
                        if sveltekit {
                            "NavigationState"
                        } else {
                            "unknown"
                        }
                        .to_owned(),
                    )],
                );
                if sveltekit {
                    declaration = declaration.replacen(
                        "\n\n",
                        "\nimport type { NavigationState } from \"../host.js\";\n\n",
                        1,
                    );
                }
                assert_eq!(generate_svelte_control_declaration(&host), declaration);
                let snapshots = [
                    (root.join(format!("{name}/svelte-control.js")), js.code),
                    (
                        root.join(format!("{name}/svelte-control.js.map")),
                        js.source_map,
                    ),
                    (
                        root.join(format!("{name}/typescript/svelte-control.d.ts")),
                        generate_svelte_control_declaration(&host),
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
    }
}
