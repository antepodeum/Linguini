//! Local-storage locale-resolution leaf for generated web runtimes.

use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_local_storage_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/local-storage.d.ts", None),
        "web/local-storage_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the selected web capability as checked JavaScript.
pub fn compile_web_local_storage_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/local-storage.js", None),
        "web/local-storage",
        minijinja::context! {},
    )
    .render(&[])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{compile_web_local_storage_module, generate_web_local_storage_declaration};

    #[test]
    fn local_storage_leaf_preserves_runtime_and_declaration_contract() {
        let javascript = compile_web_local_storage_module();
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
