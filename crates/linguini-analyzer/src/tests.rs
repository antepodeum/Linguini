use super::{
    analyze_branch_coverage, analyze_expressions, analyze_function_patterns,
    analyze_locale_coverage, analyze_locale_coverage_with_options, analyze_locale_file,
    analyze_message_coverage, analyze_project_expressions, detect_reference_cycles,
    render_diagnostics, require_other_branch, BranchCoverage, Diagnostic, DiagnosticCategory,
    DiagnosticSeverity, ExpressionAnalysis, FormProperty, FormSignature, FunctionSignature,
    LocaleCoverageOptions, MessageToAnalyze, NamedSpan, PublicMessage, QuickFix, ReferenceNode,
    Variable,
};
use linguini_syntax::{parse_locale, parse_schema, Span};

#[test]
fn renders_primary_span_related_span_note_and_quick_fix() {
    let diagnostic = Diagnostic::error("unknown type `Color`", Span::new(13, 18))
        .with_related(Span::new(0, 5), "while checking this message")
        .with_note("schema types must be declared before use")
        .with_quick_fix(QuickFix::hint("declare enum Color"));

    let rendered = render_diagnostics("shop.lgs", "paint(color: Color)\n", &[diagnostic])
        .expect("render diagnostics");

    assert_eq!(
        rendered,
        include_str!("../../../tests/fixtures/golden/snapshots/diagnostic-schema-syntax.txt")
    );
}

#[test]
fn locale_analysis_does_not_warn_about_empty_files_without_schema_context() {
    let locale = parse_locale("").expect("empty locale is syntactically valid");
    let diagnostics = analyze_locale_file(&locale);

    assert!(diagnostics.is_empty());
}

#[test]
fn locale_analysis_reports_incomplete_nested_enum_branch_coverage() {
    let locale = parse_locale(
        "enum Gender { male, female, neuter, other }\n\
             form SizeAdj(Plural, Gender) {\n\
               one {\n\
                 male => большой\n\
                 female => большая\n\
               }\n\
               _ => большие\n\
             }\n",
    )
    .expect("locale parses");

    let diagnostics = analyze_locale_file(&locale);

    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        [
            "function `SizeAdj` for enum `Gender` is missing branch `neuter`",
            "function `SizeAdj` for enum `Gender` is missing branch `other`"
        ]
    );
}

#[test]
fn locale_analysis_accepts_wildcard_for_enum_branch_coverage() {
    let locale = parse_locale(
        "enum Gender { male, female, neuter, other }\n\
             form SizeAdj(Plural, Gender) {\n\
               one {\n\
                 male => большой\n\
                 female => большая\n\
                 _ => большое\n\
               }\n\
               _ => большие\n\
             }\n",
    )
    .expect("locale parses");

    let diagnostics = analyze_locale_file(&locale);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
        "{diagnostics:?}"
    );
}

#[test]
fn project_expression_analysis_validates_inline_selectors_bindings_and_branches() {
    let schema = parse_schema(
        "enum Gender { masculine, feminine, other }\ngreeting(name: String, gender: Gender)\n",
    )
    .expect("schema");
    let locale = parse_locale(
        "greeting = Hello {fn(gender, label: name) {\n  masculine => dear {label}\n  feminine => kind {name}\n  _ => friend {label}\n}}\n",
    )
    .expect("locale");

    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
        "{diagnostics:#?}"
    );
}

#[test]
fn project_expression_analysis_checks_inline_branch_structure() {
    let schema = parse_schema("greeting(name: String)\n").expect("schema");
    let locale = parse_locale(
        "greeting = {fn(label: missing) { _ => {label} }} {fn() {\n  _ => first\n  later => unreachable\n}}\n",
    )
    .expect("locale");

    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "unknown variable `missing`"));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "branch `later` is unreachable after `_`"));
}

