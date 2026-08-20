pub const INDEX_RUNTIME: &str = include_str!("templates/index.runtime.ts");
pub const INDEX_RUNTIME_DECLARATIONS: &str = include_str!("templates/index.runtime.d.ts");
pub const LOCALE_RUNTIME: &str = include_str!("templates/locale.runtime.ts");
pub const LOCALE_DECLARATIONS: &str = include_str!("templates/locale.runtime.d.ts");
pub const PROJECT_INDEX_ENTRY: &str = include_str!("templates/project-index.entry.ts");
pub const PROJECT_INDEX_DECLARATIONS: &str = include_str!("templates/project-index.entry.d.ts");
pub const SHARED_RUNTIME: &str = include_str!("templates/shared.runtime.ts");
pub const SHARED_DECLARATIONS: &str = include_str!("templates/shared.runtime.d.ts");
#[cfg(test)]
pub const SINGLE_INDEX_RUNTIME: &str = include_str!("templates/single-index.runtime.ts");
#[cfg(test)]
pub const SINGLE_INDEX_DECLARATIONS: &str = include_str!("templates/single-index.runtime.d.ts");
pub const SVELTE_RUNTIME: &str = include_str!("templates/svelte.runtime.ts");
pub const SVELTE_DECLARATIONS: &str = include_str!("templates/svelte.runtime.d.ts");
pub const SVELTE_CONTROL_RUNTIME: &str = include_str!("templates/svelte-control.runtime.ts");
pub const SVELTE_CONTROL_DECLARATIONS: &str = include_str!("templates/svelte-control.runtime.d.ts");
pub const SVELTE_CONTEXT_RUNTIME: &str = include_str!("templates/svelte.context.runtime.ts");
pub const SVELTE_CONTEXT_DECLARATIONS: &str = include_str!("templates/svelte.context.runtime.d.ts");
pub const SVELTE_EFFECTS_RUNTIME: &str = include_str!("templates/svelte-effects.runtime.ts");
pub const SVELTE_EFFECTS_DECLARATIONS: &str = include_str!("templates/svelte-effects.runtime.d.ts");
pub const SVELTE_LOCALE_RUNTIME: &str = include_str!("templates/svelte-locale.runtime.ts");
pub const SVELTE_LOCALE_CONTEXT_RUNTIME: &str =
    include_str!("templates/svelte-locale.context.runtime.ts");
pub const SVELTE_LOCALE_STANDALONE_RUNTIME: &str =
    include_str!("templates/svelte-locale.standalone.runtime.ts");
pub const SVELTE_LOCALE_DECLARATIONS: &str = include_str!("templates/svelte-locale.runtime.d.ts");
pub const SVELTE_LOCALE_CONTEXT_DECLARATIONS: &str =
    include_str!("templates/svelte-locale.context.runtime.d.ts");
pub const SVELTE_LOCALE_STANDALONE_DECLARATIONS: &str =
    include_str!("templates/svelte-locale.standalone.runtime.d.ts");
pub const SVELTEKIT_RUNTIME: &str = include_str!("templates/sveltekit.runtime.ts");
pub const SVELTEKIT_DECLARATIONS: &str = include_str!("templates/sveltekit.runtime.d.ts");
pub const SVELTEKIT_CONTROL_RUNTIME: &str = include_str!("templates/sveltekit-control.runtime.ts");
pub const SVELTEKIT_CONTROL_DECLARATIONS: &str =
    include_str!("templates/sveltekit-control.runtime.d.ts");
pub const WEB_RUNTIME: &str = include_str!("templates/web.runtime.ts");
pub const WEB_DECLARATIONS: &str = include_str!("templates/web.runtime.d.ts");
pub const WEB_PATH_RUNTIME: &str = include_str!("templates/web.path.runtime.ts");
pub const WEB_PATH_DECLARATIONS: &str = include_str!("templates/web.path.runtime.d.ts");
pub const WEB_COOKIE_RUNTIME: &str = include_str!("templates/web.cookie.runtime.ts");
pub const WEB_COOKIE_DECLARATIONS: &str = include_str!("templates/web.cookie.runtime.d.ts");
pub const WEB_LOCAL_STORAGE_RUNTIME: &str = include_str!("templates/web.local-storage.runtime.ts");
pub const WEB_LOCAL_STORAGE_DECLARATIONS: &str =
    include_str!("templates/web.local-storage.runtime.d.ts");
