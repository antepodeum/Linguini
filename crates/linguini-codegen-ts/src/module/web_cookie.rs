//! Cookie-based locale-resolution leaf for generated web runtimes.

use crate::ecmascript::{
    EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

const COOKIE_BODY_BEFORE_VALUE: &str = "  for (const cookie of String(";
const COOKIE_BODY_AFTER_VALUE: &str = r#").split(";")) {
    const [rawName, ...rawValue] = cookie.trim().split("=");
    if (rawName !== options.cookie.name) continue;
    try {
      return matchLocale(decodeURIComponent(rawValue.join("=")));
    } catch {
      return undefined;
    }
  }
  return undefined;
"#;

pub(super) fn generate_typescript_web_cookie_module() -> String {
    web_cookie_module(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/cookie.ts",
        Some("web/cookie.d.ts".to_owned()),
    ))
    .render_code()
}

pub(super) fn generate_web_cookie_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/cookie.d.ts",
        None,
    ));
    module.push_statement(EcmaStatement::type_declaration(
        concat!(
            "export declare function resolveCookieLocale<Locale extends string>(\n",
            "  input: Record<string, unknown>,\n",
            "  options: { cookie: { name: string } },\n",
            "  matchLocale: (value: unknown) => Locale | undefined,\n",
            "): Locale | undefined;\n",
        ),
        None,
    ));
    module.render_code()
}

/// Compile cookie-based locale resolution as source-mapped TypeScript.
pub fn compile_typescript_web_cookie_module() -> RenderedEcmaModule {
    compile_web_cookie_module(EcmaScriptTarget::TypeScript)
}

/// Compile cookie-based locale resolution as checked JavaScript.
pub fn compile_javascript_web_cookie_module() -> RenderedEcmaModule {
    compile_web_cookie_module(EcmaScriptTarget::JavaScript)
}

fn compile_web_cookie_module(target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = match target {
        EcmaScriptTarget::TypeScript => "web/cookie.ts",
        EcmaScriptTarget::JavaScript => "web/cookie.js",
    };
    web_cookie_module(EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn web_cookie_module(output: EcmaModuleOutput) -> EcmaModule {
    let (signature, value) = match output.target() {
        EcmaScriptTarget::TypeScript => (
            concat!(
                "export function resolveCookieLocale<Locale extends string>(\n",
                "  input: Record<string, unknown>,\n",
                "  options: { cookie: { name: string } },\n",
                "  matchLocale: (value: unknown) => Locale | undefined,\n",
                "): Locale | undefined {\n",
            ),
            "input.cookie as string | undefined ?? \"\"",
        ),
        EcmaScriptTarget::JavaScript => (
            concat!(
                "/**\n",
                " * @template {string} Locale\n",
                " * @param {Record<string, unknown>} input\n",
                " * @param {{cookie: {name: string}}} options\n",
                " * @param {(value: unknown) => Locale | undefined} matchLocale\n",
                " * @returns {Locale | undefined}\n",
                " */\n",
                "export function resolveCookieLocale(input, options, matchLocale) {\n",
            ),
            "(/** @type {string | undefined} */ (input.cookie)) ?? \"\"",
        ),
    };
    let mut module = EcmaModule::new(output);
    module.push_statement(EcmaStatement::generated(format!(
        "{signature}{COOKIE_BODY_BEFORE_VALUE}{value}{COOKIE_BODY_AFTER_VALUE}}}\n"
    )));
    module
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        compile_javascript_web_cookie_module, compile_typescript_web_cookie_module,
        generate_typescript_web_cookie_module, generate_web_cookie_declaration,
    };

    #[test]
    fn cookie_leaf_uses_one_target_aware_body() {
        let typescript = compile_typescript_web_cookie_module();
        let javascript = compile_javascript_web_cookie_module();
        assert_eq!(
            typescript
                .code
                .strip_suffix("//# sourceMappingURL=cookie.ts.map\n")
                .expect("TypeScript map trailer"),
            generate_typescript_web_cookie_module()
        );
        assert!(javascript.code.contains("@template {string} Locale"));
        assert!(!javascript.code.contains("input.cookie as string"));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=cookie.js.map\n"));
        assert!(javascript.source_map.contains("\"file\":\"web/cookie.js\""));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-cookie");
        let snapshots = [
            (root.join("web/cookie.js"), javascript.code),
            (root.join("web/cookie.js.map"), javascript.source_map),
            (root.join("typescript/web/cookie.ts"), typescript.code),
            (
                root.join("typescript/web/cookie.d.ts"),
                generate_web_cookie_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write cookie snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read cookie snapshot")
            );
        }
    }
}