#[test]
fn project_expression_analysis_checks_inline_enum_coverage_and_variants() {
    let schema = parse_schema(
        "enum Gender { masculine, feminine, other }\n\
missing(gender: Gender)\nunknown(gender: Gender)\n",
    )
    .expect("schema");
    let locale = parse_locale(
        "missing = {fn(gender) {\n  masculine => Dear\n  feminine => Kind\n}}\n\
unknown = {fn(gender) {\n  masculine => Dear\n  typo => Wrong\n  _ => Friend\n}}\n",
    )
    .expect("locale");

    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "linguini.incomplete_match"
            && diagnostic.message.contains("missing branch `other`")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "linguini.unknown_enum_variant"
            && diagnostic.message.contains("unknown variant `typo`")
    }));
}

#[test]
fn project_expression_analysis_accepts_inline_plural_and_enum_alias_selectors() {
    let schema = parse_schema(
        "enum Gender { masculine, feminine, other }\n\
type Voice = Gender\n\
greeting(voice: Voice, count: Number)\n",
    )
    .expect("schema");
    let locale = parse_locale(
        "greeting = {fn(voice, Plural(count)) {\n\
  masculine {\n    one => One\n    other => Many\n  }\n\
  feminine {\n    one => One\n    other => Many\n  }\n\
  other {\n    one => One\n    other => Many\n  }\n\
}}\n",
    )
    .expect("locale");

    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
        "{diagnostics:#?}"
    );
}

#[test]
fn project_expression_analysis_walks_inline_bindings_in_every_text_context() {
    let schema = parse_schema("enum Fruit { apple }\n").expect("schema");
    let locale = parse_locale(
        "let broken = {fn(value: missing_variable) { _ => {value} }}\n\
         fn choose(value: String) { _ => {fn(result: missing_function()) { _ => {value} }} }\n\
         impl Fruit {\n\
           apple { label = {fn(value: missing_form) { _ => {value} }} }\n\
         }\n",
    )
    .expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    for message in [
        "unknown variable `missing_variable`",
        "unknown function `missing_function`",
        "unknown variable `missing_form`",
    ] {
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == message),
            "missing diagnostic {message:?}: {diagnostics:#?}"
        );
    }
}

#[test]
fn inline_function_keeps_typed_bindings_out_of_dispatch() {
    let schema = parse_schema(
        "enum Tone { formal, casual }\n\
         greeting(name: String, tone: Tone, count: Number, amount: Decimal)\n",
    )
    .expect("schema");
    let locale = parse_locale(
        "greeting = {fn(tone, Plural(count), label: name, formatted: amount) {\n\
           formal {\n\
             one => {label}: {formatted @number}\n\
             other => {label}: {formatted @number}\n\
           }\n\
           casual {\n\
             _ => {name}: {formatted @number}\n\
           }\n\
         }}\n",
    )
    .expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
        "{diagnostics:#?}"
    );
}

#[test]
fn inline_function_rejects_non_dispatch_selector_types_and_unknown_binding_values() {
    let schema = parse_schema("summary(count: String)\n").expect("schema");
    let locale =
        parse_locale("summary = {fn(count, label: missing) { _ => {label} }}\n").expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "linguini.invalid_dispatch_type"
            && diagnostic.message.contains("got `String`")
    }));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "unknown variable `missing`"));
}

#[test]
fn inline_function_suggests_plural_for_raw_numeric_selectors() {
    let schema = parse_schema("summary(count: Number)\n").expect("schema");
    let locale = parse_locale("summary = {fn(count) { _ => fallback }}\n").expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "linguini.invalid_dispatch_type"
            && diagnostic
                .message
                .contains("wrap numeric values in `Plural(...)`")
    }));
}

#[test]
fn inline_function_branch_scope_inherits_enclosing_parameters_and_globals() {
    let schema = parse_schema("enum Tone { formal, casual }\ngreeting(name: String, tone: Tone)\n")
        .expect("schema");
    let locale = parse_locale(
        "let brand = Linguini\ngreeting = {fn(tone) {\n  formal => {brand}: Hello {name}\n  casual => {brand}: Hi {name}\n}}\n",
    )
    .expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
        "{diagnostics:#?}"
    );
}

