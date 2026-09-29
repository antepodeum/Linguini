//! Lightweight browser controls, emitted from selected web capabilities.

use super::runtime_code::RuntimeCode;
use super::TypeScriptWebOptions;
use crate::ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaScriptTarget, EcmaStatement,
    RenderedEcmaModule,
};

pub(super) fn generate_typescript_svelte_control_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> String {
    control_module(options, sveltekit, EcmaScriptTarget::TypeScript).render_code()
}

/// Compile lightweight, capability-selected Svelte browser controls as TypeScript.
pub fn compile_typescript_svelte_control_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> RenderedEcmaModule {
    control_module(options, sveltekit, EcmaScriptTarget::TypeScript).render(&[])
}

/// Compile lightweight, capability-selected Svelte browser controls as checked JavaScript.
pub fn compile_javascript_svelte_control_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> RenderedEcmaModule {
    control_module(options, sveltekit, EcmaScriptTarget::JavaScript).render(&[])
}

fn control_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
    target: EcmaScriptTarget,
) -> EcmaModule {
    let features = options.features();
    let path = if target.is_typescript() {
        "svelte-control.ts"
    } else {
        "svelte-control.js"
    };
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
    let mut code = RuntimeCode::default();
    code.typed("", "/** @typedef {import(\"./locale.js\").Locale} Locale */\n/** @typedef {import(\"./web.js\").LinkLocalizationAttributes} LinkLocalizationAttributes */\n\n");
    if !sveltekit {
        code.shared("const browser = typeof window !== \"undefined\" && typeof document !== \"undefined\";\n\n");
    }
    code.shared(
        r#"export const linguini = createLinguiniControl();
export const setLocale = linguini.setLocale;
export const localizeHref = linguini.localizeHref;
export const localizeUrl = linguini.localizeUrl;
export const shouldLocalizeHref = linguini.shouldLocalizeHref;
export const shouldLocalizeLink = linguini.shouldLocalizeLink;
export const localizeHrefAttribute = linguini.localizeHrefAttribute;
export const delocalizeUrl = linguini.delocalizeUrl;
export const alternateLinks = linguini.alternateLinks;
export const destroy = linguini.destroy;

function createLinguiniControl() {
"#,
    );
    code.typed("  async function setLocale(nextLocale: string, setOptions: Record<string, unknown> = {}) {\n", "  /** @param {string} nextLocale @param {Record<string, unknown>} [setOptions] @returns {Promise<Locale>} */\n  async function setLocale(nextLocale, setOptions = {}) {\n");
    code.shared("    const resolved = await prepareLocale(nextLocale);\n");
    code.typed(r#"    const options: Record<string, unknown> & {
      cookie: boolean;
      navigate: boolean;
      replaceState: boolean;
      invalidateAll: boolean;
      keepFocus: boolean;
      noScroll: boolean;
    } = {
"#, "    /** @type {Record<string, unknown> & {cookie: boolean; navigate: boolean; replaceState: boolean; invalidateAll: boolean; keepFocus: boolean; noScroll: boolean}} */\n    const options = {\n");
    code.shared(
        r#"      cookie: true,
      navigate: true,
      replaceState: false,
      invalidateAll: true,
      keepFocus: true,
      noScroll: true,
      ...setOptions,
    };
    if (browser) {
      setCurrentLocale(resolved);
"#,
    );
    if features.has_local_storage {
        code.shared("      if (web.options.locale.switch.writesLocalStorage) {\n        writeLocalStorage(web, resolved);\n      }\n");
    }
    if features.has_cookie {
        code.shared("      if (options.cookie && web.options.locale.switch.writesCookie) {\n        writeLocaleCookie(web, resolved);\n      }\n");
    }
    if features.has_path {
        code.shared("      if (options.navigate && web.options.locale.switch.writesPath) {\n        const href = web.localizeHref(window.location.href, resolved);\n");
        if sveltekit {
            code.shared("        await goto(href, {\n          replaceState: Boolean(options.replaceState),\n          invalidateAll: Boolean(options.invalidateAll),\n");
            code.typed("          keepFocus: options.keepFocus as boolean | undefined,\n          noScroll: options.noScroll as boolean | undefined,\n          state: options.state as App.PageState | undefined,\n", "          keepFocus: options.keepFocus,\n          noScroll: options.noScroll,\n          state: /** @type {App.PageState | undefined} */ (options.state),\n");
            code.shared("        });\n        clearCurrentLocaleOverride();\n");
        } else {
            code.shared("        if (options.replaceState) {\n          window.location.replace(href);\n        } else {\n          window.location.assign(href);\n        }\n");
        }
        code.shared("      }\n");
    }
    code.shared(
        r#"      refreshLinguiniEffects();
    }

    return resolved;
  }

  return {
    get locale() {
      return getCurrentLocale();
    },
    get lang() {
      return getCurrentLocale();
    },
    get direction() {
      return web.getTextDirection(getCurrentLocale());
    },
    get textDirection() {
      return web.getTextDirection(getCurrentLocale());
    },
    get htmlAttrs() {
      return web.htmlAttrs(getCurrentLocale());
    },
    setLocale,
"#,
    );
    forwarding_methods(&mut code);
    code.shared("    destroy: destroyLinguiniEffects,\n  };\n}\n");
    if features.has_local_storage {
        code.shared("\n");
        code.typed("function writeLocalStorage(web: typeof import(\"./svelte-effects.svelte.js\").web, locale: string) {\n", "/** @param {typeof import(\"./svelte-effects.svelte.js\").web} web @param {string} locale */\nfunction writeLocalStorage(web, locale) {\n");
        code.shared(
            r#"  try {
    window.localStorage.setItem(web.options.localStorage.key, locale);
  } catch {
    // Ignore storage failures in private browsing and locked-down contexts.
  }
}
"#,
        );
    }
    if features.has_cookie {
        code.shared("\n");
        code.typed("function writeLocaleCookie(\n  web: typeof import(\"./svelte-effects.svelte.js\").web,\n  locale: Parameters<typeof web.serializeLocaleCookie>[0],\n) {\n", "/** @param {typeof import(\"./svelte-effects.svelte.js\").web} web @param {Locale} locale */\nfunction writeLocaleCookie(web, locale) {\n");
        code.shared(
            r#"  try {
    document.cookie = web.serializeLocaleCookie(locale, { httpOnly: false });
  } catch {
    // Ignore cookie failures in sandboxed and locked-down contexts.
  }
}
"#,
        );
    }
    module.push_statement(EcmaStatement::generated(code.render(target)));
    module
}

