//! Path-based locale-resolution leaf for generated web runtimes.

use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_path_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/path.d.ts", None),
        "web/path_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the selected web capability as checked JavaScript.
pub fn compile_web_path_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/path.js", None),
        "web/path",
        minijinja::context! {},
    )
    .render(&[])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{compile_web_path_module, generate_web_path_declaration};

    #[test]
    fn path_leaf_preserves_runtime_and_declaration_contract() {
        let javascript = compile_web_path_module();
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
