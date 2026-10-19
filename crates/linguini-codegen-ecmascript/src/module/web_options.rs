//! Semantic context for the closed web policy template.

use super::EcmaWebOptions;

pub(super) fn web_options_literal(options: &EcmaWebOptions) -> String {
    let features = options.features();
    crate::templates::render(
        "web/options",
        minijinja::context! {
            locale_prefix => options.locale_prefix.as_str(),
            canonical => if options.canonical_redirect { "redirect" } else { "preserve" },
            sources => options.sources.iter().map(|source| source.as_str()).collect::<Vec<_>>(),
            writes_path => options.locale_switch.writes_path,
            writes_cookie => options.locale_switch.writes_cookie,
            writes_storage => options.locale_switch.writes_local_storage,
            links => features.link_mode.as_str(), exclude => &options.exclude,
            has_cookie => features.has_cookie, cookie_name => &options.cookie_name,
            cookie_path => options.cookie_path.as_deref().unwrap_or("auto"),
            cookie_domain => &options.cookie_domain, cookie_max_age => options.cookie_max_age,
            same_site => &options.cookie_same_site, secure => options.cookie_secure,
            http_only => options.cookie_http_only,
            has_storage => features.has_local_storage, storage_key => &options.local_storage_key,
        },
    )
}
