//! Shared Svelte locale state and deterministic locale-loader ownership.

use super::runtime_code::RuntimeCode;
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
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

pub(super) fn generate_typescript_svelte_locale_module(mode: SvelteLocaleMode) -> String {
    locale_module(
        mode,
        EcmaModuleOutput::new(
            EcmaScriptTarget::TypeScript,
            "svelte-locale.svelte.ts",
            Some("svelte-locale.svelte.d.ts".to_owned()),
        ),
    )
    .render_code()
}

pub(super) fn generate_svelte_locale_declaration(mode: SvelteLocaleMode) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "svelte-locale.svelte.d.ts",
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "./locale".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new("Locale", "Locale")]),
    });
    let mut code = String::from("export type LinguiniLocaleLoader = (locale: Locale) => void | Promise<void>;\n\nexport declare function registerLocaleLoader(loader: LinguiniLocaleLoader): () => void;\nexport declare function prepareLocale(locale: unknown): Promise<Locale>;\nexport declare function getCurrentLocale(): Locale;\n");
    if mode != SvelteLocaleMode::Context {
        code.push_str(
            "export declare function initializeCurrentLocale(locale: unknown): Locale;\n",
        );
    }
    code.push_str("export declare function setCurrentLocale(locale: unknown): Locale;\n");
    if mode == SvelteLocaleMode::SvelteKit {
        code.push_str("export declare function clearCurrentLocaleOverride(): void;\n");
    }
    module.push_statement(EcmaStatement::type_declaration(code, None));
    module.render_code()
}

/// Compile framework-selected Svelte locale state as TypeScript.
pub fn compile_typescript_svelte_locale_module(mode: SvelteLocaleMode) -> RenderedEcmaModule {
    compile_locale(mode, EcmaScriptTarget::TypeScript)
}

/// Compile framework-selected Svelte locale state as checked JavaScript.
pub fn compile_javascript_svelte_locale_module(mode: SvelteLocaleMode) -> RenderedEcmaModule {
    compile_locale(mode, EcmaScriptTarget::JavaScript)
}

