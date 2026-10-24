//! Module-owned templates with the shared ECMAScript rendering policy.
use linguini_codegen_ecmascript::EcmaTemplateEngine;
use minijinja::Value;
use std::sync::OnceLock;
pub(crate) fn render(name: &str, values: Value) -> String {
    static ENGINE: OnceLock<EcmaTemplateEngine> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            EcmaTemplateEngine::new([
                (
                    "svelte/facade",
                    include_str!("../templates/svelte/facade.jinja"),
                ),
                (
                    "svelte/facade_declarations",
                    include_str!("../templates/svelte/facade_declarations.jinja"),
                ),
                (
                    "svelte/locale",
                    include_str!("../templates/svelte/locale.jinja"),
                ),
                (
                    "svelte/locale_declarations",
                    include_str!("../templates/svelte/locale_declarations.jinja"),
                ),
            ])
        })
        .render(name, values)
}
