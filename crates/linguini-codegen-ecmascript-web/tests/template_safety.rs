#[test]
fn generated_runtime_does_not_rewrite_html_with_anchor_regexes() {
    let web_runtime = linguini_codegen_ecmascript_web::compile_web_module(&Default::default()).code;
    assert!(!web_runtime.contains("localizeMarkupLinks"));
    assert!(!web_runtime.contains(r"/<a\b"));
    assert!(!web_runtime.contains("\"directory\""));
}

#[test]
fn web_runtime_options_are_structured_and_drop_flat_compatibility_fields() {
    let web_runtime = linguini_codegen_ecmascript_web::compile_web_module(&Default::default()).code;
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