fn forwarding_methods(code: &mut RuntimeCode) {
    code.typed("    localizeHref: (href: string, locale = getCurrentLocale(), input?: Record<string, unknown>)", "    /** @param {string} href @param {Locale} [locale] @param {Record<string, unknown>} [input] */\n    localizeHref: (href, locale = getCurrentLocale(), input)");
    code.shared(" => web.localizeHref(href, locale, input),\n");
    code.typed("    localizeUrl: (url: string | URL, locale = getCurrentLocale(), input?: Record<string, unknown>)", "    /** @param {string | URL} url @param {Locale} [locale] @param {Record<string, unknown>} [input] */\n    localizeUrl: (url, locale = getCurrentLocale(), input)");
    code.shared(" => web.localizeUrl(url, locale, input),\n");
    code.typed("    shouldLocalizeHref: (href: string, input?: Record<string, unknown>)", "    /** @param {string} href @param {Record<string, unknown>} [input] */\n    shouldLocalizeHref: (href, input)");
    code.shared(" => web.shouldLocalizeHref(href, input),\n");
    code.typed("    shouldLocalizeLink: (href: string, attributes = {}, input?: Record<string, unknown>)", "    /** @param {string} href @param {LinkLocalizationAttributes} [attributes] @param {Record<string, unknown>} [input] */\n    shouldLocalizeLink: (href, attributes = {}, input)");
    code.shared(" => web.shouldLocalizeLink(href, attributes, input),\n");
    code.typed("    localizeHrefAttribute: (href: string, locale = getCurrentLocale(), input?: Record<string, unknown>)", "    /** @param {string} href @param {Locale} [locale] @param {Record<string, unknown>} [input] */\n    localizeHrefAttribute: (href, locale = getCurrentLocale(), input)");
    code.shared(" => web.localizeHrefAttribute(href, locale, input),\n");
    code.typed("    delocalizeUrl: (url: string | URL, input?: Record<string, unknown>)", "    /** @param {string | URL} url @param {Record<string, unknown>} [input] */\n    delocalizeUrl: (url, input)");
    code.shared(" => web.delocalizeUrl(url, input),\n");
    code.typed("    alternateLinks: (url: string | URL, input?: Record<string, unknown>)", "    /** @param {string | URL} url @param {Record<string, unknown>} [input] */\n    alternateLinks: (url, input)");
    code.shared(" => web.alternateLinks(url, input),\n");
}

#[cfg(test)]
mod tests {
    use super::super::{
        svelte_control_types::generate_svelte_control_declaration, TypeScriptLocaleSource,
    };
    use super::*;
    use std::path::Path;

    #[test]
    fn controls_select_navigation_persistence_and_target_types() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-svelte-control");
        for (name, sources) in [
            ("none", vec![]),
            ("path", vec![TypeScriptLocaleSource::Path]),
            ("cookie", vec![TypeScriptLocaleSource::Cookie]),
            ("storage", vec![TypeScriptLocaleSource::LocalStorage]),
            (
                "all",
                vec![
                    TypeScriptLocaleSource::Path,
                    TypeScriptLocaleSource::Cookie,
                    TypeScriptLocaleSource::LocalStorage,
                    TypeScriptLocaleSource::AcceptLanguage,
                ],
            ),
        ] {
            let options = TypeScriptWebOptions {
                sources,
                ..Default::default()
            };
            for sveltekit in [false, true] {
                let name = format!("{}-{name}", if sveltekit { "kit" } else { "plain" });
                let ts = compile_typescript_svelte_control_module(&options, sveltekit);
                let js = compile_javascript_svelte_control_module(&options, sveltekit);
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
                let declaration = super::super::templates::render_template(
                    &std::fs::read_to_string(root.join("legacy/svelte-control.runtime.d.ts"))
                        .unwrap(),
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
                        root.join(format!("{name}/typescript/svelte-control.ts")),
                        ts.code,
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
