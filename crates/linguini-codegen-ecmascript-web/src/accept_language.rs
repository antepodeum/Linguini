//! Accept-Language locale-resolution leaf for generated web runtimes.

use linguini_codegen_ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_accept_language_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(
            EcmaModuleKind::Declaration,
            "web/accept-language.d.ts",
            None,
        ),
        "web/accept-language_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the selected web capability as checked JavaScript.
pub fn compile_web_accept_language_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/accept-language.js", None),
        "web/accept-language",
        minijinja::context! {},
    )
    .render(&[])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{compile_web_accept_language_module, generate_web_accept_language_declaration};

    #[test]
    fn accept_language_leaf_preserves_runtime_and_declaration_contract() {
        let javascript = compile_web_accept_language_module();
        assert!(javascript.code.contains("@template {string} Locale"));
        assert!(!javascript.code.contains("let best:"));
        assert!(!javascript.code.contains(".filter("));
        assert!(!javascript.code.contains(".sort("));
        assert!(!javascript.code.contains("locales.entries()"));
        assert!(!javascript.code.contains("const candidate = {"));
        assert!(javascript
            .code
            .ends_with("//# sourceMappingURL=accept-language.js.map\n"));
        assert!(javascript
            .source_map
            .contains("\"file\":\"web/accept-language.js\""));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-accept-language");
        let snapshots = [
            (root.join("web/accept-language.js"), javascript.code),
            (
                root.join("web/accept-language.js.map"),
                javascript.source_map,
            ),
            (
                root.join("typescript/web/accept-language.d.ts"),
                generate_web_accept_language_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().expect("snapshot parent"))
                    .expect("create snapshot directory");
                std::fs::write(path, contents).expect("write Accept-Language snapshot");
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(
                contents,
                std::fs::read_to_string(path).expect("read Accept-Language snapshot")
            );
        }
    }
}
