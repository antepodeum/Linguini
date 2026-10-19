//! Build-time rendering of shared ECMAScript source templates.

use std::sync::OnceLock;

use minijinja::{Environment, UndefinedBehavior, Value};

fn environment() -> &'static Environment<'static> {
    static ENVIRONMENT: OnceLock<Environment<'static>> = OnceLock::new();
    ENVIRONMENT.get_or_init(|| {
        let mut environment = Environment::new();
        environment.set_keep_trailing_newline(true);
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment.add_filter("js", crate::ecmascript::string_literal);
        for (name, source) in [
            (
                "index_declarations",
                include_str!("../templates/index_declarations.jinja"),
            ),
            (
                "sveltekit/ambient_declarations",
                include_str!("../templates/sveltekit/ambient_declarations.jinja"),
            ),
            (
                "sveltekit/control_declarations",
                include_str!("../templates/sveltekit/control_declarations.jinja"),
            ),
            (
                "svelte/effects_declarations",
                include_str!("../templates/svelte/effects_declarations.jinja"),
            ),
            (
                "svelte/control_declarations",
                include_str!("../templates/svelte/control_declarations.jinja"),
            ),
            (
                "svelte/facade_declarations",
                include_str!("../templates/svelte/facade_declarations.jinja"),
            ),
            (
                "svelte/locale_declarations",
                include_str!("../templates/svelte/locale_declarations.jinja"),
            ),
            (
                "web/type_declaration",
                include_str!("../templates/web/type_declaration.jinja"),
            ),
            (
                "web/type_jsdoc",
                include_str!("../templates/web/type_jsdoc.jinja"),
            ),
            (
                "web/factory_declaration",
                include_str!("../templates/web/factory_declaration.jinja"),
            ),
            (
                "module/constant",
                include_str!("../templates/module/constant.jinja"),
            ),
            (
                "module/form",
                include_str!("../templates/module/form.jinja"),
            ),
            (
                "module/message",
                include_str!("../templates/module/message.jinja"),
            ),
            (
                "module/message_property",
                include_str!("../templates/module/message_property.jinja"),
            ),
            (
                "module/object",
                include_str!("../templates/module/object.jinja"),
            ),
            (
                "module/locale_default",
                include_str!("../templates/module/locale_default.jinja"),
            ),
            (
                "module/normalization",
                include_str!("../templates/module/normalization.jinja"),
            ),
            ("functions", include_str!("../templates/functions.jinja")),
            (
                "web/options",
                include_str!("../templates/web/options.jinja"),
            ),
            ("expression", include_str!("../templates/expression.jinja")),
            (
                "formatters/date",
                include_str!("../templates/formatters/date.jinja"),
            ),
            (
                "formatters/date_parts",
                include_str!("../templates/formatters/date_parts.jinja"),
            ),
            (
                "formatters/types",
                include_str!("../templates/formatters/types.jinja"),
            ),
            (
                "formatters/date_expression",
                include_str!("../templates/formatters/date_expression.jinja"),
            ),
            (
                "formatter_call",
                include_str!("../templates/formatter_call.jinja"),
            ),
            (
                "formatters/number",
                include_str!("../templates/formatters/number.jinja"),
            ),
            ("dispatch", include_str!("../templates/dispatch.jinja")),
            (
                "dispatch_function",
                include_str!("../templates/dispatch_function.jinja"),
            ),
            ("selector", include_str!("../templates/selector.jinja")),
            (
                "local_function",
                include_str!("../templates/local_function.jinja"),
            ),
            ("plural", include_str!("../templates/plural.jinja")),
            (
                "svelte/facade",
                include_str!("../templates/svelte/facade.jinja"),
            ),
            ("hmr", include_str!("../templates/hmr.jinja")),
            (
                "svelte/locale",
                include_str!("../templates/svelte/locale.jinja"),
            ),
            (
                "svelte/forwarding",
                include_str!("../templates/svelte/forwarding.jinja"),
            ),
            (
                "svelte/control",
                include_str!("../templates/svelte/control.jinja"),
            ),
            (
                "svelte/effects",
                include_str!("../templates/svelte/effects.jinja"),
            ),
            (
                "sveltekit/adapter",
                include_str!("../templates/sveltekit/adapter.jinja"),
            ),
            (
                "sveltekit/request_context",
                include_str!("../templates/sveltekit/request_context.jinja"),
            ),
            (
                "sveltekit/web_factory",
                include_str!("../templates/sveltekit/web_factory.jinja"),
            ),
            (
                "sveltekit/context",
                include_str!("../templates/sveltekit/context.jinja"),
            ),
            (
                "project/locale",
                include_str!("../templates/project/locale.jinja"),
            ),
            (
                "project/locale_declarations",
                include_str!("../templates/project/locale_declarations.jinja"),
            ),
            (
                "project/index",
                include_str!("../templates/project/index.jinja"),
            ),
            (
                "web/constants",
                include_str!("../templates/web/constants.jinja"),
            ),
            ("web/facade", include_str!("../templates/web/facade.jinja")),
            (
                "web/request",
                include_str!("../templates/web/request.jinja"),
            ),
            ("web/url", include_str!("../templates/web/url.jinja")),
            (
                "number_helpers",
                include_str!("../templates/number_helpers.jinja"),
            ),
            (
                "digit_helpers",
                include_str!("../templates/digit_helpers.jinja"),
            ),
            (
                "date_helpers",
                include_str!("../templates/date_helpers.jinja"),
            ),
            (
                "plural_operands",
                include_str!("../templates/plural_operands.jinja"),
            ),
            (
                "provider_runtime",
                include_str!("../templates/provider_runtime.jinja"),
            ),
            ("web/path", include_str!("../templates/web/path.jinja")),
            (
                "web/path_declarations",
                include_str!("../templates/web/path_declarations.jinja"),
            ),
            ("web/cookie", include_str!("../templates/web/cookie.jinja")),
            (
                "web/cookie_declarations",
                include_str!("../templates/web/cookie_declarations.jinja"),
            ),
            (
                "web/local-storage",
                include_str!("../templates/web/local-storage.jinja"),
            ),
            (
                "web/local-storage_declarations",
                include_str!("../templates/web/local-storage_declarations.jinja"),
            ),
            (
                "web/accept-language",
                include_str!("../templates/web/accept-language.jinja"),
            ),
            (
                "web/accept-language_declarations",
                include_str!("../templates/web/accept-language_declarations.jinja"),
            ),
            ("web/routes", include_str!("../templates/web/routes.jinja")),
            (
                "web/routes_declarations",
                include_str!("../templates/web/routes_declarations.jinja"),
            ),
            (
                "web/link-transform",
                include_str!("../templates/web/link-transform.jinja"),
            ),
            (
                "web/link-transform_declarations",
                include_str!("../templates/web/link-transform_declarations.jinja"),
            ),
            (
                "web/runtime-links",
                include_str!("../templates/web/runtime-links.jinja"),
            ),
            (
                "web/runtime-links_declarations",
                include_str!("../templates/web/runtime-links_declarations.jinja"),
            ),
            (
                "web/server-cookie",
                include_str!("../templates/web/server-cookie.jinja"),
            ),
            (
                "web/server-cookie_declarations",
                include_str!("../templates/web/server-cookie_declarations.jinja"),
            ),
            (
                "web/switch-route",
                include_str!("../templates/web/switch-route.jinja"),
            ),
            (
                "web/switch-route_declarations",
                include_str!("../templates/web/switch-route_declarations.jinja"),
            ),
            ("shared", include_str!("../templates/shared.jinja")),
            (
                "shared_declarations",
                include_str!("../templates/shared_declarations.jinja"),
            ),
        ] {
            environment
                .add_template(name, source)
                .expect("valid embedded codegen template");
        }
        environment
    })
}

pub(crate) fn render(name: &str, values: Value) -> String {
    environment()
        .get_template(name)
        .expect("registered codegen template")
        .render(values)
        .expect("complete validated codegen template context")
}

/// Render an unmapped helper module from its canonical build-time template.
pub(crate) fn module(
    output: crate::ecmascript::EcmaModuleOutput,
    name: &str,
    values: Value,
) -> crate::ecmascript::EcmaModule {
    let mut module = crate::ecmascript::EcmaModule::new(output);
    module.push_statement(crate::ecmascript::EcmaStatement::generated(render(
        name, values,
    )));
    module
}