fn compile_locale(mode: SvelteLocaleMode, target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = if target.is_typescript() {
        "svelte-locale.svelte.ts"
    } else {
        "svelte-locale.svelte.js"
    };
    locale_module(mode, EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn locale_module(mode: SvelteLocaleMode, output: EcmaModuleOutput) -> EcmaModule {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    if mode == SvelteLocaleMode::SvelteKit {
        module.push_import(EcmaImport::named(
            "$app/state",
            vec![EcmaNamedImport::new("page", "page")],
        ));
    }
    module.push_import(EcmaImport::named_with_types(
        if target.is_typescript() {
            "./locale"
        } else {
            "./locale.js"
        },
        vec![
            EcmaNamedImport::new("baseLocale", "baseLocale"),
            EcmaNamedImport::new("normalizeLocale", "normalizeLocale"),
        ],
        vec![EcmaNamedImport::new("Locale", "Locale")],
    ));
    let mut code = RuntimeCode::default();
    code.typed(
        "export type LinguiniLocaleLoader = (locale: Locale) => void | Promise<void>;\n\nconst localeLoaders = new Set<LinguiniLocaleLoader>();\n\n",
        "/** @typedef {import(\"./locale.js\").Locale} Locale */\n/** @typedef {(locale: Locale) => void | Promise<void>} LinguiniLocaleLoader */\n\n/** @type {Set<LinguiniLocaleLoader>} */ const localeLoaders = new Set();\n\n",
    );
    if mode == SvelteLocaleMode::SvelteKit {
        code.typed(
            "let clientLocale = $state<Locale>(baseLocale);\n",
            "/** @type {Locale} */ let clientLocale = $state(baseLocale);\n",
        );
        code.shared("let hasClientOverride = $state(false);\n\n");
        code.shared(
            r#"/**
 * Register a locale message loader and return its deterministic disposer.
 *
 * The disposer is idempotent, which lets HMR consumers safely replace a loader
 * without retaining callbacks from a previous module instance.
 */
"#,
        );
    } else {
        code.typed(
            "let activeLocale = $state<Locale>(baseLocale);\n\n",
            "/** @type {Locale} */ let activeLocale = $state(baseLocale);\n\n",
        );
    }
    code.typed(
        "export function registerLocaleLoader(loader: LinguiniLocaleLoader): () => void {\n",
        "/** @param {LinguiniLocaleLoader} loader @returns {() => void} */\nexport function registerLocaleLoader(loader) {\n",
    );
    code.shared(
        r#"  localeLoaders.add(loader);
  let disposed = false;
  return () => {
    if (disposed) return;
    disposed = true;
    localeLoaders.delete(loader);
  };
}

"#,
    );
    if mode == SvelteLocaleMode::SvelteKit {
        code.shared(
            r#"/**
 * Ensure all currently registered consumers have loaded a locale's messages.
 *
 * This intentionally snapshots the registry before starting work: a loader
 * may register or dispose another loader while its promise is pending, but
 * that mutation must not alter the current preparation operation.
 */
"#,
        );
    }
    code.typed(
        "export async function prepareLocale(locale: unknown): Promise<Locale> {\n",
        "/** @param {unknown} locale @returns {Promise<Locale>} */\nexport async function prepareLocale(locale) {\n",
    );
    code.shared(
        r#"  const resolved = normalizeLocale(locale) ?? baseLocale;
  const loaders = [...localeLoaders];
  await Promise.all(loaders.map((loader) => loader(resolved)));
  return resolved;
}

"#,
    );
    code.typed(
        "export function getCurrentLocale(): Locale {\n",
        "/** @returns {Locale} */\nexport function getCurrentLocale() {\n",
    );
    if mode == SvelteLocaleMode::SvelteKit {
        code.shared(
            r#"  const dataLocale = page.data?.linguini?.locale;
  const resolvedClientLocale = normalizeLocale(clientLocale) ?? baseLocale;
  return hasClientOverride
    ? resolvedClientLocale
    : normalizeLocale(dataLocale) ?? resolvedClientLocale;
}

"#,
        );
    } else {
        code.shared("  return activeLocale;\n}\n\n");
    }
    if mode != SvelteLocaleMode::Context {
        code.typed("export function initializeCurrentLocale(locale: unknown): Locale {\n", "/** @param {unknown} locale @returns {Locale} */\nexport function initializeCurrentLocale(locale) {\n");
        if mode == SvelteLocaleMode::SvelteKit {
            code.shared("  clientLocale = normalizeLocale(locale) ?? baseLocale;\n  hasClientOverride = false;\n  return clientLocale;\n}\n\n");
        } else {
            code.shared("  activeLocale = normalizeLocale(locale) ?? baseLocale;\n  return activeLocale;\n}\n\n");
        }
    }
    code.typed("export function setCurrentLocale(locale: unknown): Locale {\n", "/** @param {unknown} locale @returns {Locale} */\nexport function setCurrentLocale(locale) {\n");
    if mode == SvelteLocaleMode::SvelteKit {
        code.shared("  clientLocale = normalizeLocale(locale) ?? baseLocale;\n  hasClientOverride = true;\n  return clientLocale;\n}\n\n");
        code.typed(
            "export function clearCurrentLocaleOverride(): void {\n",
            "/** @returns {void} */\nexport function clearCurrentLocaleOverride() {\n",
        );
        code.shared("  hasClientOverride = false;\n}\n");
    } else {
        code.shared(
            "  activeLocale = normalizeLocale(locale) ?? baseLocale;\n  return activeLocale;\n}\n",
        );
    }
    module.push_statement(EcmaStatement::generated(code.render(target)));
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
            let ts = compile_typescript_svelte_locale_module(mode);
            let js = compile_javascript_svelte_locale_module(mode);
            assert_eq!(
                ts.code
                    .strip_suffix("//# sourceMappingURL=svelte-locale.svelte.ts.map\n")
                    .unwrap(),
                generate_typescript_svelte_locale_module(mode)
            );
            assert!(!js.code.contains("$state<"));
            assert!(!js.code.contains("type Locale }"));
            let snapshots = [
                (root.join(name).join("svelte-locale.svelte.js"), js.code),
                (
                    root.join(name).join("svelte-locale.svelte.js.map"),
                    js.source_map,
                ),
                (
                    root.join(name).join("typescript/svelte-locale.svelte.ts"),
                    ts.code,
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
