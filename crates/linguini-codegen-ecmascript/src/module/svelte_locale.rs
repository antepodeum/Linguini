//! Shared Svelte locale state and deterministic locale-loader ownership.

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaStatement, RenderedEcmaModule,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvelteLocaleMode {
    Context,
    Standalone,
    SvelteKit,
}

impl SvelteLocaleMode {
    pub(super) fn from_framework(web: bool, sveltekit: bool) -> Self {
        if !web {
            Self::Context
        } else if sveltekit {
            Self::SvelteKit
        } else {
            Self::Standalone
        }
    }
}

pub(super) fn generate_svelte_locale_declaration(mode: SvelteLocaleMode) -> String {
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
        minijinja::context! { initialize => mode != SvelteLocaleMode::Context, override => mode == SvelteLocaleMode::SvelteKit },
    );
    module.push_statement(EcmaStatement::type_declaration(code, None));
    module.render_code()
}

/// Compile framework-selected Svelte locale state as checked JavaScript.
pub fn compile_svelte_locale_module(mode: SvelteLocaleMode) -> RenderedEcmaModule {
    compile_locale(mode, EcmaModuleKind::JavaScript)
}

fn compile_locale(mode: SvelteLocaleMode, target: EcmaModuleKind) -> RenderedEcmaModule {
    let path = "svelte-locale.svelte.js";
    locale_module(mode, EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn locale_module(mode: SvelteLocaleMode, output: EcmaModuleOutput) -> EcmaModule {
    let mut module = EcmaModule::new(output);
    if mode == SvelteLocaleMode::SvelteKit {
        module.push_import(EcmaImport::named(
            "$app/state",
            vec![EcmaNamedImport::new("page", "page")],
        ));
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
        minijinja::context! { sveltekit => mode == SvelteLocaleMode::SvelteKit, web => mode != SvelteLocaleMode::Context },
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
            ("context", SvelteLocaleMode::Context),
            ("standalone", SvelteLocaleMode::Standalone),
            ("sveltekit", SvelteLocaleMode::SvelteKit),
        ] {
            let js = compile_svelte_locale_module(mode);
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
                    generate_svelte_locale_declaration(mode),
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
