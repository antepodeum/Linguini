pub const SHARED_RUNTIME: &str = include_str!("templates/shared.runtime.ts");
pub const SHARED_DECLARATIONS: &str = include_str!("templates/shared.runtime.d.ts");
#[cfg(test)]
pub const SINGLE_INDEX_RUNTIME: &str = include_str!("templates/single-index.runtime.ts");
#[cfg(test)]
pub const SINGLE_INDEX_DECLARATIONS: &str = include_str!("templates/single-index.runtime.d.ts");
pub const SVELTE_RUNTIME: &str = include_str!("templates/svelte.runtime.ts");
pub const SVELTE_DECLARATIONS: &str = include_str!("templates/svelte.runtime.d.ts");
pub const SVELTE_CONTEXT_RUNTIME: &str = include_str!("templates/svelte.context.runtime.ts");
pub const SVELTE_CONTEXT_DECLARATIONS: &str = include_str!("templates/svelte.context.runtime.d.ts");
pub const SVELTEKIT_RUNTIME: &str = include_str!("templates/sveltekit.runtime.ts");
pub const SVELTEKIT_DECLARATIONS: &str = include_str!("templates/sveltekit.runtime.d.ts");
pub const SVELTEKIT_CONTROL_RUNTIME: &str = include_str!("templates/sveltekit-control.runtime.ts");
pub const SVELTEKIT_CONTROL_DECLARATIONS: &str =
    include_str!("templates/sveltekit-control.runtime.d.ts");
pub const WEB_RUNTIME: &str = include_str!("templates/web.runtime.ts");
pub const WEB_DECLARATIONS: &str = include_str!("templates/web.runtime.d.ts");
pub const WEB_SERVER_COOKIE_RUNTIME: &str = include_str!("templates/web.server-cookie.runtime.ts");
pub const WEB_SERVER_COOKIE_DECLARATIONS: &str =
    include_str!("templates/web.server-cookie.runtime.d.ts");
pub const WEB_SWITCH_ROUTE_RUNTIME: &str = include_str!("templates/web.switch-route.runtime.ts");
pub const WEB_SWITCH_ROUTE_DECLARATIONS: &str =
    include_str!("templates/web.switch-route.runtime.d.ts");

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
        assert!(SVELTE_RUNTIME.contains("createLinguiniRune"));
        assert!(SVELTE_DECLARATIONS.contains("LinguiniRune"));
        assert!(SVELTE_CONTEXT_RUNTIME.contains("createLinguiniRune"));
        assert!(SVELTE_CONTEXT_DECLARATIONS.contains("LinguiniRune"));
        assert!(WEB_SERVER_COOKIE_RUNTIME.contains("persistLocaleCookie"));
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
