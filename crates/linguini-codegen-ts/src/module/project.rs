use super::templates::{
    render_template, SVELTEKIT_CONTROL_DECLARATIONS, SVELTEKIT_CONTROL_RUNTIME,
    SVELTEKIT_DECLARATIONS, SVELTEKIT_RUNTIME,
};
use super::web_runtime_links;
use super::{web_accept_language, web_cookie, web_link_transform, web_local_storage, web_path};
use super::{TypeScriptLocaleSource, TypeScriptWebOptions};

pub fn generate_project_svelte_locale_module(web: bool, sveltekit: bool) -> String {
    super::svelte_locale::generate_typescript_svelte_locale_module(
        super::svelte_locale::SvelteLocaleMode::from_framework(web, sveltekit),
    )
}

pub fn generate_project_svelte_locale_declaration(web: bool, sveltekit: bool) -> String {
    super::svelte_locale::generate_svelte_locale_declaration(
        super::svelte_locale::SvelteLocaleMode::from_framework(web, sveltekit),
    )
}

pub fn generate_project_svelte_effects_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> String {
    super::svelte_effects::generate_typescript_svelte_effects_module(options, sveltekit)
}

pub fn generate_project_svelte_effects_declaration() -> String {
    super::svelte_effects::generate_svelte_effects_declaration()
}

pub fn generate_project_svelte_module(
    options: Option<&TypeScriptWebOptions>,
    _sveltekit: bool,
) -> String {
    super::svelte_facade::generate_typescript_svelte_module(options.is_some())
}

pub fn generate_project_svelte_control_module_with_options(
    sveltekit: bool,
    options: &TypeScriptWebOptions,
) -> String {
    super::svelte_control::generate_typescript_svelte_control_module(options, sveltekit)
}

pub fn generate_project_svelte_declaration(web: bool, _sveltekit: bool) -> String {
    super::svelte_facade_types::generate_svelte_declaration(web)
}

pub fn generate_project_svelte_control_declaration(sveltekit: bool) -> String {
    super::svelte_control_types::generate_svelte_control_declaration(sveltekit)
}

pub fn generate_project_sveltekit_module(options: &TypeScriptWebOptions) -> String {
    render_template(SVELTEKIT_RUNTIME, &sveltekit_replacements(options))
}

pub fn generate_project_sveltekit_declaration() -> String {
    SVELTEKIT_DECLARATIONS.to_owned()
}

pub fn generate_project_sveltekit_control_module(options: &TypeScriptWebOptions) -> String {
    render_template(SVELTEKIT_CONTROL_RUNTIME, &sveltekit_replacements(options))
}

fn sveltekit_replacements(options: &TypeScriptWebOptions) -> Vec<(&'static str, String)> {
    let has_cookie = options.features().has_cookie;
    vec![
        ("OPTIONS", web_options_literal(options)),
        (
            "SERVER_COOKIE_IMPORT",
            if has_cookie {
                "import { persistLocaleCookie } from \"./web/server-cookie.js\";".to_owned()
            } else {
                String::new()
            },
        ),
        (
            "PERSIST_COOKIE_DECLARATION",
            if has_cookie {
                "  const persistCookie = options.persistCookie !== false && web.options.locale.switch.writesCookie;"
                    .to_owned()
            } else {
                String::new()
            },
        ),
        (
            "COOKIE_INPUT",
            if has_cookie {
                "      cookie: event.request.headers.get(\"cookie\") ?? undefined,".to_owned()
            } else {
                String::new()
            },
        ),
        (
            "PERSIST_REDIRECT_COOKIE",
            if has_cookie {
                "      if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });"
                    .to_owned()
            } else {
                String::new()
            },
        ),
        (
            "PERSIST_RESPONSE_COOKIE",
            if has_cookie {
                "    if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });"
                    .to_owned()
            } else {
                String::new()
            },
        ),
        (
            "SWITCH_ROUTE_IMPORT",
            if options.switch_route.is_some() {
                "import { handleLocaleSwitchRoute } from \"./web/switch-route.js\";".to_owned()
            } else {
                String::new()
            },
        ),
        (
            "SWITCH_ROUTE_BRANCH",
            if options.switch_route.is_some() {
                "    const switchResponse = handleLocaleSwitchRoute(web, event);\n    if (switchResponse) return switchResponse;"
                    .to_owned()
            } else {
                String::new()
            },
        ),
    ]
}

pub fn generate_project_sveltekit_control_declaration() -> String {
    SVELTEKIT_CONTROL_DECLARATIONS.to_owned()
}

pub fn generate_project_web_module_with_options(options: &TypeScriptWebOptions) -> String {
    super::web_facade::generate_typescript_web_module(options)
}

pub fn generate_project_web_source_module(source: TypeScriptLocaleSource) -> String {
    match source {
        TypeScriptLocaleSource::Path => web_path::generate_typescript_web_path_module(),
        TypeScriptLocaleSource::Cookie => web_cookie::generate_typescript_web_cookie_module(),
        TypeScriptLocaleSource::LocalStorage => {
            web_local_storage::generate_typescript_web_local_storage_module()
        }
        TypeScriptLocaleSource::AcceptLanguage => {
            web_accept_language::generate_typescript_web_accept_language_module()
        }
    }
}

pub fn generate_project_web_source_declaration(source: TypeScriptLocaleSource) -> String {
    match source {
        TypeScriptLocaleSource::Path => web_path::generate_web_path_declaration(),
        TypeScriptLocaleSource::Cookie => web_cookie::generate_web_cookie_declaration(),
        TypeScriptLocaleSource::LocalStorage => {
            web_local_storage::generate_web_local_storage_declaration()
        }
        TypeScriptLocaleSource::AcceptLanguage => {
            web_accept_language::generate_web_accept_language_declaration()
        }
    }
}

pub fn generate_project_web_link_module(mode: super::TypeScriptLinkMode) -> Option<String> {
    match mode {
        super::TypeScriptLinkMode::Transform => {
            Some(web_link_transform::generate_typescript_web_link_transform_module())
        }
        super::TypeScriptLinkMode::Runtime => {
            Some(web_runtime_links::generate_typescript_web_runtime_links_module())
        }
        super::TypeScriptLinkMode::Manual => None,
    }
}

pub fn generate_project_web_link_declaration(mode: super::TypeScriptLinkMode) -> Option<String> {
    match mode {
        super::TypeScriptLinkMode::Transform => {
            Some(web_link_transform::generate_web_link_transform_declaration())
        }
        super::TypeScriptLinkMode::Runtime => {
            Some(web_runtime_links::generate_web_runtime_links_declaration())
        }
        super::TypeScriptLinkMode::Manual => None,
    }
}

pub fn generate_project_web_server_cookie_module() -> String {
    super::web_server_cookie::generate_typescript_web_server_cookie_module()
}

pub fn generate_project_web_server_cookie_declaration() -> String {
    super::web_server_cookie::generate_web_server_cookie_declaration()
}

pub fn generate_project_web_switch_route_module(options: &TypeScriptWebOptions) -> Option<String> {
    super::web_switch_route::generate_typescript_web_switch_route_module(options)
}

pub fn generate_project_web_switch_route_declaration() -> String {
    super::web_switch_route::generate_web_switch_route_declaration()
}

pub fn generate_project_web_declaration() -> String {
    super::web_types::generate_web_declaration()
}

fn web_options_literal(options: &TypeScriptWebOptions) -> String {
    super::web_options::web_options_literal(
        options,
        crate::ecmascript::EcmaScriptTarget::TypeScript,
    )
}
