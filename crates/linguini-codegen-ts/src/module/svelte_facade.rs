//! Svelte message-provider facade, with optional lightweight web controls.

use super::runtime_code::RuntimeCode;
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

const WEB_METHODS: &[&str] = &[
    "localizeHref",
    "localizeUrl",
    "shouldLocalizeHref",
    "shouldLocalizeLink",
    "localizeHrefAttribute",
    "delocalizeUrl",
    "alternateLinks",
];

pub(super) fn generate_typescript_svelte_module(web: bool) -> String {
    facade_module(web, EcmaScriptTarget::TypeScript).render_code()
}

/// Compile the context-only or web-enabled Svelte message facade as TypeScript.
pub fn compile_typescript_svelte_module(web: bool) -> RenderedEcmaModule {
    facade_module(web, EcmaScriptTarget::TypeScript).render(&[])
}

/// Compile the context-only or web-enabled Svelte message facade as checked JavaScript.
pub fn compile_javascript_svelte_module(web: bool) -> RenderedEcmaModule {
    facade_module(web, EcmaScriptTarget::JavaScript).render(&[])
}

fn facade_module(web: bool, target: EcmaScriptTarget) -> EcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        target,
        if target.is_typescript() {
            "svelte.ts"
        } else {
            "svelte.js"
        },
        None,
    ));
    module.push_import(EcmaImport {
        specifier: if target.is_typescript() {
            "./index"
        } else {
            "./index.js"
        }
        .to_owned(),
        bindings: EcmaImportBindings::Namespace("runtime".to_owned()),
    });
    let mut code = RuntimeCode::default();
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
        code.shared("const unregisterRuntimeLoader = registerLocaleLoader(async (locale) => {\n  await runtime.prepareLinguini(locale);\n});\n");
        super::hmr::register_disposer(&mut code, "unregisterRuntimeLoader");
        code.shared("\nexport const linguini = createLinguiniRune(runtime, controls);\n");
    } else {
        module.push_import(EcmaImport::named(
            "./svelte-locale.svelte.js",
            ["getCurrentLocale", "prepareLocale", "setCurrentLocale"]
                .into_iter()
                .map(|name| EcmaNamedImport::new(name, name))
                .collect(),
        ));
        code.shared("export const linguini = createLinguiniRune(runtime);\n");
    }
    for name in ["l", "messages", "setLocale"]
        .into_iter()
        .chain(if web { WEB_METHODS } else { &[] }.iter().copied())
    {
        code.generated(format!("export const {name} = linguini.{name};\n"));
    }
    code.shared("\n");
    if web {
        code.typed("function createLinguiniRune(\n  runtime: typeof import(\"./index\"),\n  controls: typeof import(\"./svelte-control.js\").linguini,\n) {\n", "/** @param {typeof import(\"./index.js\")} runtime @param {typeof import(\"./svelte-control.js\").linguini} controls */\nfunction createLinguiniRune(runtime, controls) {\n");
        code.shared("  const messages = runtime.createLinguiniProvider({\n    getLocale: () => controls.locale,\n  });\n\n");
    } else {
        code.typed("function createLinguiniRune(runtime: typeof import(\"./index\")) {\n", "/** @param {typeof import(\"./index.js\")} runtime */\nfunction createLinguiniRune(runtime) {\n");
        code.shared("  const messages = runtime.createLinguiniProvider({\n    getLocale: getCurrentLocale,\n  });\n\n");
        code.typed("  async function setLocale(nextLocale: string) {\n", "  /** @param {string} nextLocale @returns {Promise<import(\"./index.js\").Locale>} */\n  async function setLocale(nextLocale) {\n");
        code.shared("    return setCurrentLocale(await prepareLocale(nextLocale));\n  }\n\n");
    }
    code.shared("  return {\n    messages,\n    l: messages,\n");
    for name in ["locale", "lang", "direction", "textDirection", "htmlAttrs"] {
        code.generated(format!("    get {name}() {{\n"));
        if web {
            code.generated(format!("      return controls.{name};\n"));
        } else {
            match name {
                "locale" | "lang" => code.shared("      return getCurrentLocale();\n"),
                "direction" | "textDirection" => code.shared("      return runtime.getTextDirection(getCurrentLocale());\n"),
                _ => code.shared("      const locale = getCurrentLocale();\n      return { lang: locale, dir: runtime.getTextDirection(locale) };\n"),
            }
        }
        code.shared("    },\n");
    }
    if web {
        for name in ["setLocale"]
            .into_iter()
            .chain(WEB_METHODS.iter().copied())
            .chain(["destroy"])
        {
            code.generated(format!("    {name}: controls.{name},\n"));
        }
    } else {
        code.shared("    setLocale,\n");
    }
    code.shared("  };\n}\n");
    module.push_statement(EcmaStatement::generated(code.render(target)));
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
            let ts = compile_typescript_svelte_module(web);
            let js = compile_javascript_svelte_module(web);
            assert_eq!(js.code.contains("registerLocaleLoader"), web);
            assert_eq!(js.code.contains("controls.destroy"), web);
            assert!(!js.code.contains("MutationObserver"));
            let legacy =
                std::fs::read_to_string(root.join(format!("{name}/legacy/svelte.runtime.ts")))
                    .unwrap();
            let body = |code: &str| {
                code[code
                    .find(if web {
                        "const unregisterRuntimeLoader"
                    } else {
                        "export const linguini"
                    })
                    .unwrap()..]
                    .lines()
                    .filter(|line| !line.starts_with("//# sourceMappingURL="))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            assert_eq!(body(&ts.code), body(&legacy));
            assert_eq!(
                generate_svelte_declaration(web),
                std::fs::read_to_string(root.join(format!("{name}/legacy/svelte.runtime.d.ts")))
                    .unwrap()
            );
            let mut snapshots = vec![
                (root.join(format!("{name}/svelte.js")), js.code),
                (root.join(format!("{name}/svelte.js.map")), js.source_map),
                (root.join(format!("{name}/typescript/svelte.ts")), ts.code),
                (
                    root.join(format!("{name}/typescript/svelte.d.ts")),
                    generate_svelte_declaration(web),
                ),
            ];
            if web {
                snapshots.push((
                    root.join("web/svelte-control.d.ts"),
                    super::super::svelte_control_types::generate_svelte_control_declaration(true),
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
