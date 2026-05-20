use super::names::{escape_string, property_key, safe_identifier};
use super::templates::{
    render_template, INDEX_RUNTIME, INDEX_RUNTIME_DECLARATIONS, PROJECT_INDEX_DECLARATIONS,
    PROJECT_INDEX_ENTRY, SVELTEKIT_DECLARATIONS, SVELTEKIT_RUNTIME, SVELTE_DECLARATIONS,
    SVELTE_RUNTIME, WEB_DECLARATIONS, WEB_RUNTIME,
};
use super::{TypeScriptLocaleModule, TypeScriptWebOptions};
use linguini_cldr::built_in_text_direction;

pub fn generate_project_index(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    render_template(
        PROJECT_INDEX_ENTRY,
        &[
            ("IMPORTS", project_locale_imports(locales)),
            ("LOCALES", locale_literals(locales).join(", ")),
            ("BASE_LOCALE", base_locale_literal(locales, base_locale)),
            ("LOCALE_DIRECTIONS", project_locale_directions(locales)),
            ("LOCALE_MODULES", project_locale_modules(locales)),
            ("LOCALE_LOADERS", project_locale_loaders(locales)),
            ("INDEX_RUNTIME", template_body(INDEX_RUNTIME)),
        ],
    )
}

pub fn generate_project_index_declaration(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    render_template(
        PROJECT_INDEX_DECLARATIONS,
        &[
            ("IMPORTS", project_locale_imports(locales)),
            ("LOCALES", locale_literals(locales).join(", ")),
            ("BASE_LOCALE", base_locale_literal(locales, base_locale)),
            (
                "LOCALE_DIRECTIONS",
                project_locale_direction_declarations(locales),
            ),
            (
                "LOCALE_MODULES",
                project_locale_module_declarations(locales),
            ),
            (
                "LOCALE_LOADERS",
                project_locale_loader_declarations(locales),
            ),
            (
                "INDEX_RUNTIME_DECLARATIONS",
                template_body(INDEX_RUNTIME_DECLARATIONS),
            ),
        ],
    )
}

pub fn generate_project_svelte_module(options: &TypeScriptWebOptions) -> String {
    render_template(SVELTE_RUNTIME, &[("OPTIONS", web_options_literal(options))])
}

pub fn generate_project_svelte_declaration() -> String {
    SVELTE_DECLARATIONS.to_owned()
}

pub fn generate_project_sveltekit_module(options: &TypeScriptWebOptions) -> String {
    render_template(
        SVELTEKIT_RUNTIME,
        &[("OPTIONS", web_options_literal(options))],
    )
}

pub fn generate_project_sveltekit_declaration() -> String {
    SVELTEKIT_DECLARATIONS.to_owned()
}

pub fn generate_project_web_module() -> String {
    WEB_RUNTIME.to_owned()
}

pub fn generate_project_web_declaration() -> String {
    WEB_DECLARATIONS.to_owned()
}

fn template_body(template: &str) -> String {
    template.strip_suffix('\n').unwrap_or(template).to_owned()
}

fn project_locale_imports(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "import {} from \"./locales/{}\";",
                locale_identifier(&locale.locale),
                escape_string(&locale.locale)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn project_locale_directions(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  {}: \"{}\",\n",
                property_key(&locale.locale),
                locale_direction(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_direction_declarations(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  readonly {}: \"{}\";\n",
                property_key(&locale.locale),
                locale_direction(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_modules(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  {}: {},\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_module_declarations(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  readonly {}: typeof {};\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_loaders(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  {}: () => Promise.resolve({}),\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_loader_declarations(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  readonly {}: () => Promise<typeof {}>;\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}

fn locale_literals(locales: &[TypeScriptLocaleModule]) -> Vec<String> {
    locales
        .iter()
        .map(|locale| format!("\"{}\"", escape_string(&locale.locale)))
        .collect()
}

fn base_locale_literal(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    let selected = base_locale
        .filter(|base_locale| locales.iter().any(|locale| locale.locale == *base_locale))
        .or_else(|| locales.first().map(|locale| locale.locale.as_str()));

    selected
        .map(|locale| format!("\"{}\"", escape_string(locale)))
        .unwrap_or_else(|| "\"\" as LinguiniLanguageInput".to_owned())
}

fn web_options_literal(options: &TypeScriptWebOptions) -> String {
    let strategy = js_string_array(&options.strategy);
    let exclude = js_string_array(&options.exclude);
    let mut fields = vec![
        format!("strategy: [{strategy}] as const"),
        format!("cookieName: \"{}\"", escape_string(&options.cookie_name)),
        format!("cookiePath: \"{}\"", escape_string(&options.cookie_path)),
        format!("cookieMaxAge: {}", options.cookie_max_age),
        format!(
            "cookieSameSite: \"{}\"",
            escape_string(&options.cookie_same_site)
        ),
        format!("cookieSecure: {}", js_bool(options.cookie_secure)),
        format!("cookieHttpOnly: {}", js_bool(options.cookie_http_only)),
        format!(
            "localStorageKey: \"{}\"",
            escape_string(&options.local_storage_key)
        ),
        format!(
            "prefixDefaultLocale: {}",
            js_bool(options.prefix_default_locale)
        ),
        format!("basePath: \"{}\"", escape_string(&options.base_path)),
        format!(
            "trailingSlash: \"{}\"",
            escape_string(&options.trailing_slash)
        ),
        format!("redirect: {}", js_bool(options.redirect)),
        format!("exclude: [{exclude}] as const"),
        format!("localizeLinks: {}", js_bool(options.localize_links)),
    ];

    if let Some(cookie_domain) = &options.cookie_domain {
        fields.push(format!(
            "cookieDomain: \"{}\"",
            escape_string(cookie_domain)
        ));
    }
    if let Some(global_variable_name) = &options.global_variable_name {
        fields.push(format!(
            "globalVariableName: \"{}\"",
            escape_string(global_variable_name)
        ));
    }
    if let Some(origin) = &options.origin {
        fields.push(format!("origin: \"{}\"", escape_string(origin)));
    }

    format!("{{ {} }} as const", fields.join(", "))
}

fn js_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|item| format!("\"{}\"", escape_string(item)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn js_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn locale_direction(locale: &str) -> &'static str {
    built_in_text_direction(locale).unwrap_or("ltr")
}
