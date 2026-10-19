//! Lightweight browser controls, emitted from selected web capabilities.

use super::EcmaWebOptions;
use crate::ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport, EcmaStatement,
    RenderedEcmaModule,
};

/// Compile lightweight, capability-selected Svelte browser controls as checked JavaScript.
pub fn compile_svelte_control_module(
    options: &EcmaWebOptions,
    sveltekit: bool,
) -> RenderedEcmaModule {
    control_module(options, sveltekit, EcmaModuleKind::JavaScript).render(&[])
}

fn control_module(options: &EcmaWebOptions, sveltekit: bool, target: EcmaModuleKind) -> EcmaModule {
    let features = options.features();
    let path = "svelte-control.js";
    let mut module = EcmaModule::new(EcmaModuleOutput::new(target, path, None));
    if sveltekit {
        module.push_import(EcmaImport::named(
            "$app/environment",
            vec![EcmaNamedImport::new("browser", "browser")],
        ));
        if features.has_path {
            module.push_import(EcmaImport::named(
                "$app/navigation",
                vec![EcmaNamedImport::new("goto", "goto")],
            ));
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
    if sveltekit && features.has_path {
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
        minijinja::context! { sveltekit, has_path => features.has_path, has_cookie => features.has_cookie, has_local_storage => features.has_local_storage },
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
                let js = compile_svelte_control_module(&options, sveltekit);
                for (selected, capability) in [
                    (options.features().has_path, "window.location"),
                    (options.features().has_cookie, "document.cookie"),
                    (options.features().has_local_storage, "window.localStorage"),
                ] {
                    assert_eq!(js.code.contains(capability), selected);
                }
                assert_eq!(
                    js.code.contains("$app/navigation"),
                    sveltekit && options.features().has_path
                );
                assert_eq!(
                    js.code.contains("clearCurrentLocaleOverride"),
                    sveltekit && options.features().has_path
                );
                assert!(!js.code.contains("./index"));
                let declaration = super::super::test_templates::render_template(
                    &super::super::test_templates::declaration_with_runtime_imports(
                        std::fs::read_to_string(root.join("legacy/svelte-control.runtime.d.ts"))
                            .unwrap(),
                    ),
                    &[(
                        "PAGE_STATE",
                        if sveltekit {
                            "App.PageState"
                        } else {
                            "unknown"
                        }
                        .to_owned(),
                    )],
                );
                assert_eq!(generate_svelte_control_declaration(sveltekit), declaration);
                let snapshots = [
                    (root.join(format!("{name}/svelte-control.js")), js.code),
                    (
                        root.join(format!("{name}/svelte-control.js.map")),
                        js.source_map,
                    ),
                    (
                        root.join(format!("{name}/typescript/svelte-control.d.ts")),
                        generate_svelte_control_declaration(sveltekit),
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
