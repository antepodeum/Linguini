use linguini_analyzer::{
    analyze_unused_messages, ApplicationBindingProvenance, ApplicationReferenceKind,
    ApplicationUsage, DiagnosticCategory, DiagnosticSeverity, PublicMessage,
};
use linguini_syntax::{SourceId, Span};

#[test]
fn exposes_source_aware_static_references_and_provenance() {
    let source = r#"import { l as messages } from "@linguini/messages";
messages.main.title;
messages.main.items(count);"#;
    let usage = ApplicationUsage::from_source_in(source, SourceId(9));
    let references = usage.references().collect::<Vec<_>>();

    assert_eq!(references.len(), 2);
    assert_eq!(references[0].canonical_path, "main.title");
    assert_eq!(references[0].kind, ApplicationReferenceKind::Value);
    assert_eq!(references[0].span.source, SourceId(9));
    assert_eq!(
        references[0].span,
        Span::in_source(
            SourceId(9),
            source.find("messages.main.title").unwrap(),
            source.find("messages.main.title").unwrap() + "messages.main.title".len()
        )
    );
    assert_eq!(
        references[0].binding.provenance,
        ApplicationBindingProvenance::Imported {
            module_specifier: "@linguini/messages".to_owned(),
            imported: "l".to_owned(),
        }
    );
    assert_eq!(references[1].kind, ApplicationReferenceKind::Call);
    assert_eq!(references[1].span.end, source.find("(count)").unwrap());
}

#[test]
fn preserves_duplicate_spans_and_marks_implicit_roots() {
    let source = "l.main.title();\nl.main.title();";
    let usage = ApplicationUsage::from_source(source);
    let references = usage.references().collect::<Vec<_>>();

    assert_eq!(references.len(), 2);
    assert_ne!(references[0].span, references[1].span);
    assert_eq!(
        references[0].binding.provenance,
        ApplicationBindingProvenance::Implicit
    );
}

#[test]
fn records_factory_bindings_and_excludes_dynamic_paths() {
    let source = r#"import { createLinguini as make } from "@linguini/runtime";
make("en").main.title();
make("en").main[group]();"#;
    let usage = ApplicationUsage::from_source_in(source, SourceId(4));
    let references = usage.references().collect::<Vec<_>>();

    assert_eq!(references.len(), 1);
    assert_eq!(references[0].canonical_path, "main.title");
    assert_eq!(references[0].binding.local, "make");
    assert_eq!(
        references[0].binding.provenance,
        ApplicationBindingProvenance::Factory {
            factory: "createLinguini".to_owned()
        }
    );
    assert!(usage.dynamic_prefixes().any(|prefix| prefix == "main"));
}

#[test]
fn keeps_markup_and_script_reference_offsets_in_the_original_source() {
    let source = "<script>const label = l.main.script();</script>\n<h1>{l.main.markup()}</h1>";
    let usage = ApplicationUsage::from_source_in(source, SourceId(12));
    let mut references = usage.references().collect::<Vec<_>>();
    references.sort_by_key(|reference| reference.span.start);

    assert_eq!(references.len(), 2);
    let script_start = source.find("l.main.script").unwrap();
    let markup_start = source.find("l.main.markup").unwrap();
    assert_eq!(
        references[0].span,
        Span::in_source(
            SourceId(12),
            script_start,
            script_start + "l.main.script".len()
        )
    );
    assert_eq!(
        references[1].span,
        Span::in_source(
            SourceId(12),
            markup_start,
            markup_start + "l.main.markup".len()
        )
    );
}

#[test]
fn spans_optional_and_encoded_bracket_paths_without_call_arguments() {
    let source = r#"l?.main?.title();
l["__lgl_name_73686F702E636C617373"]["foo-bar"](value);"#;
    let usage = ApplicationUsage::from_source(source);
    let references = usage.references().collect::<Vec<_>>();

    assert_eq!(references.len(), 2);
    assert_eq!(references[0].canonical_path, "main.title");
    assert_eq!(references[0].span, Span::new(0, "l?.main?.title".len()));
    assert_eq!(references[1].canonical_path, "shop.class.foo-bar");
    assert_eq!(
        references[1].span,
        Span::new(source.find("l[").unwrap(), source.find("(value)").unwrap())
    );
}

