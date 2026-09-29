//! Closed route-exclusion capability for generated web runtimes.

use crate::ecmascript::{
    EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

const ROUTE_BODY: &str = concat!(
    "  if (typeof pattern === \"function\") return Boolean(pattern(url));\n",
    "  if (pattern instanceof RegExp) {\n",
    "    if (!pattern.global && !pattern.sticky) return pattern.test(url.pathname);\n",
    "    pattern.lastIndex = 0;\n",
    "    try {\n",
    "      return pattern.test(url.pathname);\n",
    "    } finally {\n",
    "      pattern.lastIndex = 0;\n",
    "    }\n",
    "  }\n",
    "  if (pattern.endsWith(\"/**\")) {\n",
    "    const prefix = pattern.slice(0, -3).replace(/\\/+$/, \"\");\n",
    "    return !prefix\n",
    "      || prefix === \"/\"\n",
    "      || url.pathname === prefix\n",
    "      || url.pathname.startsWith(`${prefix}/`);\n",
    "  }\n",
    "  return url.pathname === pattern;\n",
);

pub(super) fn generate_typescript_web_routes_module() -> String {
    web_routes_module(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/routes.ts",
        Some("web/routes.d.ts".to_owned()),
    ))
    .render_code()
}

pub(super) fn generate_web_routes_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/routes.d.ts",
        None,
    ));
    module.push_statement(EcmaStatement::type_declaration(
        concat!(
            "export declare function matchesRoute(\n",
            "  pattern: string | RegExp | ((url: URL) => boolean),\n",
            "  url: URL,\n",
            "): boolean;\n",
        ),
        None,
    ));
    module.render_code()
}

/// Compile the route-exclusion helper as source-mapped TypeScript.
pub fn compile_typescript_web_routes_module() -> RenderedEcmaModule {
    compile_web_routes_module(EcmaScriptTarget::TypeScript)
}

/// Compile the route-exclusion helper as checked JavaScript.
pub fn compile_javascript_web_routes_module() -> RenderedEcmaModule {
    compile_web_routes_module(EcmaScriptTarget::JavaScript)
}

fn compile_web_routes_module(target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = match target {
        EcmaScriptTarget::TypeScript => "web/routes.ts",
        EcmaScriptTarget::JavaScript => "web/routes.js",
    };
    web_routes_module(EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn web_routes_module(output: EcmaModuleOutput) -> EcmaModule {
    let signature = match output.target() {
        EcmaScriptTarget::TypeScript => concat!(
            "export function matchesRoute(\n",
            "  pattern: string | RegExp | ((url: URL) => boolean),\n",
            "  url: URL,\n",
            ") {\n",
        ),
        EcmaScriptTarget::JavaScript => concat!(
            "/**\n",
            " * @param {string | RegExp | ((url: URL) => boolean)} pattern\n",
            " * @param {URL} url\n",
            " * @returns {boolean}\n",
            " */\n",
            "export function matchesRoute(pattern, url) {\n",
        ),
    };
    let mut module = EcmaModule::new(output);
    module.push_statement(EcmaStatement::generated(format!(
        "{signature}{ROUTE_BODY}}}\n"
    )));
    module
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        compile_javascript_web_routes_module, compile_typescript_web_routes_module,
        generate_typescript_web_routes_module, generate_web_routes_declaration,
    };

    #[test]
    fn route_leaf_is_target_aware_and_keeps_typescript_bytes() {
        let typescript = compile_typescript_web_routes_module();
        let javascript = compile_javascript_web_routes_module();
        assert_eq!(
            typescript
                .code
                .strip_suffix("//# sourceMappingURL=routes.ts.map\n")
                .expect("TypeScript map trailer"),
            generate_typescript_web_routes_module()
        );
        assert!(javascript.code.contains("@returns {boolean}"));
        assert!(!javascript.code.contains("pattern: string"));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=routes.js.map\n"));
        assert!(javascript.source_map.contains("\"file\":\"web/routes.js\""));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-routes");
        let snapshots = [
            (root.join("web/routes.js"), javascript.code),
            (root.join("web/routes.js.map"), javascript.source_map),
            (root.join("typescript/web/routes.ts"), typescript.code),
            (
                root.join("typescript/web/routes.d.ts"),
                generate_web_routes_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write route snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read route snapshot")
            );
        }
    }
}
