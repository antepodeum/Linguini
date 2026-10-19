//! Cookie-based locale-resolution leaf for generated web runtimes.

use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_cookie_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/cookie.d.ts", None),
        "web/cookie_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the selected web capability as checked JavaScript.
pub fn compile_web_cookie_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/cookie.js", None),
        "web/cookie",
        minijinja::context! {},
    )
    .render(&[])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{compile_web_cookie_module, generate_web_cookie_declaration};

    #[test]
    fn cookie_leaf_preserves_runtime_and_declaration_contract() {
        let javascript = compile_web_cookie_module();
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