#[test]
fn factory_bracket_reference_span_includes_its_receiver() {
    let source = r#"linguini["createLinguini"]("en").main.title();"#;
    let usage = ApplicationUsage::from_source(source);
    let references = usage.references().collect::<Vec<_>>();

    assert_eq!(references.len(), 1);
    assert_eq!(
        references[0].span,
        Span::new(
            0,
            source.find("(\"en\")").unwrap() + "(\"en\")".len() + ".main.title".len()
        )
    );
}

#[test]
fn does_not_emit_imported_provenance_for_properties_or_shadowed_bindings() {
    let source = r#"import { l as messages } from "@linguini/generated";
messages.main.global();
obj.messages.main.property();
obj["messages"].main.bracket_property();
function render(messages) { messages.main.parameter(); }
const local = (messages) => { messages.main.arrow(); };
{
  const messages = local;
  messages.main.block();
}"#;
    let usage = ApplicationUsage::from_source(source);
    let references = usage.references().collect::<Vec<_>>();

    assert_eq!(
        references
            .iter()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.global"]
    );
    assert!(references.iter().all(|reference| {
        matches!(
            reference.binding.provenance,
            ApplicationBindingProvenance::Imported { .. }
        )
    }));
    assert!(usage.static_paths().any(|path| path == "main.property"));
    assert!(usage.static_paths().any(|path| path == "main.parameter"));
}

#[test]
fn isolated_parenthesized_arrow_expression_does_not_use_imported_binding() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const render = (messages) => messages.main.shadowed();
messages.main.outer();"#,
    );
    let paths = usage
        .references()
        .map(|reference| reference.canonical_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(paths, ["main.outer"]);
}

#[test]
fn isolated_block_arrow_expression_does_not_use_imported_binding() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const render = (messages) => { messages.main.shadowed(); };
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn isolated_single_parameter_arrow_expression_does_not_use_imported_binding() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const render = messages => messages.main.shadowed();
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn isolated_typed_and_generic_arrows_do_not_use_imported_binding() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const typed = (messages: string) => messages.main.typed();
const generic = <T>(messages: T) => messages.main.generic();
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn isolated_destructured_declarations_shadow_imported_binding() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
{
  const { messages } = value;
  messages.main.object_shadowed();
}
{
  const [messages] = values;
  messages.main.array_shadowed();
}
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn named_function_and_class_expression_names_end_with_their_bodies() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const fn = function messages() { messages.main.function_shadowed(); };
const Cls = class messages { method() { messages.main.class_shadowed(); } };
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn unshadowed_imported_bindings_inside_functions_and_blocks_remain_references() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
function render() { messages.main.function_use(); }
{ messages.main.block_use(); }
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.function_use", "main.block_use", "main.outer"]
    );
}

#[test]
fn hoisted_function_declaration_shadows_imported_alias_in_its_scope() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
function render() {
  messages.main.before();
  function messages() {}
  messages.main.after();
}
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn hoisted_class_declaration_shadows_imported_alias_in_its_scope() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
function render() {
  messages.main.before();
  class messages {}
  messages.main.after();
}
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn concise_arrow_scope_stops_at_call_argument_comma() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
consume(messages => messages.main.shadowed(), messages.main.real());"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.real"]
    );
}

#[test]
fn concise_arrow_scope_stops_at_array_sibling() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const values = [messages => messages.main.shadowed(), messages.main.real()];"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.real"]
    );
}

#[test]
fn concise_arrow_conditional_keeps_both_branches_shadowed() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const value = messages => ready ? messages.main.one() : messages.main.two();
messages.main.real();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.real"]
    );
}

#[test]
fn nested_destructuring_uses_the_containing_lexical_scope() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
{
  const { nested: { messages } } = value;
  messages.main.shadowed();
}
messages.main.real();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.real"]
    );
}

