//! Module-owned templates with the shared ECMAScript rendering policy.
use linguini_codegen_ecmascript::EcmaTemplateEngine;
use minijinja::Value;
use std::sync::OnceLock;
pub(crate) fn render(name: &str, values: Value) -> String {
    static ENGINE: OnceLock<EcmaTemplateEngine> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            EcmaTemplateEngine::new([
                ("hmr", include_str!("../templates/hmr.jinja")),
                (
                    "svelte/control",
                    include_str!("../templates/svelte/control.jinja"),
                ),
                (
                    "svelte/control_declarations",
                    include_str!("../templates/svelte/control_declarations.jinja"),
                ),
                (
                    "svelte/effects",
                    include_str!("../templates/svelte/effects.jinja"),
                ),
                (
                    "svelte/effects_declarations",
                    include_str!("../templates/svelte/effects_declarations.jinja"),
                ),
                (
                    "svelte/facade",
                    include_str!("../templates/svelte/facade.jinja"),
                ),
                (
                    "svelte/facade_declarations",
                    include_str!("../templates/svelte/facade_declarations.jinja"),
                ),
                (
                    "svelte/forwarding",
                    include_str!("../templates/svelte/forwarding.jinja"),
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