#[test]
fn inline_function_binding_only_uses_the_structural_wildcard() {
    let schema = parse_schema("title(value: String)\n").expect("schema");
    let locale =
        parse_locale("title = {fn(label: value) { _ => {label}: {value} }}\n").expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn inline_binding_rhs_values_use_the_outer_scope_simultaneously() {
    let schema = parse_schema("title(name: String)\n").expect("schema");
    let locale =
        parse_locale("title = {fn(first: name, second: first) { _ => {first} {second} }}\n")
            .expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.message == "unknown variable `first`")
            .count(),
        1,
        "{diagnostics:#?}"
    );
}

#[test]
fn nested_inline_functions_inherit_lexical_bindings_and_parameters() {
    let schema = parse_schema("enum Tone { formal, casual }\nsummary(tone: Tone, count: Number)\n")
        .expect("schema");
    let locale = parse_locale(
        "summary = {fn(tone, label: count) {\n  formal => {fn(Plural(count), copy: label) {\n    one => {tone}: {copy}\n    other => {tone}: {copy}\n  }}\n  casual => {label}\n}}\n",
    )
    .expect("locale");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
        "{diagnostics:#?}"
    );
}

#[test]
fn inline_binding_can_call_a_named_function() {
    let schema = parse_schema("enum Tone { formal, casual }\ngreeting(name: String, tone: Tone)\n")
        .expect("schema");
    let locale = parse_locale(
        "fn Greeting(Tone, value: String) {\n  formal => Dear {value}\n  casual => Hi {value}\n}\ngreeting = {fn(tone, greet: Greeting(tone, name)) {\n  formal => {greet}\n  casual => {greet}\n}}\n",
    )
    .expect("locale");

    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
        "{diagnostics:#?}"
    );
}

#[test]
fn locale_analysis_reports_missing_impl_form_fallback() {
    let source = "enum Fruit { apple }\nimpl Fruit {\n  apple {\n    form nom(Plural) {\n      one => apple\n    }\n  }\n}\n";
    let locale = parse_locale(source).expect("locale parses");

    let diagnostics = analyze_locale_file(&locale);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0]
        .message
        .contains("impl `Fruit` variant `apple` form `nom` is missing required `other` branch"));
    let replacement = diagnostics[0].quick_fixes[0]
        .replacement
        .as_ref()
        .expect("replacement");
    assert_eq!(replacement.text, "\n_ => TODO");
    assert_eq!(&source[replacement.span.start..replacement.span.end], "");
    assert!(source[replacement.span.start..].starts_with("\n    }\n"));
}

#[test]
fn locale_coverage_accepts_missing_locale_doc_comment() {
    let schema = parse_schema("/// Translator context\ndelivery\n").expect("schema parses");
    let locale = parse_locale("delivery = Delivered\n").expect("locale parses");

    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn locale_coverage_groups_missing_schema_messages() {
    let schema = parse_schema("delivery\ncounted\n").expect("schema parses");
    let locale = parse_locale("delivery = Delivered\n").expect("locale parses");
    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "locale is missing 1 schema message: `counted`"
    );
    assert_eq!(diagnostics[0].quick_fixes.len(), 2);
    assert_eq!(
        diagnostics[0]
            .quick_fixes
            .iter()
            .map(|fix| fix.title.as_str())
            .collect::<Vec<_>>(),
        [
            "add missing locale message stubs",
            "add locale message stub `counted`"
        ]
    );
}

#[test]
fn locale_coverage_groups_missing_grouped_schema_messages() {
    let schema =
        parse_schema("email_input {\n  label\n  placeholder\n  error\n}\n").expect("schema parses");
    let locale = parse_locale("email_input {\n  label = Email\n}\n").expect("locale parses");
    let diagnostics = analyze_locale_coverage(&schema, &locale);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "locale is missing 2 schema messages: `email_input.placeholder`, `email_input.error`"
    );
    let replacement = diagnostics[0].quick_fixes[0]
        .replacement
        .as_ref()
        .expect("quick fix replacement");
    assert!(replacement.text.contains("email_input {"));
    assert!(replacement.text.contains("  placeholder = TODO"));
    assert!(replacement.text.contains("  error = TODO"));
}