#[test]
fn class_and_object_method_parameters_shadow_imported_aliases() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
class View {
  render(messages) { messages.main.class_method(); }
  *iterate(messages) { messages.main.generator_method(); }
  constructor(messages) { messages.main.constructor_method(); }
}
const object = {
  render(messages) { messages.main.object_method(); },
};
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn function_and_arrow_return_annotations_find_the_actual_body() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
function render(messages: string): Result<{ value: string }> {
  messages.main.function_annotation();
}
const arrow = (messages: string): { value: string } => messages.main.arrow_annotation();
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn type_literal_annotations_do_not_shadow_imported_aliases() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
const value: { messages: string } = input;
messages.main.real();"#,
    );
    let references = usage.references().collect::<Vec<_>>();
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].canonical_path, "main.real");
    assert!(matches!(
        references[0].binding.provenance,
        ApplicationBindingProvenance::Imported { .. }
    ));
}

#[test]
fn generic_async_and_generator_methods_shadow_imported_aliases() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
class View {
  async render<T>(messages?: T) { messages.main.async_method(); }
  *iterate<T>(messages: T) { messages.main.generator_method(); }
}
const object = {
  async render<T>(messages: T) { messages.main.object_async(); },
  *iterate<T>(messages: T) { messages.main.object_generator(); },
};
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outer"]
    );
}

#[test]
fn type_literals_after_destructuring_do_not_shadow_imported_properties() {
    let usage = ApplicationUsage::from_source(
        r#"import { l as messages } from "generated";
{
  const { value }: { messages: string } = input;
  messages.main.object_type();
}
{
  const [value]: [messages: string] = input;
  messages.main.array_type();
}
messages.main.outer();"#,
    );
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.object_type", "main.array_type", "main.outer"]
    );
}

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

#[test]
fn exposes_exact_sole_and_mixed_import_removal_contracts() {
    let source_id = SourceId(91);
    let source = concat!(
        "import { messages as msg } from \"generated-a\";\r\n",
        "const unicode = \"Привет\";\r\n",
        "import {\r\n  helper,\r\n  l as tr,\r\n  other\r\n} from \"generated-b\";\r\n",
        "msg.main.title;\r\n",
        "tr.main.items(2);\r\n",
    );
    let usage = ApplicationUsage::from_source_in(source, source_id);
    let imports = usage.imports().collect::<Vec<_>>();
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].id.source, source_id);
    assert_eq!(imports[0].module_specifier, "generated-a");
    assert_eq!(imports[0].imported, "messages");
    assert_eq!(imports[0].local, "msg");
    assert_eq!(
        &source[imports[0].declaration_span.start..imports[0].declaration_span.end],
        "import { messages as msg } from \"generated-a\";"
    );
    assert_eq!(imports[0].removal_span, imports[0].declaration_span);
    assert!(imports[0].exact_uses_only);
    assert_eq!(
        &source[imports[1].item_span.start..imports[1].item_span.end],
        "l as tr"
    );
    assert_eq!(
        &source[imports[1].removal_span.start..imports[1].removal_span.end],
        "l as tr,"
    );
    assert!(imports[1].exact_uses_only);
    let references = usage.references().collect::<Vec<_>>();
    assert_eq!(references[0].import_binding, Some(imports[0].id));
    assert_eq!(references[1].import_binding, Some(imports[1].id));
}

#[test]
fn import_safety_rejects_comments_bare_dynamic_optional_and_ambiguity() {
    for source in [
        "import { /* keep */ l } from \"generated\";\nl.main.title;",
        "import { l } from \"generated\";\nl.main.title; consume(l);",
        "import { l } from \"generated\";\nl.main.title; l.main[key]();",
        "import { l } from \"generated\";\nl.main.title?.();",
    ] {
        let usage = ApplicationUsage::from_source(source);
        let binding = usage.imports().next().expect("tracked import");
        assert!(!binding.exact_uses_only, "{source}");
    }

    let malformed =
        ApplicationUsage::from_source("import { l as } from \"generated\";\nl.main.title;");
    assert!(malformed.imports().next().is_none());
}

#[test]
fn import_identities_are_deterministic_across_repeated_declarations_and_shadowing() {
    let source = concat!(
        "import { l as first } from \"generated\";\n",
        "import { l as second } from \"generated\";\n",
        "first.main.title;\n",
        "{ const first = local; first.main.dynamic; }\n",
        "second.main.items(1);\n",
    );
    let first = ApplicationUsage::from_source_in(source, SourceId(44));
    let second = ApplicationUsage::from_source_in(source, SourceId(44));
    assert_eq!(first, second);
    let imports = first.imports().collect::<Vec<_>>();
    assert_eq!(imports.len(), 2);
    assert_ne!(imports[0].id, imports[1].id);
    assert!(imports.iter().all(|binding| binding.exact_uses_only));
    assert_eq!(
        first
            .references()
            .filter_map(|reference| reference.import_binding)
            .collect::<Vec<_>>(),
        imports.iter().map(|binding| binding.id).collect::<Vec<_>>()
    );
}

