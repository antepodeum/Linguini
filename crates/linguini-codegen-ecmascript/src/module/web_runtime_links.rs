//! Owned, bounded browser link-localization runtime.

use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub(super) fn generate_web_runtime_links_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/runtime-links.d.ts", None),
        "web/runtime-links_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile the selected web capability as checked JavaScript.
pub fn compile_web_runtime_links_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/runtime-links.js", None),
        "web/runtime-links",
        minijinja::context! {},
    )
    .render(&[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn runtime_links_preserve_runtime_and_declaration_snapshots() {
        let javascript = compile_web_runtime_links_module();
        assert!(!javascript.code.contains("import type"));
        assert!(javascript.code.contains("@type {Node[]}"));
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-runtime-links");
        let snapshots = [
            (root.join("web/runtime-links.js"), javascript.code),
            (root.join("web/runtime-links.js.map"), javascript.source_map),
            (
                root.join("typescript/web/runtime-links.d.ts"),
                generate_web_runtime_links_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, contents).unwrap();
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(contents, std::fs::read_to_string(path).unwrap());
        }
    }
}
