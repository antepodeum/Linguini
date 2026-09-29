const SVELTE_RUNTIME: &str = include_str!("../src/module/templates/svelte.runtime.ts");
const SVELTEKIT_RUNTIME: &str = include_str!("../src/module/templates/sveltekit.runtime.ts");
const WEB_RUNTIME: &str = include_str!("../src/module/templates/web.runtime.ts");
const WEB_DECLARATIONS: &str = include_str!("../src/module/templates/web.runtime.d.ts");

#[test]
fn sveltekit_page_transform_preserves_streaming() {
    assert!(SVELTEKIT_RUNTIME.contains("transformPageChunk"));
    assert!(!SVELTEKIT_RUNTIME.contains("bufferedHtml"));
    assert!(!SVELTEKIT_RUNTIME.contains("localizeMarkupLinks"));
}

#[test]
fn generated_runtime_does_not_rewrite_html_with_anchor_regexes() {
    assert!(!WEB_RUNTIME.contains("localizeMarkupLinks"));
    assert!(!WEB_RUNTIME.contains(r"/<a\b"));
    assert!(!WEB_RUNTIME.contains("\"directory\""));
}

#[test]
fn web_runtime_options_are_structured_and_drop_flat_compatibility_fields() {
    for required in ["routing?: {", "locale?: {", "cookie?: {", "routes?: {"] {
        assert!(WEB_RUNTIME.contains(required));
        assert!(WEB_DECLARATIONS.contains(required));
    }
    for removed in [
        "prefixDefaultLocale",
        "basePath?:",
        "trailingSlash",
        "localizeLinks",
    ] {
        assert!(!WEB_RUNTIME.contains(removed));
        assert!(!WEB_DECLARATIONS.contains(removed));
    }
}

#[test]
fn browser_link_observer_is_owned_batched_and_bounded() {
    let effects =
        linguini_codegen_ts::compile_typescript_svelte_effects_module(&Default::default(), true);
    let runtime = linguini_codegen_ts::compile_typescript_web_runtime_links_module();
    let web_runtime_links = runtime.code;
    assert!(effects
        .code
        .contains("hot?.dispose(destroyLinguiniEffects)"));
    assert!(web_runtime_links.contains("MAX_PENDING_ROOTS"));
    assert!(web_runtime_links.contains("NODE_BUDGET"));
    assert!(web_runtime_links.contains("shouldLocalizeLink"));
    assert!(!web_runtime_links.contains("activeAutoLinkCleanup"));
    assert!(!web_runtime_links.contains("querySelectorAll"));
    assert!(!effects.code.contains("MutationObserver"));
    assert!(!SVELTE_RUNTIME.contains("MutationObserver"));
    assert!(!SVELTE_RUNTIME.contains("AUTO_LINK_MAX_PENDING_ROOTS"));
}