#[test]
fn duplicate_local_imports_are_ambiguous_across_modules_and_symbols() {
    let source = concat!(
        "import { l as same } from \"generated-a\";\n",
        "import { messages as same } from \"generated-b\";\n",
        "same.main.title;\n",
    );
    let usage = ApplicationUsage::from_source_in(source, SourceId(55));
    let imports = usage.imports().collect::<Vec<_>>();
    assert_eq!(imports.len(), 2);
    assert!(imports.iter().all(|binding| !binding.exact_uses_only));
    assert!(usage
        .references()
        .all(|reference| reference.import_binding.is_none()));
}

#[test]
fn named_import_list_rejects_empty_chunks_but_accepts_one_trailing_comma() {
    for source in [
        "import { , l } from \"generated\";\nl.main.title;",
        "import { l,, } from \"generated\";\nl.main.title;",
        "import { l, , helper } from \"generated\";\nl.main.title;",
        "import { l } from \"generated\" garbage;\nl.main.title;",
        "import { l } garbage from \"generated\";\nl.main.title;",
        "import { l } from garbage \"generated\";\nl.main.title;",
        "import { l + helper } from \"generated\";\nl.main.title;",
        "import { l } from \"generated\" with { type: \"json\" };\nl.main.title;",
        "import defaultThing, { l } from \"generated\";\nl.main.title;",
        "import * as l from \"generated\";\nl.main.title;",
        "import { l } from \"\";\nl.main.title;",
    ] {
        let usage = ApplicationUsage::from_source(source);
        assert!(usage.imports().next().is_none(), "{source}");
        assert!(usage
            .references()
            .all(|reference| reference.import_binding.is_none()));
        assert!(usage.references().all(|reference| !matches!(
            reference.binding.provenance,
            ApplicationBindingProvenance::Imported { .. }
        )));
    }

    let source = "import { l, } from \"generated\";\nl.main.title;";
    let usage = ApplicationUsage::from_source(source);
    let binding = usage.imports().next().expect("valid trailing comma");
    assert!(binding.exact_uses_only);
    assert_eq!(binding.removal_span, binding.declaration_span);
    assert_eq!(
        &source[binding.removal_span.start..binding.removal_span.end],
        "import { l, } from \"generated\";"
    );

    let source = "import { messages as msg } from \"generated\"\nmsg.main.title;";
    let usage = ApplicationUsage::from_source(source);
    let binding = usage.imports().next().expect("ASI import");
    assert!(binding.exact_uses_only);
    assert_eq!(
        &source[binding.removal_span.start..binding.removal_span.end],
        "import { messages as msg } from \"generated\""
    );
}

#[test]
fn named_import_aliases_require_strict_module_binding_identifiers() {
    for alias in [
        "for",
        "class",
        "await",
        "yield",
        "let",
        "static",
        "enum",
        "implements",
        "interface",
        "package",
        "private",
        "protected",
        "public",
        "null",
        "true",
        "false",
        "eval",
        "arguments",
    ] {
        let source = format!("import {{ l as {alias} }} from \"generated\";\n{alias}.main.title;");
        let usage = ApplicationUsage::from_source(&source);
        assert!(usage.imports().next().is_none(), "{alias}");
        assert!(
            usage
                .references()
                .all(|reference| reference.import_binding.is_none()),
            "{alias}"
        );
        assert!(
            usage.references().all(|reference| !matches!(
                reference.binding.provenance,
                ApplicationBindingProvenance::Imported { .. }
            )),
            "{alias}"
        );
    }

    for alias in ["перевод", "$", "_"] {
        let source = format!("import {{ l as {alias} }} from \"generated\";\n{alias}.main.title;");
        let usage = ApplicationUsage::from_source(&source);
        let binding = usage.imports().next().expect("valid binding identifier");
        assert_eq!(binding.local, alias);
        assert!(binding.exact_uses_only);
        assert_eq!(
            usage
                .references()
                .next()
                .and_then(|reference| reference.import_binding),
            Some(binding.id)
        );
    }

    let usage = ApplicationUsage::from_source("import { l } from \"generated\";\nl.main.title;");
    assert!(usage.imports().next().is_some());
}

