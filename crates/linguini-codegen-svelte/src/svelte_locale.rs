//! Shared Svelte locale state and deterministic locale-loader ownership.

use linguini_codegen_ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement, RenderedEcmaModule,
};

/// Locale initialization and request-scoped read capabilities.
#[derive(Debug, Clone, Default)]
pub struct SvelteLocaleOptions {
    pub initialize: bool,
    pub request_locale: Option<crate::SvelteHostImport>,
}

pub(super) fn generate_svelte_locale_declaration(options: &SvelteLocaleOptions) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "svelte-locale.svelte.d.ts",
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "./locale.js".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new("Locale", "Locale")]),
    });
    let code = crate::templates::render(
        "svelte/locale_declarations",
        minijinja::context! { initialize => options.initialize, override => options.request_locale.is_some() },
    );
    module.push_statement(EcmaStatement::type_declaration(code, None));
    module.render_code()
}

/// Compile framework-selected Svelte locale state as checked JavaScript.
pub fn compile_svelte_locale_module(options: &SvelteLocaleOptions) -> RenderedEcmaModule {
    compile_locale(options, EcmaModuleKind::JavaScript)
}

fn compile_locale(options: &SvelteLocaleOptions, target: EcmaModuleKind) -> RenderedEcmaModule {
    let path = "svelte-locale.svelte.js";
    locale_module(options, EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn locale_module(options: &SvelteLocaleOptions, output: EcmaModuleOutput) -> EcmaModule {
    let mut module = EcmaModule::new(output);
    if let Some(binding) = &options.request_locale {
        module.push_import(binding.value_import("getRequestLocale"));
    }
    module.push_import(EcmaImport::named_with_types(
        "./locale.js",
        vec![
            EcmaNamedImport::new("baseLocale", "baseLocale"),
            EcmaNamedImport::new("normalizeLocale", "normalizeLocale"),
        ],
        vec![EcmaNamedImport::new("Locale", "Locale")],
    ));
    module.push_statement(EcmaStatement::generated(crate::templates::render(
        "svelte/locale",
        minijinja::context! { request_locale => options.request_locale.is_some(), web => options.initialize },
    )));
    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn svelte_locale_modes_share_loader_and_state_bodies() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-svelte-locale");
        for (name, mode) in [
            ("context", SvelteLocaleOptions::default()),
            (
                "standalone",
                SvelteLocaleOptions {
                    initialize: true,
                    request_locale: None,
                },
            ),
            (
                "sveltekit",
                SvelteLocaleOptions {
                    initialize: true,
                    request_locale: crate::host::test_host().request_locale,
                },
            ),
        ] {
            let js = compile_svelte_locale_module(&mode);
            assert!(!js.code.contains("$state<"));
            assert!(!js.code.contains("type Locale }"));
            let snapshots = [
                (root.join(name).join("svelte-locale.svelte.js"), js.code),
                (
                    root.join(name).join("svelte-locale.svelte.js.map"),
                    js.source_map,
                ),
                (
                    root.join(name).join("typescript/svelte-locale.svelte.d.ts"),
                    generate_svelte_locale_declaration(&mode),
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
