use linguini_analyzer::{
    analyze_unused_messages, ApplicationUsage, DiagnosticCategory, DiagnosticSeverity,
    PublicMessage,
};
use linguini_syntax::Span;

#[test]
fn extracts_static_dot_bracket_alias_and_template_references() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { l as messages } from "$lib/generated/linguini";
        messages.main.title();
        messages["shop"]["checkout"]();
        const rendered = `${messages.main.subtitle()}`;
        const ignored = "messages.main.string_only";
        // messages.main.comment_only();
        /* messages.main.block_comment_only(); */
        "#,
    );

    assert_eq!(
        usage.static_paths().collect::<Vec<_>>(),
        ["main.subtitle", "main.title", "shop.checkout"]
    );
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn recognizes_supported_generated_runtime_roots_and_aliases() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { messages as translated } from "$lib/generated/linguini/svelte";
        translated.main.alias();
        linguini.l.main.runtime();
        provider.messages.main.provider();
        const loaded = import("./lazy");
        l.main.after_dynamic_import();
        "#,
    );

    assert_eq!(
        usage.static_paths().collect::<Vec<_>>(),
        [
            "main.after_dynamic_import",
            "main.alias",
            "main.provider",
            "main.runtime"
        ]
    );
}

#[test]
fn import_meta_does_not_hide_following_message_usage() {
    let usage = ApplicationUsage::from_source(
        r#"
        if (import.meta.env.DEV || flags.import) l.main.development();
        const object = { import: from("module"), title: l.main.object_key() };
        l.main.after_import_meta();
        "#,
    );

    assert_eq!(
        usage.static_paths().collect::<Vec<_>>(),
        [
            "main.after_import_meta",
            "main.development",
            "main.object_key"
        ]
    );
}

#[test]
fn tracks_generated_facades_and_factory_results() {
    let usage = ApplicationUsage::from_source(
        r#"
        import {
          configureLinguini,
          createLinguini as make,
          lgl as base,
        } from "$lib/generated/linguini";
        const l = configureLinguini({ language: "en" });
        const runtime: Linguini = make("ru");
        l.main.configured();
        runtime.main.bound();
        base.main.default_locale();
        make("en").main.direct();
        linguini.createLinguini("en").main.namespace_factory();
        linguini["l"].main.bracket_root();
        linguini["createLinguini"]("en").main.bracket_factory();
        createLinguini(l.main.factory_locale()).main.nested_argument();
        "#,
    );

    assert_eq!(
        usage.static_paths().collect::<Vec<_>>(),
        [
            "main.bound",
            "main.bracket_factory",
            "main.bracket_root",
            "main.configured",
            "main.default_locale",
            "main.direct",
            "main.factory_locale",
            "main.namespace_factory",
            "main.nested_argument"
        ]
    );
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn decodes_injective_generated_identifiers_to_canonical_paths() {
    let messages = [
        PublicMessage::new("shop.class.foo-bar", Span::new(0, 4)),
        PublicMessage::new("shop.class.unused", Span::new(5, 9)),
    ];
    let usage =
        ApplicationUsage::from_source(r#"l["__lgl_name_73686F702E636C617373"]["foo-bar"]();"#);

    assert_eq!(
        usage.static_paths().collect::<Vec<_>>(),
        ["shop.class.foo-bar"]
    );
    let diagnostics = analyze_unused_messages(&messages, &usage, &[]);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("shop.class.unused"));
}

#[test]
fn generated_identifier_alias_keeps_the_raw_name_candidate() {
    let messages = [
        PublicMessage::new("main.__lgl_name_636C617373", Span::new(0, 4)),
        PublicMessage::new("main.other", Span::new(5, 9)),
    ];
    let usage = ApplicationUsage::from_source("l.main.__lgl_name_636C617373();");

    let diagnostics = analyze_unused_messages(&messages, &usage, &[]);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("main.other"));
}

#[test]
fn escaped_factory_result_disables_an_unsafe_unused_proof() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { createLinguini } from "./generated/linguini";
        export function messagesFor(locale: string) {
          return createLinguini(locale);
        }
        "#,
    );

    assert_eq!(usage.dynamic_prefixes().collect::<Vec<_>>(), [""]);
}

#[test]
fn factory_value_alias_disables_an_unsafe_unused_proof() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { createLinguini } from "./generated/linguini";
        const make = createLinguini;
        export { createLinguini as exportedFactory };
        make("en").main.title();
        "#,
    );

    assert_eq!(usage.dynamic_prefixes().collect::<Vec<_>>(), [""]);
}

#[test]
fn destructured_factory_result_is_treated_as_a_dynamic_escape() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { createLinguini } from "./generated/linguini";
        const { main: translated } = createLinguini("en");
        translated.title();
        "#,
    );

    assert_eq!(usage.dynamic_prefixes().collect::<Vec<_>>(), [""]);
}

#[test]
fn exported_factory_binding_disables_cross_file_unused_proof() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { createLinguini } from "./generated/linguini";
        export const translations = createLinguini("en");
        translations.main.local_use();
        "#,
    );

    assert_eq!(usage.dynamic_prefixes().collect::<Vec<_>>(), [""]);
    assert_eq!(usage.static_paths().collect::<Vec<_>>(), ["main.local_use"]);
}

