//! Web module options are decoded and validated by their generator, outside neutral TOML.

use serde::Deserialize;
use serde_json::Value;

use crate::{
    validate_web_options, EcmaLinkMode, EcmaLocalePrefixMode, EcmaLocaleSource,
    EcmaLocaleSwitchPlan, EcmaWebOptions, EcmaWebSwitchRoute, WebCodegenError,
};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct WebOptions {
    routing: Routing,
    locale: Locale,
    cookie: Option<Cookie>,
    local_storage: Option<Storage>,
    links: Links,
    routes: Routes,
    switch_route: Option<SwitchRoute>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Routing {
    locale_prefix: Option<EcmaLocalePrefixMode>,
    canonical: Option<String>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Locale {
    sources: Option<Vec<EcmaLocaleSource>>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Cookie {
    name: Option<String>,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<String>,
    same_site: Option<String>,
    secure: Option<Value>,
    http_only: Option<bool>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Storage {
    key: Option<String>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Links {
    mode: Option<EcmaLinkMode>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Routes {
    exclude: Vec<String>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct SwitchRoute {
    path: Option<String>,
    return_query: Option<String>,
    status: Option<u16>,
}

fn invalid(reason: impl Into<String>) -> WebCodegenError {
    WebCodegenError::InvalidModuleOptions {
        reason: reason.into(),
    }
}

/// Lowers one module's serializable options into closed generator capabilities.
pub fn parse_module_options(
    value: &Value,
    project_name: &str,
) -> Result<EcmaWebOptions, WebCodegenError> {
    let raw: WebOptions =
        serde_json::from_value(value.clone()).map_err(|error| invalid(error.to_string()))?;
    let locale_prefix = raw.routing.locale_prefix.unwrap_or_default();
    let canonical_redirect = match raw.routing.canonical.as_deref().unwrap_or("redirect") {
        "redirect" => true,
        "preserve" => false,
        value => return Err(invalid(format!("unknown canonical policy `{value}`"))),
    };
    let sources = raw.locale.sources.unwrap_or_else(|| {
        if locale_prefix == EcmaLocalePrefixMode::Never {
            vec![EcmaLocaleSource::Cookie, EcmaLocaleSource::AcceptLanguage]
        } else {
            vec![
                EcmaLocaleSource::Path,
                EcmaLocaleSource::Cookie,
                EcmaLocaleSource::AcceptLanguage,
            ]
        }
    });
    let has_cookie = sources.contains(&EcmaLocaleSource::Cookie);
    let has_storage = sources.contains(&EcmaLocaleSource::LocalStorage);
    if !has_cookie && raw.cookie.is_some() {
        return Err(invalid("cookie options require the cookie locale source"));
    }
    if !has_storage && raw.local_storage.is_some() {
        return Err(invalid(
            "localStorage options require the local-storage locale source",
        ));
    }
    let cookie = raw.cookie.unwrap_or_default();
    let cookie_path = cookie.path.filter(|value| value != "auto");
    let cookie_secure = match cookie.secure {
        None => None,
        Some(Value::String(value)) if value == "auto" => None,
        Some(Value::Bool(value)) => Some(value),
        _ => return Err(invalid("cookie.secure must be true, false, or auto")),
    };
    let options = EcmaWebOptions {
        locale_switch: EcmaLocaleSwitchPlan {
            writes_path: sources.contains(&EcmaLocaleSource::Path),
            writes_cookie: has_cookie,
            writes_local_storage: has_storage,
        },
        sources,
        cookie_name: if has_cookie {
            cookie
                .name
                .unwrap_or_else(|| format!("LINGUINI_{}_LOCALE", project_key(project_name, true)))
        } else {
            "LINGUINI_LOCALE".to_owned()
        },
        cookie_path,
        cookie_domain: cookie.domain,
        cookie_max_age: parse_duration(cookie.max_age.as_deref().unwrap_or("365d"))?,
        cookie_same_site: cookie.same_site.unwrap_or_else(|| "lax".to_owned()),
        cookie_secure,
        cookie_http_only: cookie.http_only.unwrap_or(false),
        local_storage_key: if has_storage {
            raw.local_storage
                .unwrap_or_default()
                .key
                .unwrap_or_else(|| format!("linguini:{}:locale", project_key(project_name, false)))
        } else {
            "LINGUINI_LOCALE".to_owned()
        },
        locale_prefix,
        canonical_redirect,
        exclude: raw.routes.exclude,
        link_mode: raw.links.mode.unwrap_or(EcmaLinkMode::Transform),
        switch_route: raw.switch_route.map(|route| EcmaWebSwitchRoute {
            path: route
                .path
                .unwrap_or_else(|| "/_linguini/locale/{locale}".to_owned()),
            return_query: route.return_query.unwrap_or_else(|| "return".to_owned()),
            status: route.status.unwrap_or(303),
        }),
    };
    validate_web_options(&options)?;
    validate_policy(&options)?;
    Ok(options)
}

fn project_key(name: &str, environment: bool) -> String {
    let separator = if environment { '_' } else { '-' };
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                if environment {
                    character.to_ascii_uppercase()
                } else {
                    character.to_ascii_lowercase()
                }
            } else {
                separator
            }
        })
        .collect::<String>()
        .trim_matches(separator)
        .to_owned()
}

fn parse_duration(value: &str) -> Result<u64, WebCodegenError> {
    let split = value
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(value.len());
    let (amount, unit) = value.split_at(split);
    let amount = amount
        .parse::<u64>()
        .map_err(|_| invalid("invalid cookie.maxAge duration"))?;
    let multiplier = match unit {
        "" | "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        "w" => 604800,
        _ => {
            return Err(invalid(
                "cookie.maxAge requires seconds, minutes, hours, days, or weeks",
            ))
        }
    };
    amount
        .checked_mul(multiplier)
        .filter(|amount| *amount > 0)
        .ok_or_else(|| invalid("cookie.maxAge must be positive and fit in u64 seconds"))
}

fn validate_policy(options: &EcmaWebOptions) -> Result<(), WebCodegenError> {
    if options.locale_prefix == EcmaLocalePrefixMode::Never && options.locale_switch.writes_path {
        return Err(invalid("path locale source requires a URL locale prefix"));
    }
    if options.locale_switch.writes_cookie {
        if options.cookie_name.is_empty()
            || options
                .cookie_name
                .bytes()
                .any(|byte| byte <= 0x20 || byte >= 0x7f || b"()<>@,;:\\\"/[]?={}".contains(&byte))
        {
            return Err(invalid("invalid cookie.name token"));
        }
        if let Some(path) = &options.cookie_path {
            if !path.starts_with('/')
                || path
                    .bytes()
                    .any(|byte| byte < 0x20 || byte == 0x7f || byte == b';')
            {
                return Err(invalid("cookie.path must be a safe absolute path or auto"));
            }
        }
        if let Some(domain) = &options.cookie_domain {
            if domain.is_empty()
                || domain.starts_with('.')
                || domain.ends_with('.')
                || domain
                    .bytes()
                    .any(|byte| !(byte.is_ascii_alphanumeric() || b".-".contains(&byte)))
            {
                return Err(invalid("invalid cookie.domain"));
            }
        }
        if !matches!(options.cookie_same_site.as_str(), "lax" | "strict" | "none") {
            return Err(invalid("unknown cookie.sameSite policy"));
        }
        if options.cookie_same_site == "none" && options.cookie_secure != Some(true) {
            return Err(invalid("SameSite=None requires cookie.secure = true"));
        }
        if options.cookie_http_only {
            return Err(invalid(
                "cookie.httpOnly must be false while browser locale switching is enabled",
            ));
        }
    }
    if options.locale_switch.writes_local_storage
        && (options.local_storage_key.is_empty()
            || options.local_storage_key.chars().any(char::is_control))
    {
        return Err(invalid("invalid localStorage.key"));
    }
    if options.exclude.iter().any(|pattern| {
        !pattern.starts_with('/') || pattern.starts_with("//") || pattern.contains(['?', '#'])
    }) {
        return Err(invalid("route exclusions must be absolute path patterns"));
    }
    if let Some(route) = &options.switch_route {
        if !options.locale_switch.writes_path && !options.locale_switch.writes_cookie {
            return Err(invalid(
                "switchRoute requires the path or cookie locale source",
            ));
        }
        if !route.path.starts_with('/')
            || route.path.starts_with("//")
            || route.path.matches("{locale}").count() != 1
            || route.path.contains(['?', '#'])
            || route.path.split('/').any(|segment| segment == "..")
        {
            return Err(invalid(
                "switchRoute.path requires one {locale} marker in a safe absolute path",
            ));
        }
        if route.return_query.is_empty()
            || !route
                .return_query
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        {
            return Err(invalid("invalid switchRoute.returnQuery"));
        }
        if !matches!(route.status, 301 | 302 | 303 | 307 | 308) {
            return Err(invalid("switchRoute.status must be a redirect status"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn module_options_preserve_nested_policy_and_closed_switch_capabilities() {
        let options = parse_module_options(&json!({
            "routing": {"localePrefix": "always", "canonical": "preserve"},
            "locale": {"sources": ["path", "cookie", "local-storage", "accept-language"]},
            "cookie": {"name": "SHOP_LOCALE", "path": "/shop", "domain": "example.com", "maxAge": "2w", "sameSite": "none", "secure": true},
            "localStorage": {"key": "shop.locale"},
            "links": {"mode": "manual"},
            "routes": {"exclude": ["/admin/**"]},
            "switchRoute": {"path": "/language/{locale}", "returnQuery": "next", "status": 307}
        }), "shop").unwrap();
        assert_eq!(options.locale_prefix, EcmaLocalePrefixMode::Always);
        assert!(!options.canonical_redirect);
        assert_eq!(
            options.sources,
            [
                EcmaLocaleSource::Path,
                EcmaLocaleSource::Cookie,
                EcmaLocaleSource::LocalStorage,
                EcmaLocaleSource::AcceptLanguage
            ]
        );
        assert_eq!(options.cookie_max_age, 14 * 24 * 3600);
        assert_eq!(options.cookie_name, "SHOP_LOCALE");
        assert_eq!(options.cookie_path.as_deref(), Some("/shop"));
        assert_eq!(options.cookie_domain.as_deref(), Some("example.com"));
        assert_eq!(options.cookie_secure, Some(true));
        assert_eq!(options.cookie_same_site, "none");
        assert_eq!(options.local_storage_key, "shop.locale");
        assert_eq!(options.link_mode, EcmaLinkMode::Manual);
        assert_eq!(options.exclude, ["/admin/**"]);
        let route = options.switch_route.unwrap();
        assert_eq!(
            (
                route.path.as_str(),
                route.return_query.as_str(),
                route.status
            ),
            ("/language/{locale}", "next", 307)
        );
        assert_eq!(
            options.locale_switch,
            EcmaLocaleSwitchPlan {
                writes_path: true,
                writes_cookie: true,
                writes_local_storage: true
            }
        );
    }

    #[test]
    fn module_defaults_follow_selected_sources_and_project_identity() {
        let defaults = parse_module_options(&json!({}), "My Shop").unwrap();
        assert_eq!(defaults.cookie_name, "LINGUINI_MY_SHOP_LOCALE");
        assert_eq!(defaults.cookie_path, None);
        assert_eq!(defaults.cookie_secure, None);
        assert_eq!(defaults.link_mode, EcmaLinkMode::Transform);
        let pathless =
            parse_module_options(&json!({"routing":{"localePrefix":"never"}}), "shop").unwrap();
        assert_eq!(
            pathless.sources,
            [EcmaLocaleSource::Cookie, EcmaLocaleSource::AcceptLanguage]
        );
        let storage =
            parse_module_options(&json!({"locale":{"sources":["local-storage"]}}), "My Shop")
                .unwrap();
        assert_eq!(storage.local_storage_key, "linguini:my-shop:locale");
        let empty = parse_module_options(&json!({"locale":{"sources":[]}}), "shop").unwrap();
        assert!(empty.sources.is_empty());
        assert_eq!(
            empty.locale_switch,
            EcmaLocaleSwitchPlan {
                writes_path: false,
                writes_cookie: false,
                writes_local_storage: false
            }
        );
    }

    #[test]
    fn module_policy_rejects_unknown_impossible_or_unsafe_options() {
        for value in [
            json!({"unknown": true}),
            json!({"routing":{"localePrefix":"sometimes"}}),
            json!({"routing":{"canonical":"sometimes"}}),
            json!({"links":{"mode":"auto"}}),
            json!({"locale":{"sources":["preferredLanguage"]}}),
            json!({"locale":{"sources":["cookie","cookie"]}}),
            json!({"routing":{"localePrefix":"never"},"locale":{"sources":["path"]}}),
            json!({"locale":{"sources":[]},"cookie":{}}),
            json!({"locale":{"sources":[]},"localStorage":{}}),
            json!({"locale":{"sources":[]},"switchRoute":{}}),
            json!({"cookie":{"name":"bad;name"}}),
            json!({"cookie":{"path":"relative"}}),
            json!({"cookie":{"path":"/bad;path"}}),
            json!({"cookie":{"domain":".example.com"}}),
            json!({"cookie":{"domain":"bad domain"}}),
            json!({"cookie":{"sameSite":"bad"}}),
            json!({"cookie":{"sameSite":"none"}}),
            json!({"cookie":{"sameSite":"none","secure":"auto"}}),
            json!({"cookie":{"secure":"sometimes"}}),
            json!({"cookie":{"httpOnly":true}}),
            json!({"cookie":{"maxAge":"0d"}}),
            json!({"cookie":{"maxAge":"1ms"}}),
            json!({"cookie":{"maxAge":"18446744073709551615w"}}),
            json!({"locale":{"sources":["local-storage"]},"localStorage":{"key":""}}),
            json!({"routes":{"exclude":["//host/path"]}}),
            json!({"routes":{"exclude":["/path?query"]}}),
            json!({"switchRoute":{"path":"/no-marker"}}),
            json!({"switchRoute":{"path":"/../{locale}"}}),
            json!({"switchRoute":{"returnQuery":"bad&query"}}),
            json!({"switchRoute":{"status":200}}),
        ] {
            assert!(parse_module_options(&value, "shop").is_err(), "{value}");
        }
        for (value, expected) in [
            ("30", 30),
            ("2s", 2),
            ("3m", 180),
            ("4h", 14400),
            ("2d", 172800),
            ("2w", 1209600),
        ] {
            assert_eq!(parse_duration(value).unwrap(), expected);
        }
    }
}