#[test]
fn locale_message_coverage_uses_requested_warning_severity_for_missing_messages() {
    let diagnostics = analyze_locale_coverage_with_options(
        &parse_schema("delivery\n").expect("schema parses"),
        &parse_locale("").expect("empty locale parses"),
        LocaleCoverageOptions {
            missing_message_severity: DiagnosticSeverity::Warning,
            subject: "locale `ru`".to_owned(),
            quick_fix_id: None,
        },
    );

    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostics[0].span, Span::new(0, 0));
    assert_eq!(
        diagnostics[0].message,
        "locale `ru` is missing 1 schema message: `delivery`"
    );
}

#[test]
fn message_coverage_accepts_matching_public_messages() {
    let diagnostics = analyze_message_coverage(
        &[PublicMessage::new("delivery", Span::new(0, 8))],
        &[PublicMessage::new("delivery", Span::new(10, 18))],
    );

    assert!(diagnostics.is_empty());
}

#[test]
fn message_coverage_reports_missing_public_message() {
    let diagnostics =
        analyze_message_coverage(&[PublicMessage::new("delivery", Span::new(0, 8))], &[]);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "missing locale implementation for public message `delivery`"
    );
}

#[test]
fn message_coverage_reports_unknown_public_message() {
    let diagnostics =
        analyze_message_coverage(&[], &[PublicMessage::new("delivery", Span::new(10, 18))]);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "unknown public message implementation `delivery`"
    );
}

#[test]
fn branch_coverage_reports_missing_enum_variant() {
    let diagnostics = analyze_branch_coverage(BranchCoverage {
        subject: "form `Fruit.nom`",
        enum_name: "Fruit",
        variants: vec![
            NamedSpan::new("apple", Span::new(5, 10)),
            NamedSpan::new("pear", Span::new(13, 17)),
        ],
        branches: vec![NamedSpan::new("apple", Span::new(30, 35))],
        span: Span::new(20, 40),
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "form `Fruit.nom` for enum `Fruit` is missing branch `pear`"
    );
    assert_eq!(diagnostics[0].related.len(), 1);
    assert_eq!(diagnostics[0].quick_fixes.len(), 1);
    assert_eq!(diagnostics[0].quick_fixes[0].title, "add branch `pear`");
}

#[test]
fn required_other_branch_reports_missing_fallback() {
    let diagnostics = require_other_branch(
        "plural map `Fruit.nom`",
        &[NamedSpan::new("one", Span::new(0, 3))],
        Span::new(0, 12),
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "plural map `Fruit.nom` is missing required `other` branch"
    );
    assert_eq!(diagnostics[0].quick_fixes.len(), 1);
    assert_eq!(diagnostics[0].quick_fixes[0].title, "add `_` branch");
    assert_eq!(
        diagnostics[0].quick_fixes[0]
            .replacement
            .as_ref()
            .expect("replacement")
            .text,
        "\n_ => TODO"
    );
}

#[test]
fn required_other_branch_accepts_fallback() {
    let diagnostics = require_other_branch(
        "plural map `Fruit.nom`",
        &[NamedSpan::new("other", Span::new(0, 5))],
        Span::new(0, 12),
    );

    assert!(diagnostics.is_empty());
}

#[test]
fn expression_analysis_accepts_valid_delivery_message() {
    let locale = parse_locale("delivery = {Delivered(count, fruit.Gender)} {fruit.nom(count)}\n")
        .expect("locale parses");
    let value = message_value(&locale, "delivery");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            value,
            vec![
                Variable::new("count", "Number", Span::new(0, 0)),
                Variable::new("fruit", "Fruit", Span::new(0, 0)),
            ],
        )],
        functions: vec![FunctionSignature::new("Delivered", 2, Span::new(12, 21))],
        forms: vec![FormSignature::new(
            "Fruit",
            vec![
                FormProperty::new("Gender", Span::new(0, 0)),
                FormProperty::plural("nom", Span::new(0, 0)),
            ],
            Span::new(0, 0),
        )],
    });

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn expression_analysis_reports_unknown_variable() {
    let locale = parse_locale("delivery = {fruit.nom}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![],
        )],
        functions: vec![],
        forms: vec![],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "unknown variable `fruit`");
}

