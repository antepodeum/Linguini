//! Framework-selected browser effects with closed, guarded capability reads.

use super::runtime_code::RuntimeCode;
use super::web_options::web_options_literal;
use super::{TypeScriptLinkMode, TypeScriptWebOptions};
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

pub(super) fn generate_typescript_svelte_effects_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> String {
    effects_module(options, sveltekit, output(EcmaScriptTarget::TypeScript)).render_code()
}

pub(super) fn generate_svelte_effects_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "svelte-effects.svelte.d.ts",
        None,
    ));
    for (path, name) in [("./locale", "Locale"), ("./web", "LinguiniWebLocale")] {
        module.push_import(EcmaImport {
            specifier: path.to_owned(),
            bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(name, name)]),
        });
    }
    module.push_statement(EcmaStatement::type_declaration("export declare const web: LinguiniWebLocale<Locale>;\nexport declare function refreshLinguiniEffects(): void;\nexport declare function destroyLinguiniEffects(): void;", None));
    module.render_code()
}

/// Compile browser effects with selected framework imports as TypeScript.
pub fn compile_typescript_svelte_effects_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> RenderedEcmaModule {
    effects_module(options, sveltekit, output(EcmaScriptTarget::TypeScript)).render(&[])
}

/// Compile browser effects with selected framework imports as checked JavaScript.
pub fn compile_javascript_svelte_effects_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> RenderedEcmaModule {
    effects_module(options, sveltekit, output(EcmaScriptTarget::JavaScript)).render(&[])
}

fn output(target: EcmaScriptTarget) -> EcmaModuleOutput {
    EcmaModuleOutput::new(
        target,
        if target.is_typescript() {
            "svelte-effects.svelte.ts"
        } else {
            "svelte-effects.svelte.js"
        },
        None,
    )
}

fn effects_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
    output: EcmaModuleOutput,
) -> EcmaModule {
    let target = output.target();
    let features = options.features();
    let mut module = EcmaModule::new(output);
    if sveltekit {
        for (path, name) in [("$app/environment", "browser"), ("$app/paths", "base")] {
            module.push_import(EcmaImport::named(
                path,
                vec![EcmaNamedImport::new(name, name)],
            ));
        }
    }
    module.push_import(EcmaImport {
        specifier: if target.is_typescript() {
            "./locale"
        } else {
            "./locale.js"
        }
        .to_owned(),
        bindings: EcmaImportBindings::Namespace("locale".to_owned()),
    });
    module.push_import(EcmaImport::named(
        if target.is_typescript() {
            "./web"
        } else {
            "./web.js"
        },
        vec![EcmaNamedImport::new(
            "createWebLocaleI18n",
            "createWebLocaleI18n",
        )],
    ));
    module.push_import(EcmaImport::named(
        "./svelte-locale.svelte.js",
        vec![
            EcmaNamedImport::new("getCurrentLocale", "getCurrentLocale"),
            EcmaNamedImport::new("initializeCurrentLocale", "initializeCurrentLocale"),
        ],
    ));
    if features.link_mode == TypeScriptLinkMode::Runtime {
        module.push_import(EcmaImport::named(
            "./web/runtime-links.js",
            vec![EcmaNamedImport::new(
                "startRuntimeLinkLocalization",
                "startRuntimeLinkLocalization",
            )],
        ));
    }
    let mut code = String::new();
    if !sveltekit {
        code.push_str("const browser = typeof window !== \"undefined\" && typeof document !== \"undefined\";\n\n");
    }
    let annotation = if target.is_typescript() {
        ""
    } else {
        "/** @type {import(\"./web.js\").LinguiniWebLocale<import(\"./locale.js\").Locale>} */\n"
    };
    code.push_str(&format!("{annotation}export const web = createWebLocaleI18n(locale, {}, {});\n\ninitializeCurrentLocale(readInitialLocale());\n\n", web_options_literal(options, target), if sveltekit { "{ base }" } else { "{}" }));
    let mut body = RuntimeCode::default();
    if features.link_mode == TypeScriptLinkMode::Runtime {
        body.shared("const linkEffects = browser\n  ? startRuntimeLinkLocalization(web, getCurrentLocale)\n  : undefined;\n\n");
    } else {
        body.typed("const linkEffects: { refresh(): void; destroy(): void } | undefined = undefined;\n\n", "/** @type {{ refresh(): void; destroy(): void } | undefined} */\nconst linkEffects = undefined;\n\n");
    }
    body.typed(
        "export function refreshLinguiniEffects(): void {\n",
        "/** @returns {void} */\nexport function refreshLinguiniEffects() {\n",
    );
    body.shared("  linkEffects?.refresh();\n}\n\n");
    body.typed(
        "export function destroyLinguiniEffects(): void {\n",
        "/** @returns {void} */\nexport function destroyLinguiniEffects() {\n",
    );
    body.shared("  linkEffects?.destroy();\n}\n\n");
    body.typed("const hot = (import.meta as ImportMeta & {\n  hot?: { dispose(callback: () => void): void };\n}).hot;\n", "const hot = (/** @type {ImportMeta & { hot?: { dispose(callback: () => void): void } }} */ (import.meta)).hot;\n");
    body.shared("hot?.dispose(destroyLinguiniEffects);\n\nfunction readInitialLocale() {\n  if (!browser) return web.baseLocale;\n  return web.resolveLocaleSync({\n");
    if features.has_path {
        body.shared("    url: readBrowserCapability(() => window.location.href),\n");
    }
    if features.has_cookie {
        body.shared("    cookie: readBrowserCapability(() => document.cookie),\n");
    }
    if features.has_local_storage {
        body.shared("    localStorage: readBrowserCapability(() => window.localStorage),\n");
    }
    if features.has_accept_language {
        body.shared("    navigator: readBrowserCapability(() => window.navigator),\n");
    }
    if features.sources.is_empty() {
        body.shared("\n");
    }
    body.shared("  });\n}\n\n");
    body.typed("function readBrowserCapability<T>(read: () => T): T | undefined {\n", "/** @template T @param {() => T} read @returns {T | undefined} */\nfunction readBrowserCapability(read) {\n");
    body.shared("  try {\n    return read();\n  } catch {\n    return undefined;\n  }\n}\n");
    code.push_str(&body.render(target));
    module.push_statement(EcmaStatement::generated(code));
    module
}

