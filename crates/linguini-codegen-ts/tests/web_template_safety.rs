const SVELTE_RUNTIME: &str = include_str!("../src/module/templates/svelte.runtime.ts");
const SVELTE_EFFECTS_RUNTIME: &str =
    include_str!("../src/module/templates/svelte-effects.runtime.ts");
const SVELTEKIT_RUNTIME: &str = include_str!("../src/module/templates/sveltekit.runtime.ts");
const WEB_RUNTIME: &str = include_str!("../src/module/templates/web.runtime.ts");

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
fn browser_link_observer_is_owned_batched_and_bounded() {
    assert!(SVELTE_EFFECTS_RUNTIME.contains("hot?.dispose(destroyLinguiniEffects)"));
    assert!(SVELTE_EFFECTS_RUNTIME.contains("AUTO_LINK_MAX_PENDING_ROOTS"));
    assert!(SVELTE_EFFECTS_RUNTIME.contains("AUTO_LINK_NODE_BUDGET"));
    assert!(SVELTE_EFFECTS_RUNTIME.contains("shouldLocalizeLink"));
    assert!(!SVELTE_EFFECTS_RUNTIME.contains("activeAutoLinkCleanup"));
    assert!(!SVELTE_EFFECTS_RUNTIME.contains("querySelectorAll"));
    assert!(!SVELTE_RUNTIME.contains("MutationObserver"));
    assert!(!SVELTE_RUNTIME.contains("AUTO_LINK_MAX_PENDING_ROOTS"));
}
