//! Svelte message-provider facade, with optional lightweight web controls.

use linguini_codegen_ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement, RenderedEcmaModule,
};

/// Compile the context-only or web-enabled Svelte message facade as checked JavaScript.
pub fn compile_svelte_module(web: bool) -> RenderedEcmaModule {
    facade_module(web, EcmaModuleKind::JavaScript).render(&[])
}

fn facade_module(web: bool, target: EcmaModuleKind) -> EcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(target, "svelte.js", None));
    module.push_import(EcmaImport {
        specifier: "./index.js".to_owned(),
        bindings: EcmaImportBindings::Namespace("runtime".to_owned()),
    });
    if web {
        module.push_import(EcmaImport::named(
            "./svelte-control.js",
            vec![EcmaNamedImport::new("linguini", "controls")],
        ));
        module.push_import(EcmaImport::named(
            "./svelte-locale.svelte.js",
            vec![EcmaNamedImport::new(
                "registerLocaleLoader",
                "registerLocaleLoader",
            )],
        ));
    } else {
        module.push_import(EcmaImport::named(
            "./svelte-locale.svelte.js",
            ["getCurrentLocale", "prepareLocale", "setCurrentLocale"]
                .into_iter()
                .map(|name| EcmaNamedImport::new(name, name))
                .collect(),
        ));
    }
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "svelte/facade",
        minijinja::context! { web },
    )));
    module
}

#[cfg(test)]
mod tests {
    use super::super::svelte_facade_types::generate_svelte_declaration;
    use super::*;
    use std::path::Path;

    #[test]
    fn svelte_facades_share_providers_getters_and_hmr_lifecycle() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-svelte-facade");
        for (name, web) in [("context", false), ("web", true)] {
            let js = compile_svelte_module(web);
            assert_eq!(js.code.contains("registerLocaleLoader"), web);
            assert_eq!(js.code.contains("controls.destroy"), web);
            assert!(!js.code.contains("MutationObserver"));
            assert_eq!(
                generate_svelte_declaration(web),
                super::super::test_templates::declaration_with_runtime_imports(
                    std::fs::read_to_string(
                        root.join(format!("{name}/legacy/svelte.runtime.d.ts"))
                    )
                    .unwrap()
                )
            );
            let mut snapshots = vec![
                (root.join(format!("{name}/svelte.js")), js.code),
                (root.join(format!("{name}/svelte.js.map")), js.source_map),
                (
                    root.join(format!("{name}/typescript/svelte.d.ts")),
                    generate_svelte_declaration(web),
                ),
            ];
            if web {
                snapshots.push((
                    root.join("web/svelte-control.d.ts"),
                    super::super::svelte_control_types::generate_svelte_control_declaration(
                        &Default::default(),
                    ),
                ));
            }
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