#[test]
fn multiline_import_attributes_and_trailing_comments_are_not_transformable() {
    for source in [
        "import { l } from \"generated\"\nwith { type: \"json\" };\nl.main.title;",
        "import { l } from \"generated\"\nassert { type: \"json\" };\nl.main.title;",
    ] {
        let usage = ApplicationUsage::from_source(source);
        assert!(usage.imports().next().is_none(), "{source}");
        assert!(
            usage
                .references()
                .all(|reference| reference.import_binding.is_none()),
            "{source}"
        );
    }

    for source in [
        "import { l } from \"generated\" // keep\nl.main.title;",
        "import { l } from \"generated\" /* keep */\nl.main.title;",
        "import { l } from \"generated\"; // keep\nl.main.title;",
        "import { l } from \"generated\"; /* keep */\nl.main.title;",
    ] {
        let usage = ApplicationUsage::from_source(source);
        let binding = usage.imports().next().expect("tracked import");
        assert!(!binding.exact_uses_only, "{source}");
    }
}

#[test]
fn imported_svelte_runes_and_markup_keep_exact_references() {
    let source = r#"<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
  let count = $state(1);
  const title = $derived(l.main.hero.title);
  const nav = $derived([
    { label: l.main.nav.why },
    { label: l.main.nav.codegen },
  ]);
  function render() { return l.main.hero.copy; }
  function save() { return l.main.hero.primary_cta(); }
  const lines = $derived([l.main.playground.sentence(count)]);
</script>
<svelte:head><title>{l.main.hero.title}</title></svelte:head>
<p>{l.main.hero.tagline}</p>"#;
    let usage = ApplicationUsage::from_source_in(source, SourceId(72));
    let import_binding = usage.imports().next().expect("generated l import");
    assert!(import_binding.exact_uses_only);
    let import_id = import_binding.id;
    let references = usage.references().collect::<Vec<_>>();
    assert_eq!(
        references
            .iter()
            .map(|reference| (reference.canonical_path.as_str(), reference.kind))
            .collect::<Vec<_>>(),
        [
            ("main.hero.title", ApplicationReferenceKind::Value),
            ("main.nav.why", ApplicationReferenceKind::Value),
            ("main.nav.codegen", ApplicationReferenceKind::Value),
            ("main.hero.copy", ApplicationReferenceKind::Value),
            ("main.hero.primary_cta", ApplicationReferenceKind::Call),
            ("main.playground.sentence", ApplicationReferenceKind::Call),
            ("main.hero.title", ApplicationReferenceKind::Value),
            ("main.hero.tagline", ApplicationReferenceKind::Value),
        ]
    );
    assert!(references
        .iter()
        .all(|reference| reference.import_binding == Some(import_id)));
    let expected_spans = [
        "l.main.hero.title",
        "l.main.nav.why",
        "l.main.nav.codegen",
        "l.main.hero.copy",
        "l.main.hero.primary_cta",
        "l.main.playground.sentence",
        "l.main.hero.title",
        "l.main.hero.tagline",
    ];
    for (reference, expected) in references.iter().zip(expected_spans) {
        assert_eq!(&source[reference.span.start..reference.span.end], expected);
        assert_eq!(reference.span.source, SourceId(72));
    }
}

#[test]
fn imported_svelte_object_reads_do_not_look_like_shadow_bindings() {
    let source = r#"<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
  const options = $derived([
    { value: 'apple' as const, label: l.main.playground.fruit_apple_label },
    { value: 'pear' as const, label: l.main.playground.fruit_pear_label },
  ]);
</script>
<select>{#each options as option}<option>{option.label}</option>{/each}</select>"#;
    let usage = ApplicationUsage::from_source(source);
    let import_binding = usage.imports().next().expect("generated l import");
    assert!(import_binding.exact_uses_only);
    let references = usage.references().collect::<Vec<_>>();
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        [
            "main.playground.fruit_apple_label",
            "main.playground.fruit_pear_label"
        ]
    );
    assert!(references
        .iter()
        .all(|reference| reference.import_binding == Some(import_binding.id)));
}