#[test]
fn expression_analysis_reports_unknown_form_property() {
    let locale = parse_locale("delivery = {fruit.acc}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![Variable::new("fruit", "Fruit", Span::new(0, 0))],
        )],
        functions: vec![],
        forms: vec![FormSignature::new(
            "Fruit",
            vec![FormProperty::new("nom", Span::new(0, 0))],
            Span::new(0, 0),
        )],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "unknown form property `acc` on type `Fruit`"
    );
}

#[test]
fn expression_analysis_reports_function_arity() {
    let locale = parse_locale("delivery = {size(fruit)}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![Variable::new("fruit", "Fruit", Span::new(0, 0))],
        )],
        functions: vec![FunctionSignature::new("size", 2, Span::new(0, 0))],
        forms: vec![],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "function `size` expects 2 argument(s), got 1"
    );
}

#[test]
fn expression_analysis_reports_unknown_function_call() {
    let locale = parse_locale("delivery = {missing(fruit)}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![Variable::new("fruit", "Fruit", Span::new(0, 0))],
        )],
        functions: vec![],
        forms: vec![],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "unknown function `missing`");
}

#[test]
fn expression_analysis_accepts_int_plural_argument() {
    let locale =
        parse_locale("delivery = {SizeAdj(size, count, fruit.Gender)}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![
                Variable::new("size", "Size", Span::new(0, 0)),
                Variable::new("count", "Number", Span::new(0, 0)),
                Variable::new("fruit", "Fruit", Span::new(0, 0)),
            ],
        )],
        functions: vec![FunctionSignature::new("SizeAdj", 3, Span::new(0, 0))],
        forms: vec![FormSignature::new(
            "Fruit",
            vec![FormProperty::new("Gender", Span::new(0, 0))],
            Span::new(0, 0),
        )],
    });

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn function_pattern_analysis_reports_dispatch_depth() {
    let locale = parse_locale("form Choose(Gender, Size) {\n  male => ok\n  _ => ok\n}\n")
        .expect("locale parses");
    let diagnostics = analyze_function_patterns(&locale);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "function `Choose` branch pattern expects 2 value(s), got 1"
    );
}

#[test]
fn expression_analysis_reports_ambiguous_implicit_plural_argument() {
    let locale = parse_locale("summary = {fruit.nom}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "summary",
            message_value(&locale, "summary"),
            vec![
                Variable::new("apples", "Number", Span::new(0, 0)),
                Variable::new("pears", "Number", Span::new(0, 0)),
                Variable::new("fruit", "Fruit", Span::new(0, 0)),
            ],
        )],
        functions: vec![],
        forms: vec![FormSignature::new(
            "Fruit",
            vec![FormProperty::plural("nom", Span::new(0, 0))],
            Span::new(0, 0),
        )],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "ambiguous implicit plural argument for `fruit.nom`; pass a numeric argument explicitly"
    );
    assert_eq!(
        diagnostics[0]
            .quick_fixes
            .iter()
            .map(|fix| fix.title.as_str())
            .collect::<Vec<_>>(),
        ["pass `apples` explicitly", "pass `pears` explicitly"]
    );
}

#[test]
fn expression_analysis_accepts_single_implicit_plural_argument() {
    let locale = parse_locale("summary = {fruit.nom}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "summary",
            message_value(&locale, "summary"),
            vec![
                Variable::new("count", "Number", Span::new(0, 0)),
                Variable::new("fruit", "Fruit", Span::new(0, 0)),
            ],
        )],
        functions: vec![],
        forms: vec![FormSignature::new(
            "Fruit",
            vec![FormProperty::plural("nom", Span::new(0, 0))],
            Span::new(0, 0),
        )],
    });

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn reference_cycle_analysis_reports_cycle() {
    let diagnostics = detect_reference_cycles(&[ReferenceNode::new(
        "delivery",
        vec![NamedSpan::new("delivery", Span::new(4, 12))],
        Span::new(0, 12),
    )]);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "cyclic reference `delivery -> delivery`"
    );
}

