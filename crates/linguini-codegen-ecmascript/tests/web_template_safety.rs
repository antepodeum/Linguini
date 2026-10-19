#[test]
fn sveltekit_page_transform_preserves_streaming() {
    let sveltekit_runtime =
        linguini_codegen_ecmascript::compile_sveltekit_module(&Default::default()).code;
    assert!(sveltekit_runtime.contains("transformPageChunk"));
    assert!(!sveltekit_runtime.contains("bufferedHtml"));
    assert!(!sveltekit_runtime.contains("localizeMarkupLinks"));
}

#[test]
fn generated_runtime_does_not_rewrite_html_with_anchor_regexes() {
    let web_runtime = linguini_codegen_ecmascript::compile_web_module(&Default::default()).code;
    assert!(!web_runtime.contains("localizeMarkupLinks"));
    assert!(!web_runtime.contains(r"/<a\b"));
    assert!(!web_runtime.contains("\"directory\""));
}

#[test]
fn web_runtime_options_are_structured_and_drop_flat_compatibility_fields() {
    let web_runtime = linguini_codegen_ecmascript::compile_web_module(&Default::default()).code;
    let web_declarations =
        include_str!("../../../tests/fixtures/golden/snapshots/js-web/default/typescript/web.d.ts");
    for required in ["routing?: {", "locale?: {", "cookie?: {", "routes?: {"] {
        assert!(web_runtime.contains(required));
        assert!(web_declarations.contains(required));
    }
    for removed in [
        "prefixDefaultLocale",
        "basePath?:",
        "trailingSlash",
        "localizeLinks",
    ] {
        assert!(!web_runtime.contains(removed));
        assert!(!web_declarations.contains(removed));
    }
}

#[test]
fn browser_link_observer_is_owned_batched_and_bounded() {
    let svelte_runtime = linguini_codegen_ecmascript::compile_svelte_module(true).code;
    let effects =
        linguini_codegen_ecmascript::compile_svelte_effects_module(&Default::default(), true);
    let runtime = linguini_codegen_ecmascript::compile_web_runtime_links_module();
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
    assert!(!svelte_runtime.contains("MutationObserver"));
    assert!(!svelte_runtime.contains("AUTO_LINK_MAX_PENDING_ROOTS"));
}
