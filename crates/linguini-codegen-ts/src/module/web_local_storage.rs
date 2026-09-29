//! Local-storage locale-resolution leaf for generated web runtimes.

use crate::ecmascript::{
    EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

const STORAGE_BODY_BEFORE_VALUE: &str = "  try {\n    return matchLocale(\n      ";
const STORAGE_BODY_AFTER_VALUE: &str = r#"?.getItem(options.localStorage.key) ?? undefined,
    );
  } catch {
    return undefined;
  }
"#;

pub(super) fn generate_typescript_web_local_storage_module() -> String {
    web_local_storage_module(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/local-storage.ts",
        Some("web/local-storage.d.ts".to_owned()),
    ))
    .render_code()
}

pub(super) fn generate_web_local_storage_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/local-storage.d.ts",
        None,
    ));
    module.push_statement(EcmaStatement::type_declaration(
        concat!(
            "export declare function resolveLocalStorageLocale<Locale extends string>(\n",
            "  input: Record<string, unknown>,\n",
            "  options: { localStorage: { key: string } },\n",
            "  matchLocale: (value: unknown) => Locale | undefined,\n",
            "): Locale | undefined;\n",
        ),
        None,
    ));
    module.render_code()
}

/// Compile local-storage locale resolution as source-mapped TypeScript.
pub fn compile_typescript_web_local_storage_module() -> RenderedEcmaModule {
    compile_web_local_storage_module(EcmaScriptTarget::TypeScript)
}

/// Compile local-storage locale resolution as checked JavaScript.
pub fn compile_javascript_web_local_storage_module() -> RenderedEcmaModule {
    compile_web_local_storage_module(EcmaScriptTarget::JavaScript)
}

fn compile_web_local_storage_module(target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = match target {
        EcmaScriptTarget::TypeScript => "web/local-storage.ts",
        EcmaScriptTarget::JavaScript => "web/local-storage.js",
    };
    web_local_storage_module(EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn web_local_storage_module(output: EcmaModuleOutput) -> EcmaModule {
    let (signature, value) = match output.target() {
        EcmaScriptTarget::TypeScript => (
            concat!(
                "export function resolveLocalStorageLocale<Locale extends string>(\n",
                "  input: Record<string, unknown>,\n",
                "  options: { localStorage: { key: string } },\n",
                "  matchLocale: (value: unknown) => Locale | undefined,\n",
                "): Locale | undefined {\n",
            ),
            "(input.localStorage as Storage | undefined)",
        ),
        EcmaScriptTarget::JavaScript => (
            concat!(
                "/**\n",
                " * @template {string} Locale\n",
                " * @param {Record<string, unknown>} input\n",
                " * @param {{localStorage: {key: string}}} options\n",
                " * @param {(value: unknown) => Locale | undefined} matchLocale\n",
                " * @returns {Locale | undefined}\n",
                " */\n",
                "export function resolveLocalStorageLocale(input, options, matchLocale) {\n",
            ),
            "(/** @type {Storage | undefined} */ (input.localStorage))",
        ),
    };
    let mut module = EcmaModule::new(output);
    module.push_statement(EcmaStatement::generated(format!(
        "{signature}{STORAGE_BODY_BEFORE_VALUE}{value}{STORAGE_BODY_AFTER_VALUE}}}\n"
    )));
    module
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        compile_javascript_web_local_storage_module, compile_typescript_web_local_storage_module,
        generate_typescript_web_local_storage_module, generate_web_local_storage_declaration,
    };

    #[test]
    fn local_storage_leaf_uses_one_target_aware_body() {
        let typescript = compile_typescript_web_local_storage_module();
        let javascript = compile_javascript_web_local_storage_module();
        assert_eq!(
            typescript
                .code
                .strip_suffix("//# sourceMappingURL=local-storage.ts.map\n")
                .expect("TypeScript map trailer"),
            generate_typescript_web_local_storage_module()
        );
        assert!(javascript.code.contains("@template {string} Locale"));
        assert!(!javascript.code.contains("input.localStorage as Storage"));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=local-storage.js.map\n"));
        assert!(javascript
            .source_map
            .contains("\"file\":\"web/local-storage.js\""));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-local-storage");
        let snapshots = [
            (root.join("web/local-storage.js"), javascript.code),
            (root.join("web/local-storage.js.map"), javascript.source_map),
            (
                root.join("typescript/web/local-storage.ts"),
                typescript.code,
            ),
            (
                root.join("typescript/web/local-storage.d.ts"),
                generate_web_local_storage_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write local-storage snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read local-storage snapshot")
            );
        }
    }
}