pub const WEB_ACCEPT_LANGUAGE_RUNTIME: &str =
    include_str!("templates/web.accept-language.runtime.ts");
pub const WEB_ACCEPT_LANGUAGE_DECLARATIONS: &str =
    include_str!("templates/web.accept-language.runtime.d.ts");
pub const WEB_LINK_TRANSFORM_RUNTIME: &str =
    include_str!("templates/web.link-transform.runtime.ts");
pub const WEB_LINK_TRANSFORM_DECLARATIONS: &str =
    include_str!("templates/web.link-transform.runtime.d.ts");
pub const WEB_RUNTIME_LINKS_RUNTIME: &str = include_str!("templates/web.runtime-links.runtime.ts");
pub const WEB_RUNTIME_LINKS_DECLARATIONS: &str =
    include_str!("templates/web.runtime-links.runtime.d.ts");
pub const WEB_SERVER_COOKIE_RUNTIME: &str = include_str!("templates/web.server-cookie.runtime.ts");
pub const WEB_SERVER_COOKIE_DECLARATIONS: &str =
    include_str!("templates/web.server-cookie.runtime.d.ts");

pub fn render_template(template: &str, replacements: &[(&str, String)]) -> String {
    let mut output = template.to_owned();
    for (key, value) in replacements {
        output = output.replace(&format!("{{{{{}}}}}", key), value);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_runtime_templates_are_non_empty() {
        assert!(INDEX_RUNTIME.contains("createLinguini"));
        assert!(INDEX_RUNTIME_DECLARATIONS.contains("createLinguini"));
        assert!(LOCALE_RUNTIME.contains("normalizeLocale"));
        assert!(LOCALE_DECLARATIONS.contains("normalizeLocale"));
        assert!(PROJECT_INDEX_ENTRY.contains("{{INDEX_RUNTIME}}"));
        assert!(PROJECT_INDEX_DECLARATIONS.contains("{{INDEX_RUNTIME_DECLARATIONS}}"));
        assert!(SHARED_RUNTIME.contains("selectBranch"));
        assert!(SHARED_RUNTIME.contains("normalizeMessageArgs"));
        assert!(SHARED_RUNTIME.contains("Reflect.ownKeys"));
        assert!(SHARED_RUNTIME.contains("hasOwnProperty.call(branches, key)"));
        assert!(SHARED_RUNTIME.contains("throw new Error"));
        assert!(!SHARED_RUNTIME.contains("?? \"\""));
        assert!(SHARED_DECLARATIONS.contains("selectBranch"));
        assert!(SHARED_DECLARATIONS.contains("normalizeMessageArgs"));
        assert!(SINGLE_INDEX_RUNTIME.contains("createLinguini"));
        assert!(SINGLE_INDEX_DECLARATIONS.contains("createLinguini"));
        assert!(WEB_RUNTIME.contains("createWebI18n"));
        assert!(WEB_RUNTIME.contains("createWebLocaleI18n"));
        assert!(WEB_DECLARATIONS.contains("LinguiniRequestContext"));
        assert!(WEB_DECLARATIONS.contains("LinguiniWebLocale"));
        assert!(WEB_PATH_RUNTIME.contains("resolvePathLocale"));
        assert!(WEB_COOKIE_RUNTIME.contains("resolveCookieLocale"));
        assert!(WEB_LOCAL_STORAGE_RUNTIME.contains("resolveLocalStorageLocale"));
        assert!(WEB_ACCEPT_LANGUAGE_RUNTIME.contains("resolveAcceptLanguageLocale"));
        assert!(SVELTE_RUNTIME.contains("createLinguiniRune"));
        assert!(SVELTE_DECLARATIONS.contains("LinguiniRune"));
        assert!(SVELTE_CONTROL_RUNTIME.contains("createLinguiniControl"));
        assert!(SVELTE_CONTROL_DECLARATIONS.contains("LinguiniSvelteControl"));
        assert!(SVELTE_CONTEXT_RUNTIME.contains("createLinguiniRune"));
        assert!(SVELTE_CONTEXT_DECLARATIONS.contains("LinguiniRune"));
        assert!(SVELTE_EFFECTS_RUNTIME.contains("{{LINK_RUNTIME_START}}"));
        assert!(WEB_RUNTIME_LINKS_RUNTIME.contains("startRuntimeLinkLocalization"));
        assert!(WEB_RUNTIME_LINKS_RUNTIME.contains("MutationObserver"));
        assert!(WEB_LINK_TRANSFORM_RUNTIME.contains("localizeTransformedHref"));
        assert!(WEB_SERVER_COOKIE_RUNTIME.contains("persistLocaleCookie"));
        assert!(SVELTE_EFFECTS_DECLARATIONS.contains("destroyLinguiniEffects"));
        assert!(SVELTE_LOCALE_RUNTIME.contains("getCurrentLocale"));
        assert!(SVELTE_LOCALE_CONTEXT_RUNTIME.contains("getCurrentLocale"));
        assert!(SVELTE_LOCALE_STANDALONE_RUNTIME.contains("initializeCurrentLocale"));
        assert!(SVELTE_LOCALE_DECLARATIONS.contains("getCurrentLocale"));
        assert!(SVELTE_LOCALE_CONTEXT_DECLARATIONS.contains("getCurrentLocale"));
        assert!(SVELTE_LOCALE_STANDALONE_DECLARATIONS.contains("initializeCurrentLocale"));
        assert!(SVELTEKIT_RUNTIME.contains("createHandle"));
        assert!(SVELTEKIT_RUNTIME.contains("export const linguiniHandle"));
        assert!(SVELTEKIT_RUNTIME.contains("export const linguiniReroute"));
        assert!(SVELTEKIT_RUNTIME.contains("export const linguiniLoad"));
        assert!(SVELTEKIT_RUNTIME.contains("{{SERVER_COOKIE_IMPORT}}"));
        assert!(SVELTEKIT_DECLARATIONS.contains("linguiniHandle: Handle"));
        assert!(SVELTEKIT_DECLARATIONS.contains("linguiniReroute: Reroute"));
        assert!(SVELTEKIT_DECLARATIONS.contains("linguiniLoad: ServerLoad"));
        assert!(SVELTEKIT_CONTROL_RUNTIME.contains("createWebLocaleI18n"));
        assert!(SVELTEKIT_CONTROL_DECLARATIONS.contains("LinguiniServerLocaleContext"));
        assert!(SVELTEKIT_RUNTIME.contains("status: 307"));
        assert!(!SVELTEKIT_RUNTIME.contains("redirectStatus"));
        assert!(SVELTEKIT_RUNTIME.contains("locals.linguini = context"));
        assert!(!SVELTEKIT_RUNTIME.contains("locals.locale"));
        assert!(!SVELTEKIT_RUNTIME.contains("locals.direction"));
        assert!(!SVELTEKIT_RUNTIME.contains("locals.l ="));
        assert!(!SVELTEKIT_DECLARATIONS.contains("\n      locale: Locale;"));
        assert!(!SVELTEKIT_DECLARATIONS.contains("\n      direction: TextDirection;"));
        assert!(!SVELTEKIT_DECLARATIONS.contains("\n      l: Linguini;"));
        assert!(SVELTEKIT_DECLARATIONS.contains("SerializedLinguiniContext"));
    }

    #[test]
    fn render_template_replaces_double_brace_tokens() {
        let rendered = render_template("before {{VALUE}} after", &[("VALUE", "ok".to_owned())]);
        assert_eq!(rendered, "before ok after");
    }
}
