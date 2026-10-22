#[test]
fn sveltekit_page_transform_preserves_streaming() {
    let sveltekit_runtime =
        linguini_codegen_sveltekit::compile_sveltekit_module(&Default::default()).code;
    assert!(sveltekit_runtime.contains("transformPageChunk"));
    assert!(!sveltekit_runtime.contains("bufferedHtml"));
    assert!(!sveltekit_runtime.contains("localizeMarkupLinks"));
}

#[test]
fn browser_link_observer_is_owned_batched_and_bounded() {
    let svelte_runtime = linguini_codegen_svelte::compile_svelte_module(true).code;
    let effects = linguini_codegen_svelte::compile_svelte_effects_module(
        &Default::default(),
        &Default::default(),
    );
    let runtime = linguini_codegen_ecmascript_web::compile_web_runtime_links_module();
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
