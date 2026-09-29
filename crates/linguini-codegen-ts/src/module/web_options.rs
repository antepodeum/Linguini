//! Target-aware serialization of the closed web policy.

use super::names::escape_string;
use super::{TypeScriptLocaleSource, TypeScriptLocaleSwitchPlan, TypeScriptWebOptions};
use crate::ecmascript::EcmaScriptTarget;

pub(super) fn web_options_literal(
    options: &TypeScriptWebOptions,
    target: EcmaScriptTarget,
) -> String {
    let features = options.features();
    let as_const = const_annotation(target);
    let sources = js_locale_source_array(&options.sources);
    let locale_switch = locale_switch_literal(options.locale_switch, target);
    let exclude = js_string_array(&options.exclude);
    let mut fields = vec![
        format!(
            "routing: {{ localePrefix: \"{}\", canonical: \"{}\" }}",
            options.locale_prefix.as_str(),
            if options.canonical_redirect {
                "redirect"
            } else {
                "preserve"
            }
        ),
        format!("locale: {{ sources: [{sources}]{as_const}, switch: {locale_switch} }}"),
        format!("links: {{ mode: \"{}\" }}", features.link_mode.as_str()),
        format!("routes: {{ exclude: [{exclude}]{as_const} }}"),
    ];

    if features.has_cookie {
        let mut cookie = vec![
            format!("name: \"{}\"", escape_string(&options.cookie_name)),
            options.cookie_path.as_ref().map_or_else(
                || "path: \"auto\"".to_owned(),
                |path| format!("path: \"{}\"", escape_string(path)),
            ),
            format!("maxAge: {}", options.cookie_max_age),
            format!("sameSite: \"{}\"", escape_string(&options.cookie_same_site)),
            options.cookie_secure.map_or_else(
                || "secure: \"auto\"".to_owned(),
                |secure| format!("secure: {}", js_bool(secure)),
            ),
            format!("httpOnly: {}", js_bool(options.cookie_http_only)),
        ];
        if let Some(cookie_domain) = &options.cookie_domain {
            cookie.push(format!("domain: \"{}\"", escape_string(cookie_domain)));
        }
        fields.push(format!("cookie: {{ {} }}", cookie.join(", ")));
    }
    if features.has_local_storage {
        fields.push(format!(
            "localStorage: {{ key: \"{}\" }}",
            escape_string(&options.local_storage_key)
        ));
    }

    format!("{{ {} }}{as_const}", fields.join(", "))
}

fn locale_switch_literal(plan: TypeScriptLocaleSwitchPlan, target: EcmaScriptTarget) -> String {
    let as_const = const_annotation(target);
    format!(
        "{{ writesPath: {}, writesCookie: {}, writesLocalStorage: {} }}{as_const}",
        js_bool(plan.writes_path),
        js_bool(plan.writes_cookie),
        js_bool(plan.writes_local_storage),
    )
}

fn js_locale_source_array(values: &[TypeScriptLocaleSource]) -> String {
    values
        .iter()
        .map(|source| format!("\"{}\"", source.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
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

fn const_annotation(target: EcmaScriptTarget) -> &'static str {
    if target.is_typescript() {
        " as const"
    } else {
        ""
    }
}
