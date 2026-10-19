//! Closed route-exclusion capability for generated web runtimes.

use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_routes_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/routes.d.ts", None),
        "web/routes_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the selected web capability as checked JavaScript.
pub fn compile_web_routes_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/routes.js", None),
        "web/routes",
        minijinja::context! {},
    )
    .render(&[])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{compile_web_routes_module, generate_web_routes_declaration};

    #[test]
    fn route_leaf_preserves_runtime_and_declaration_contract() {
        let javascript = compile_web_routes_module();
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
