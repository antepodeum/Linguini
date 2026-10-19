//! Build-time transformed-link helper for generated web projects.

use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_link_transform_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/link-transform.d.ts", None),
        "web/link-transform_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the selected web capability as checked JavaScript.
pub fn compile_web_link_transform_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/link-transform.js", None),
        "web/link-transform",
        minijinja::context! {},
    )
    .render(&[])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{compile_web_link_transform_module, generate_web_link_transform_declaration};

    #[test]
    fn link_transform_leaf_preserves_runtime_and_declaration_contract() {
        let javascript = compile_web_link_transform_module();
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
