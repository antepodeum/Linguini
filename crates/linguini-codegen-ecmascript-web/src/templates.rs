use linguini_codegen_ecmascript::EcmaTemplateEngine;
use minijinja::Value;
use std::sync::OnceLock;

pub(crate) fn render(name: &str, values: Value) -> String {
    static ENGINE: OnceLock<EcmaTemplateEngine> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            EcmaTemplateEngine::new([
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
                    "web/options",
                    include_str!("../templates/web/options.jinja"),
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
            ])
        })
        .render(name, values)
}

pub(crate) fn module(
    output: linguini_codegen_ecmascript::EcmaModuleOutput,
    name: &str,
    values: Value,
) -> linguini_codegen_ecmascript::EcmaModule {
    let mut module = linguini_codegen_ecmascript::EcmaModule::new(output);
    module.push_statement(linguini_codegen_ecmascript::EcmaStatement::generated(
        render(name, values),
    ));
    module
}