#[test]
fn records_dynamic_access_at_narrowest_known_prefix() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { l } from "./generated/linguini";
        l.shop.checkout[key]();
        const runtimeGroup = l.account;
        "#,
    );

    assert_eq!(
        usage.dynamic_prefixes().collect::<Vec<_>>(),
        ["account", "shop.checkout"]
    );
}

#[test]
fn bare_root_escape_disables_project_wide_unused_proof() {
    let usage = ApplicationUsage::from_source(
        r#"
        import { l } from "./generated/linguini";
        registerMessages(l);
        "#,
    );

    assert_eq!(usage.dynamic_prefixes().collect::<Vec<_>>(), [""]);
}

#[test]
fn diagnoses_only_messages_without_configured_references() {
    let messages = [
        PublicMessage::new("main.used", Span::new(0, 4)),
        PublicMessage::new("main.unused", Span::new(5, 11)),
        PublicMessage::new("shop.dynamic.one", Span::new(12, 15)),
        PublicMessage::new("shop.dynamic.two", Span::new(16, 19)),
        PublicMessage::new("admin.ignored", Span::new(20, 24)),
    ];
    let usage = ApplicationUsage::from_source(
        r#"
        l.main.used();
        l.shop.dynamic[key]();
        "#,
    );

    let diagnostics = analyze_unused_messages(&messages, &usage, &["admin".to_owned()]);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "schema message `main.unused` is not referenced by configured application sources"
    );
    assert_eq!(diagnostics[0].code, "unused_message");
    assert_eq!(diagnostics[0].lint_name, Some("unused_message"));
    assert_eq!(diagnostics[0].category, DiagnosticCategory::Lint);
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostics[0].source_span, Some(Span::new(5, 11)));
}

#[test]
fn static_group_escape_is_conservative() {
    let messages = [
        PublicMessage::new("main.account.title", Span::new(0, 4)),
        PublicMessage::new("main.account.subtitle", Span::new(5, 9)),
        PublicMessage::new("main.other", Span::new(10, 14)),
    ];
    let usage = ApplicationUsage::from_source("consume(l.main.account);");

    let diagnostics = analyze_unused_messages(&messages, &usage, &[]);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("main.other"));
}

#[test]
fn chained_host_properties_do_not_hide_a_used_message() {
    let messages = [
        PublicMessage::new("main.title", Span::new(0, 4)),
        PublicMessage::new("main.other", Span::new(5, 9)),
    ];
    let usage = ApplicationUsage::from_source("const upper = l.main.title.toUpperCase;");

    let diagnostics = analyze_unused_messages(&messages, &usage, &[]);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("main.other"));
}

#[test]
fn component_markup_text_does_not_hide_embedded_message_usage() {
    let usage = ApplicationUsage::from_source("<p>Don't worry</p>\n<h1>{l.main.title()}</h1>");

    assert_eq!(usage.static_paths().collect::<Vec<_>>(), ["main.title"]);
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn component_scripts_and_vue_interpolations_are_scanned_as_code() {
    let usage = ApplicationUsage::from_source(
        r#"
        <script lang="ts">
          const label = l.main.script();
        </script>
        <template>
          <p>It's translated: {{ l.main.interpolation() }}</p>
        </template>
        "#,
    );

    assert_eq!(
        usage.static_paths().collect::<Vec<_>>(),
        ["main.interpolation", "main.script"]
    );
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn vue_directive_attributes_are_scanned_as_expressions() {
    let usage = ApplicationUsage::from_source(
        r#"
        <button
          @click="l.main.save()"
          :title="l.main.title()"
          v-if="l.main.visible()"
          aria-label="l.main.not_an_expression()"
        >Save</button>
        "#,
    );

    assert_eq!(
        usage.static_paths().collect::<Vec<_>>(),
        ["main.save", "main.title", "main.visible"]
    );
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn svelte_block_closers_do_not_make_usage_uncertain() {
    let usage = ApplicationUsage::from_source(
        r#"
        {#if ready}
          <p>{l.main.ready()}</p>
        {/if}
        "#,
    );

    assert_eq!(usage.static_paths().collect::<Vec<_>>(), ["main.ready"]);
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn regex_literals_do_not_hide_following_message_usage() {
    let usage = ApplicationUsage::from_source(
        r#"
        /\'/.test(value);
        l.main.title();
        "#,
    );

    assert_eq!(usage.static_paths().collect::<Vec<_>>(), ["main.title"]);
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn regex_braces_do_not_close_template_interpolations() {
    let usage = ApplicationUsage::from_source(
        r#"const result = `${/}/.test(value) ? l.main.title() : "fallback"}`;"#,
    );

    assert_eq!(usage.static_paths().collect::<Vec<_>>(), ["main.title"]);
    assert!(usage.dynamic_prefixes().next().is_none());
}

#[test]
fn import_like_regex_text_is_not_parsed_as_an_import() {
    let usage = ApplicationUsage::from_source(
        r#"
        const pattern = /import { l as hidden } from "fake"/;
        l.main.title();
        "#,
    );

    assert_eq!(usage.static_paths().collect::<Vec<_>>(), ["main.title"]);
    assert!(usage.dynamic_prefixes().next().is_none());
}