#[test]
fn zero_argument_call_is_not_treated_as_a_reference() {
    let locale = parse_locale("delivery = {ready()}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![Variable::new("ready", "String", Span::new(0, 0))],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![],
        )],
        functions: vec![FunctionSignature::new("ready", 0, Span::new(0, 0))],
        forms: vec![],
    });

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn explicit_plural_argument_must_be_numeric() {
    let locale = parse_locale("delivery = {Plural(label)}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![Variable::new("label", "String", Span::new(0, 0))],
        )],
        functions: vec![],
        forms: vec![],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "linguini.type_mismatch");
}

#[test]
fn lowercase_plural_call_is_not_an_intrinsic() {
    let locale = parse_locale("delivery = {plural(count)}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![Variable::new("count", "Number", Span::new(0, 0))],
        )],
        functions: vec![],
        forms: vec![],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "unknown function `plural`");
}

#[test]
fn implicit_plural_requires_a_numeric_variable() {
    let locale = parse_locale("delivery = {fruit.nom}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![Variable::new("fruit", "Fruit", Span::new(0, 0))],
        )],
        functions: vec![],
        forms: vec![FormSignature::new(
            "Fruit",
            vec![FormProperty::plural("nom", Span::new(0, 0))],
            Span::new(0, 0),
        )],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "linguini.missing_plural_argument");
}

#[test]
fn typed_function_arguments_are_checked() {
    let locale = parse_locale("delivery = {choose(label)}\n").expect("locale parses");
    let diagnostics = analyze_expressions(ExpressionAnalysis {
        variables: vec![],
        messages: vec![MessageToAnalyze::new(
            "delivery",
            message_value(&locale, "delivery"),
            vec![Variable::new("label", "String", Span::new(0, 0))],
        )],
        functions: vec![FunctionSignature::typed(
            "choose",
            ["Number"],
            Span::new(0, 0),
        )],
        forms: vec![],
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "linguini.type_mismatch");
}

#[test]
fn wildcard_order_duplicates_and_unreachable_arms_are_diagnosed() {
    let diagnostics = analyze_branch_coverage(BranchCoverage {
        subject: "form `Choose`",
        enum_name: "Gender",
        variants: vec![NamedSpan::new("male", Span::new(0, 4))],
        branches: vec![
            NamedSpan::new("_", Span::new(10, 11)),
            NamedSpan::new("male", Span::new(12, 16)),
            NamedSpan::new("male", Span::new(17, 21)),
        ],
        span: Span::new(10, 21),
    });

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "linguini.wildcard_order"));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.lint_name == Some("unreachable_arm")));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "linguini.duplicate_branch"));
}

#[test]
fn complete_enum_with_wildcard_reports_redundant_wildcard_lint() {
    let diagnostics = analyze_branch_coverage(BranchCoverage {
        subject: "form `Choose`",
        enum_name: "Gender",
        variants: vec![NamedSpan::new("male", Span::new(0, 4))],
        branches: vec![
            NamedSpan::new("male", Span::new(10, 14)),
            NamedSpan::new("_", Span::new(15, 16)),
        ],
        span: Span::new(10, 16),
    });

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].lint_name, Some("redundant_wildcard"));
    assert_eq!(diagnostics[0].category, DiagnosticCategory::Lint);
}

