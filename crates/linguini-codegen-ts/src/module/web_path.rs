//! Path-based locale-resolution leaf for generated web runtimes.

use crate::ecmascript::{
    EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

const PATH_BODY_BEFORE_PARSED: &str = "  if (!value) return undefined;\n";
const PATH_BODY_AFTER_PARSED: &str = r#"  try {
    parsed = new URL(String(value), "http://localhost");
  } catch {
    return undefined;
  }
  const pathname = parsed.pathname.startsWith("/") ? parsed.pathname : `/${parsed.pathname}`;
  const base = options.environment.base && options.environment.base !== "/"
    ? `/${options.environment.base.replace(/^\/+|\/+$/g, "")}`
    : "";
  const stripped = base && pathname === base
    ? "/"
    : base && pathname.startsWith(`${base}/`)
      ? pathname.slice(base.length)
      : pathname;
  const segment = stripped.split("/").filter(Boolean)[0];
  return typeof segment === "string"
    ? locales.find((locale) => locale.toLowerCase() === segment.toLowerCase())
    : undefined;
"#;

pub(super) fn generate_typescript_web_path_module() -> String {
    web_path_module(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/path.ts",
        Some("web/path.d.ts".to_owned()),
    ))
    .render_code()
}

pub(super) fn generate_web_path_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/path.d.ts",
        None,
    ));
    module.push_statement(EcmaStatement::type_declaration(
        concat!(
            "export declare function resolvePathLocale<Locale extends string>(\n",
            "  input: Record<string, unknown>,\n",
            "  options: { environment: { base: string } },\n",
            "  locales: readonly Locale[],\n",
            "): Locale | undefined;\n",
        ),
        None,
    ));
    module.render_code()
}

/// Compile path-based locale resolution as source-mapped TypeScript.
pub fn compile_typescript_web_path_module() -> RenderedEcmaModule {
    compile_web_path_module(EcmaScriptTarget::TypeScript)
}

/// Compile path-based locale resolution as checked JavaScript.
pub fn compile_javascript_web_path_module() -> RenderedEcmaModule {
    compile_web_path_module(EcmaScriptTarget::JavaScript)
}

fn compile_web_path_module(target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = match target {
        EcmaScriptTarget::TypeScript => "web/path.ts",
        EcmaScriptTarget::JavaScript => "web/path.js",
    };
    web_path_module(EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn web_path_module(output: EcmaModuleOutput) -> EcmaModule {
    let (signature, value, parsed) = match output.target() {
        EcmaScriptTarget::TypeScript => (
            concat!(
                "export function resolvePathLocale<Locale extends string>(\n",
                "  input: Record<string, unknown>,\n",
                "  options: { environment: { base: string } },\n",
                "  locales: readonly Locale[],\n",
                "): Locale | undefined {\n",
            ),
            "  const value = input.url as URL | string | undefined;\n",
            "  let parsed: URL;\n",
        ),
        EcmaScriptTarget::JavaScript => (
            concat!(
                "/**\n",
                " * @template {string} Locale\n",
                " * @param {Record<string, unknown>} input\n",
                " * @param {{environment: {base: string}}} options\n",
                " * @param {readonly Locale[]} locales\n",
                " * @returns {Locale | undefined}\n",
                " */\n",
                "export function resolvePathLocale(input, options, locales) {\n",
            ),
            "  const value = /** @type {URL | string | undefined} */ (input.url);\n",
            "  /** @type {URL} */ let parsed;\n",
        ),
    };
    let mut module = EcmaModule::new(output);
    module.push_statement(EcmaStatement::generated(format!(
        "{signature}{value}{PATH_BODY_BEFORE_PARSED}{parsed}{PATH_BODY_AFTER_PARSED}}}\n"
    )));
    module
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        compile_javascript_web_path_module, compile_typescript_web_path_module,
        generate_typescript_web_path_module, generate_web_path_declaration,
    };

    #[test]
    fn path_leaf_uses_one_target_aware_body() {
        let typescript = compile_typescript_web_path_module();
        let javascript = compile_javascript_web_path_module();
        assert_eq!(
            typescript
                .code
                .strip_suffix("//# sourceMappingURL=path.ts.map\n")
                .expect("TypeScript map trailer"),
            generate_typescript_web_path_module()
        );
        assert!(javascript.code.contains("@template {string} Locale"));
        assert!(!javascript.code.contains("let parsed: URL"));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=path.js.map\n"));
        assert!(javascript.source_map.contains("\"file\":\"web/path.js\""));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-path");
        let snapshots = [
            (root.join("web/path.js"), javascript.code),
            (root.join("web/path.js.map"), javascript.source_map),
            (root.join("typescript/web/path.ts"), typescript.code),
            (
                root.join("typescript/web/path.d.ts"),
                generate_web_path_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write path snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read path snapshot")
            );
        }
    }
}
