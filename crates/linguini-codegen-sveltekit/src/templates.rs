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
                    "sveltekit/locale",
                    include_str!("../templates/sveltekit/locale.jinja"),
                ),
                (
                    "sveltekit/locale_declarations",
                    include_str!("../templates/sveltekit/locale_declarations.jinja"),
                ),
                (
                    "sveltekit/navigation",
                    include_str!("../templates/sveltekit/navigation.jinja"),
                ),
                (
                    "sveltekit/navigation_declarations",
                    include_str!("../templates/sveltekit/navigation_declarations.jinja"),
                ),
                (
                    "sveltekit/adapter",
                    include_str!("../templates/sveltekit/adapter.jinja"),
                ),
                (
                    "sveltekit/ambient_declarations",
                    include_str!("../templates/sveltekit/ambient_declarations.jinja"),
                ),
                (
                    "sveltekit/context",
                    include_str!("../templates/sveltekit/context.jinja"),
                ),
                (
                    "sveltekit/control_declarations",
                    include_str!("../templates/sveltekit/control_declarations.jinja"),
                ),
                (
                    "sveltekit/request_context",
                    include_str!("../templates/sveltekit/request_context.jinja"),
                ),
                (
                    "sveltekit/web_factory",
                    include_str!("../templates/sveltekit/web_factory.jinja"),
                ),
            ])
        })
        .render(name, values)
}