#[cfg(test)]
mod tests {
    use super::super::TypeScriptLocaleSource;
    use super::*;
    use std::path::Path;

    #[test]
    fn effects_select_only_owned_capabilities_and_target_types() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-svelte-effects");
        let cases = [
            ("none", vec![], TypeScriptLinkMode::Manual),
            (
                "path",
                vec![TypeScriptLocaleSource::Path],
                TypeScriptLinkMode::Transform,
            ),
            (
                "cookie",
                vec![TypeScriptLocaleSource::Cookie],
                TypeScriptLinkMode::Manual,
            ),
            (
                "storage",
                vec![TypeScriptLocaleSource::LocalStorage],
                TypeScriptLinkMode::Manual,
            ),
            (
                "language",
                vec![TypeScriptLocaleSource::AcceptLanguage],
                TypeScriptLinkMode::Manual,
            ),
            (
                "all",
                vec![
                    TypeScriptLocaleSource::Path,
                    TypeScriptLocaleSource::Cookie,
                    TypeScriptLocaleSource::LocalStorage,
                    TypeScriptLocaleSource::AcceptLanguage,
                ],
                TypeScriptLinkMode::Runtime,
            ),
        ];
        for (name, sources, link_mode) in cases {
            let options = TypeScriptWebOptions {
                sources,
                link_mode,
                ..Default::default()
            };
            for sveltekit in [false, true] {
                let name = format!("{}-{name}", if sveltekit { "kit" } else { "plain" });
                let ts = compile_typescript_svelte_effects_module(&options, sveltekit);
                let js = compile_javascript_svelte_effects_module(&options, sveltekit);
                let legacy =
                    std::fs::read_to_string(root.join("legacy/svelte-effects.runtime.ts")).unwrap();
                let rendered = super::super::templates::render_template(&legacy, &[
                    ("BROWSER_RUNTIME", String::new()),
                    ("OPTIONS", web_options_literal(&options, EcmaScriptTarget::TypeScript)),
                    ("ENVIRONMENT", if sveltekit { "{ base }" } else { "{}" }.to_owned()),
                    ("LINK_RUNTIME_IMPORT", String::new()),
                    ("LINK_RUNTIME_START", if link_mode == TypeScriptLinkMode::Runtime {
                        "const linkEffects = browser\n  ? startRuntimeLinkLocalization(web, getCurrentLocale)\n  : undefined;"
                    } else { "const linkEffects: { refresh(): void; destroy(): void } | undefined = undefined;" }.to_owned()),
                ]);
                let legacy_body = rendered
                    .lines()
                    .filter(|line| {
                        ![
                            (options.features().has_path, "window.location"),
                            (options.features().has_cookie, "document.cookie"),
                            (options.features().has_local_storage, "window.localStorage"),
                            (options.features().has_accept_language, "window.navigator"),
                        ]
                        .iter()
                        .any(|(selected, capability)| !selected && line.contains(capability))
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let body = |code: &str| {
                    code[code.find("export const web").unwrap()..]
                        .lines()
                        .filter(|line| {
                            !line.trim().is_empty() && !line.starts_with("//# sourceMappingURL=")
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                assert_eq!(
                    body(&ts.code),
                    body(&legacy_body),
                    "only import layout may change in {name}"
                );
                assert_eq!(
                    generate_svelte_effects_declaration(),
                    std::fs::read_to_string(root.join("legacy/svelte-effects.runtime.d.ts"))
                        .unwrap()
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
                    link_mode == TypeScriptLinkMode::Runtime
                );
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
                        root.join(format!("{name}/typescript/svelte-effects.svelte.ts")),
                        ts.code,
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
    }
}
