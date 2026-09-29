//! Build-time transformed-link helper for generated web projects.

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

const LINK_BODY: &str = r#"  return web.shouldLocalizeLink(href, attributes, input)
    ? web.localizeHref(href, getCurrentLocale(), input)
    : href;
}
"#;

pub(super) fn generate_typescript_web_link_transform_module() -> String {
    web_link_transform_module(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/link-transform.ts",
        Some("web/link-transform.d.ts".to_owned()),
    ))
    .render_code()
}

pub(super) fn generate_web_link_transform_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/link-transform.d.ts",
        None,
    ));
    module.push_import(type_import());
    module.push_statement(EcmaStatement::type_declaration(
        concat!(
            "export declare function localizeTransformedHref(\n",
            "  href: string,\n",
            "  attributes?: LinkLocalizationAttributes,\n",
            "  input?: Record<string, unknown>,\n",
            "): string;\n",
        ),
        None,
    ));
    module.render_code()
}

/// Compile the transformed-link helper as source-mapped TypeScript.
pub fn compile_typescript_web_link_transform_module() -> RenderedEcmaModule {
    compile_web_link_transform_module(EcmaScriptTarget::TypeScript)
}

/// Compile the transformed-link helper as checked JavaScript.
pub fn compile_javascript_web_link_transform_module() -> RenderedEcmaModule {
    compile_web_link_transform_module(EcmaScriptTarget::JavaScript)
}

fn compile_web_link_transform_module(target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = match target {
        EcmaScriptTarget::TypeScript => "web/link-transform.ts",
        EcmaScriptTarget::JavaScript => "web/link-transform.js",
    };
    web_link_transform_module(EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn web_link_transform_module(output: EcmaModuleOutput) -> EcmaModule {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    if target.is_typescript() {
        module.push_import(type_import());
    }
    module.push_import(EcmaImport::named(
        "../svelte-effects.svelte.js",
        vec![EcmaNamedImport::new("web", "web")],
    ));
    module.push_import(EcmaImport::named(
        "../svelte-locale.svelte.js",
        vec![EcmaNamedImport::new("getCurrentLocale", "getCurrentLocale")],
    ));
    let signature = match target {
        EcmaScriptTarget::TypeScript => concat!(
            "export function localizeTransformedHref(\n",
            "  href: string,\n",
            "  attributes: LinkLocalizationAttributes = {},\n",
            "  input: Record<string, unknown> = {},\n",
            ") {\n",
        ),
        EcmaScriptTarget::JavaScript => concat!(
            "/**\n",
            " * @param {string} href\n",
            " * @param {{download?: boolean, ignored?: boolean, rel?: string | null}} [attributes={}]\n",
            " * @param {Record<string, unknown>} [input={}]\n",
            " * @returns {string}\n",
            " */\n",
            "export function localizeTransformedHref(href, attributes = {}, input = {}) {\n",
        ),
    };
    module.push_statement(EcmaStatement::generated(format!("{signature}{LINK_BODY}")));
    module
}

fn type_import() -> EcmaImport {
    EcmaImport {
        specifier: "../web".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
            "LinkLocalizationAttributes",
            "LinkLocalizationAttributes",
        )]),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        compile_javascript_web_link_transform_module, compile_typescript_web_link_transform_module,
        generate_typescript_web_link_transform_module, generate_web_link_transform_declaration,
    };

    #[test]
    fn link_transform_leaf_uses_one_target_aware_body() {
        let typescript = compile_typescript_web_link_transform_module();
        let javascript = compile_javascript_web_link_transform_module();
        assert_eq!(
            typescript
                .code
                .strip_suffix("//# sourceMappingURL=link-transform.ts.map\n")
                .expect("TypeScript map trailer"),
            generate_typescript_web_link_transform_module()
        );
        assert!(javascript.code.contains("@returns {string}"));
        assert!(!javascript.code.contains("import type"));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=link-transform.js.map\n"));
        assert!(javascript
            .source_map
            .contains("\"file\":\"web/link-transform.js\""));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-link-transform");
        let snapshots = [
            (root.join("web/link-transform.js"), javascript.code),
            (
                root.join("web/link-transform.js.map"),
                javascript.source_map,
            ),
            (
                root.join("typescript/web/link-transform.ts"),
                typescript.code,
            ),
            (
                root.join("typescript/web/link-transform.d.ts"),
                generate_web_link_transform_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write link-transform snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read link-transform snapshot")
            );
        }
    }
}