#[test]
fn destructuring_property_keys_do_not_shadow_imported_aliases() {
    let source = r#"
import { l as messages } from "generated";
const { messages: renamed } = value;
messages.main.alias_keeps_import();
"#;
    let usage = ApplicationUsage::from_source(source);
    let import_binding = usage.imports().next().expect("generated messages import");
    assert!(import_binding.exact_uses_only);
    let reference = usage.references().next().expect("imported alias reference");
    assert_eq!(reference.canonical_path, "main.alias_keeps_import");
    assert_eq!(reference.import_binding, Some(import_binding.id));
}

#[test]
fn typed_variable_declarations_still_shadow_imported_aliases() {
    for declaration in [
        "const messages: Message = local;",
        "let messages: Message = local;",
    ] {
        let source = format!(
            "import {{ l as messages }} from \"generated\";\n{declaration}\nmessages.main.typed_binding();"
        );
        let usage = ApplicationUsage::from_source(&source);
        let import_binding = usage.imports().next().expect("generated messages import");
        assert!(!import_binding.exact_uses_only, "{declaration}");
        assert!(usage.references().next().is_none(), "{declaration}");
    }
}

#[test]
fn var_bindings_shadow_imports_across_nested_blocks() {
    let source = r#"
import { l as messages } from "generated";
function render() {
  {
    var messages = local;
    messages.main.inside_block();
  }
  messages.main.after_block_var_shadow();
}
messages.main.outside_function();
"#;
    let usage = ApplicationUsage::from_source(source);
    assert_eq!(
        usage
            .references()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        ["main.outside_function"]
    );
}

#[test]
fn repeated_svelte_markup_references_keep_each_exact_span() {
    let source = r#"<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
</script>
<div class="planned-strip">
  <p>{l.main.codegen.planned_title}</p>
  <p>{l.main.codegen.planned_intro}</p>
  <div class="planned-icons" aria-label={l.main.codegen.planned_title}>
    {#each items as item, index (item.label)}
      <span>{item.label}</span>
    {/each}
  </div>
</div>"#;
    let usage = ApplicationUsage::from_source(source);
    let import_binding = usage.imports().next().expect("generated l import");
    assert!(import_binding.exact_uses_only);
    let references = usage.references().collect::<Vec<_>>();
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.canonical_path.as_str())
            .collect::<Vec<_>>(),
        [
            "main.codegen.planned_title",
            "main.codegen.planned_intro",
            "main.codegen.planned_title"
        ]
    );
    assert!(references
        .iter()
        .all(|reference| reference.import_binding == Some(import_binding.id)));
    assert_eq!(
        references
            .iter()
            .map(|reference| &source[reference.span.start..reference.span.end])
            .collect::<Vec<_>>(),
        [
            "l.main.codegen.planned_title",
            "l.main.codegen.planned_intro",
            "l.main.codegen.planned_title"
        ]
    );
}

#[test]
fn repeated_svelte_markup_references_keep_distinct_transform_spans() {
    let source = r#"<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
</script>
<button title={l.main.hero.title}>{l.main.hero.title}</button>"#;
    let usage = ApplicationUsage::from_source_in(source, SourceId(73));
    let import_binding = usage.imports().next().expect("generated l import");
    assert!(import_binding.exact_uses_only);
    let references = usage.references().collect::<Vec<_>>();
    assert_eq!(references.len(), 2);
    assert!(references
        .iter()
        .all(|reference| reference.canonical_path == "main.hero.title"));
    assert!(references
        .iter()
        .all(|reference| reference.import_binding == Some(import_binding.id)));
    assert!(references[0].span.start < references[1].span.start);
    assert_ne!(references[0].span, references[1].span);
    for reference in &references {
        assert_eq!(
            &source[reference.span.start..reference.span.end],
            "l.main.hero.title"
        );
        assert_eq!(reference.span.source, SourceId(73));
        assert_eq!(reference.kind, ApplicationReferenceKind::Value);
    }
}