#[test]
fn nested_message_groups_use_canonical_paths_and_recursive_stubs() {
    let schema = parse_schema("shop { main { title subtitle } }\n").expect("nested schema parses");
    let locale = parse_locale("shop { main { title = Shop } }\n").expect("nested locale parses");
    let diagnostics = analyze_locale_coverage(&schema, &locale);

    let missing = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("shop.main.subtitle"))
        .expect("nested missing message");
    let replacement = missing.quick_fixes[0]
        .replacement
        .as_ref()
        .expect("recursive stub replacement");
    assert!(replacement
        .text
        .contains("shop {\n  main {\n    subtitle = TODO\n  }\n}"));
}

#[test]
fn reference_cycle_analysis_reports_one_scc_with_all_edges() {
    let diagnostics = detect_reference_cycles(&[
        ReferenceNode::new(
            "a",
            vec![NamedSpan::new("b", Span::new(1, 2))],
            Span::new(0, 2),
        ),
        ReferenceNode::new(
            "b",
            vec![NamedSpan::new("a", Span::new(4, 5))],
            Span::new(3, 5),
        ),
    ]);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].related.len(), 2);
    assert_eq!(diagnostics[0].code, "linguini.reference_cycle");
}

#[test]
fn project_expression_analysis_checks_real_message_calls() {
    let schema = parse_schema("delivery(count: Number)\n").expect("schema parses");
    let locale = parse_locale(
        "fn choose(Number, value: String) { _ => {value} }\ndelivery = {choose(count, count)}\n",
    )
    .expect("locale parses");
    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "linguini.type_mismatch"));
}

#[test]
fn project_expression_analysis_honors_typed_form_map_selectors() {
    let schema = parse_schema(
        "enum Fruit { apple }\nenum Gender { male, other }\ndelivery(fruit: Fruit, gender: Gender)\n",
    )
    .expect("schema parses");
    let locale = parse_locale(
        "impl Fruit {\n\
           apple {\n\
             form label(gender: Gender) {\n\
               male => He\n\
               _ => They\n\
             }\n\
           }\n\
         }\n\
         delivery = {fruit.label(gender)}\n",
    )
    .expect("locale parses");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn project_expression_analysis_resolves_aliases_and_numeric_plural_arguments() {
    let schema = parse_schema(
        "enum Fruit { apple }\n\
         type Produce = Fruit\n\
         type Money = Decimal\n\
         type ShortDate = Date\n\
         delivery(fruit: Produce, count: Number, amount: Money, date: ShortDate)\n",
    )
    .expect("schema parses");
    let locale = parse_locale(
        "impl Fruit {\n\
           apple {\n\
             form label(Plural) {\n\
               one => apple\n\
               _ => apples\n\
             }\n\
           }\n\
         }\n\
         form Delivered(Plural) {\n\
           one => Delivered\n\
           _ => Delivered\n\
         }\n\
         delivery = {Delivered(count)} {fruit.label(count)} {amount @currency} {date @date}\n",
    )
    .expect("locale parses");

    let diagnostics = analyze_project_expressions(&schema, &locale);

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn source_less_diagnostic_has_no_source_span() {
    let diagnostic = Diagnostic::error("project problem", Span::new(0, 0)).without_source();

    assert!(diagnostic.source_span.is_none());
}

#[test]
fn function_style_lints_are_attached_by_name() {
    let locale = parse_locale(
        "enum Gender { male, female }\n\
         fn Same(Gender) {\n\
           male => same\n\
           female => same\n\
         }\n",
    )
    .expect("locale parses");
    let diagnostics = analyze_locale_file(&locale);
    let lint_names = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.lint_name)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(lint_names.contains("fn_without_strings"));
    assert!(lint_names.contains("collapsible_arms"));
}

#[test]
fn non_dispatchable_primitive_is_rejected() {
    let locale = parse_locale("form ByDate(Date) { _ => today }\n").expect("locale parses");
    let diagnostics = analyze_locale_file(&locale);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "linguini.invalid_dispatch_type"));
}

fn message_value(locale: &linguini_syntax::LocaleFile, name: &str) -> linguini_syntax::TextPattern {
    locale
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            linguini_syntax::LocaleDeclaration::Message(message) if message.name.value == name => {
                Some(message.value.clone())
            }
            _ => None,
        })
        .expect("message exists")
}
