use crate::{
    ensure_no_unresolved_references, lower_locale, lower_schema, qualify_module, validate_ir,
    IrExpressionKind, IrFormEntry, IrInlineFunctionInput, IrTextBlockMode, IrTextPart,
};
use linguini_syntax::{parse_locale, parse_schema, LocaleDeclaration, SchemaDeclaration};
use std::fs;
use std::path::Path;

#[test]
fn schema_ir_snapshot_is_stable() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lgs"
    ))
    .expect("schema");
    let snapshot = format!("{:#?}", lower_schema(&schema));

    assert_snapshot(
        "tests/fixtures/golden/snapshots/ir-schema-shop.txt",
        &snapshot,
    );
}

#[test]
fn locale_ir_snapshot_is_stable() {
    let locale =
        parse_locale(include_str!("../../../tests/fixtures/golden/locale/ru.lgl")).expect("locale");
    let snapshot = format!("{:#?}", lower_locale(&locale));

    assert_snapshot(
        "tests/fixtures/golden/snapshots/ir-locale-ru.txt",
        &snapshot,
    );
}

#[test]
fn lowering_preserves_docs_for_every_declaration_kind() {
    let schema = lower_schema(
        &parse_schema(
            "/// Enum docs\nenum Choice { one, other }\n\
             /// Alias docs\ntype Amount = Number\n\
             /// Message first paragraph\n///\n/// Message second paragraph\nmessage(value: Amount)\n\
             /// Group docs\ngroup {\n  /// Nested docs\n  nested\n}\n",
        )
        .expect("schema"),
    );
    assert_eq!(schema.enums[0].docs, ["Enum docs"]);
    assert_eq!(schema.type_aliases[0].docs, ["Alias docs"]);
    assert_eq!(
        schema.messages[0].docs,
        ["Message first paragraph", "", "Message second paragraph"]
    );
    assert_eq!(schema.messages[1].docs, ["Nested docs"]);
    assert_eq!(schema.groups[0].docs, ["Group docs"]);

    let locale = lower_locale(
        &parse_locale(
            "/// Enum implementation docs\nenum Choice { one, other }\n\
             /// Variable docs\nlet label = Label\n\
             /// Implementation docs\nimpl Choice { one { label = One } other { label = Other } }\n\
             /// Form docs\nform Select(Choice) { one => One other => Other }\n\
             /// Function docs\nfn Pick(Choice) { one => One other => Other }\n\
             /// Message implementation docs\nmessage = Message\n\
             /// Locale group docs\ngroup {\n  /// Nested implementation docs\n  nested = Nested\n}\n",
        )
        .expect("locale"),
    );
    assert_eq!(locale.enums[0].docs, ["Enum implementation docs"]);
    assert_eq!(locale.variables[0].docs, ["Variable docs"]);
    assert_eq!(locale.forms[0].docs, ["Implementation docs"]);
    assert_eq!(locale.functions[0].docs, ["Form docs"]);
    assert_eq!(locale.functions[1].docs, ["Function docs"]);
    assert_eq!(locale.messages[0].docs, ["Message implementation docs"]);
    assert_eq!(locale.messages[1].docs, ["Nested implementation docs"]);
    assert_eq!(locale.groups[0].docs, ["Locale group docs"]);
}

#[test]
fn ir_reference_validation_accepts_golden_delivery_fixture() {
    let schema = parse_schema(include_str!(
        "../../../tests/fixtures/golden/schema/shop.lgs"
    ))
    .expect("schema");
    let locale =
        parse_locale(include_str!("../../../tests/fixtures/golden/locale/ru.lgl")).expect("locale");

    ensure_no_unresolved_references(&lower_schema(&schema), &lower_locale(&locale))
        .expect("references resolved");
}

#[test]
fn ir_reference_validation_reports_unknown_message_and_placeholder() {
    let schema = parse_schema("known(name: String)\n").expect("schema");
    let locale = parse_locale("missing = {unknown}\n").expect("locale");
    let errors = ensure_no_unresolved_references(&lower_schema(&schema), &lower_locale(&locale))
        .expect_err("unresolved references");

    assert!(errors
        .iter()
        .any(|error| error.message == "unresolved message `missing`"));
}

#[test]
fn ir_reference_validation_reports_unknown_placeholder_root() {
    let schema = parse_schema("known(name: String)\n").expect("schema");
    let locale = parse_locale("known = {unknown}\n").expect("locale");
    let errors = ensure_no_unresolved_references(&lower_schema(&schema), &lower_locale(&locale))
        .expect_err("unresolved references");

    assert!(errors
        .iter()
        .any(|error| error.message == "unresolved reference `unknown`"));
}

#[test]
fn lowering_preserves_zero_argument_call_kind_and_source_span() {
    let locale =
        parse_locale("fn ready() { _ => Ready }\nmessage = {ready()}\n").expect("locale parses");
    let module = lower_locale(&locale);
    let body = module.messages[0].body.as_ref().expect("message body");
    let crate::IrTextPart::Placeholder(expression) = &body.parts[0] else {
        panic!("placeholder");
    };

    assert_eq!(expression.kind, IrExpressionKind::Call);
    assert!(expression.arguments.is_empty());
    assert!(expression.span.end > expression.span.start);
}

#[test]
fn lowering_and_validation_preserve_inline_function_dispatch() {
    let schema = lower_schema(
        &parse_schema(
            "enum Gender { masculine, feminine, other }\ngreeting(name: String, gender: Gender)\n",
        )
        .expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale(
            "greeting = {fn(gender, label: name) {\n\
               masculine => Dear {label}\n\
               feminine => Kind {label}\n\
               _ => Friend {label}\n\
             }}\n",
        )
        .expect("locale"),
    );
    let body = locale.messages[0].body.as_ref().expect("message body");
    let IrTextPart::Placeholder(expression) = &body.parts[0] else {
        panic!("inline placeholder");
    };

    assert!(expression.arguments.is_empty());
    let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind else {
        panic!("inline kind");
    };
    let IrInlineFunctionInput::Selector { value, .. } = &inputs[0] else {
        panic!("selector input");
    };
    assert_eq!(value.path, ["gender"]);
    let IrInlineFunctionInput::Binding { name, value, .. } = &inputs[1] else {
        panic!("payload binding");
    };
    assert_eq!(name, "label");
    assert_eq!(value.path, ["name"]);
    assert_eq!(
        branches
            .iter()
            .map(|branch| branch.key.as_str())
            .collect::<Vec<_>>(),
        ["masculine", "feminine", "_"]
    );
    validate_ir(&schema, &locale).expect("inline dispatch validates");
}

#[test]
fn ir_validation_requires_inline_function_enum_coverage() {
    let schema = lower_schema(
        &parse_schema("enum Gender { masculine, feminine, other }\ngreeting(gender: Gender)\n")
            .expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale("greeting = {fn(gender) {\n  masculine => Dear\n  feminine => Kind\n}}\n")
            .expect("locale"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("missing inline branch must fail");

    assert!(errors.iter().any(|error| {
        error.code == "IR034"
            && error
                .message
                .contains("inline fn is not exhaustive for enum `Gender`")
    }));
}

#[test]
fn ir_validation_accepts_nested_enum_and_plural_inline_dispatch() {
    let schema = lower_schema(
        &parse_schema(
            "enum Gender { masculine, feminine, other }\nsummary(gender: Gender, count: Number)\n",
        )
        .expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale(
            "summary = {fn(gender, Plural(count)) {\n  masculine {\n    one => one\n    other => many\n  }\n  feminine {\n    one => one\n    other => many\n  }\n  other {\n    one => one\n    other => many\n  }\n}}\n",
        )
        .expect("locale"),
    );

    validate_ir(&schema, &locale).expect("enum and plural selectors validate");
}

#[test]
fn inline_function_lowering_matches_named_function_input_roles_and_branches() {
    let schema = lower_schema(
        &parse_schema(
            "type Label = String\nenum Tone { formal, casual }\ngreeting(label: Label, tone: Tone)\n",
        )
        .expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale(
            "fn Named(Tone, label: Label) {\n\
               formal => {label}\n\
               casual => {label}\n\
             }\n\
             greeting = {fn(tone, value: label) {\n\
               formal => {value}\n\
               casual => {value}\n\
             }}\n",
        )
        .expect("locale"),
    );
    let body = locale.messages[0].body.as_ref().expect("message body");
    let IrTextPart::Placeholder(expression) = &body.parts[0] else {
        panic!("inline placeholder");
    };
    let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind else {
        panic!("inline kind");
    };

    assert!(matches!(inputs[0], IrInlineFunctionInput::Selector { .. }));
    assert!(matches!(inputs[1], IrInlineFunctionInput::Binding { .. }));
    assert!(locale.functions[0].parameters[0].name.is_none());
    assert_eq!(
        locale.functions[0].parameters[1].name.as_deref(),
        Some("label")
    );
    assert_eq!(
        branches
            .iter()
            .map(|branch| branch.key.as_str())
            .collect::<Vec<_>>(),
        locale.functions[0]
            .branches
            .iter()
            .map(|branch| branch.key.as_str())
            .collect::<Vec<_>>()
    );
    assert!(expression.path.is_empty());
    assert!(expression.arguments.is_empty());
    validate_ir(&schema, &locale).expect("named and inline function semantics match");
}

#[test]
fn inline_function_plural_selector_accepts_numeric_outer_value() {
    let schema = lower_schema(&parse_schema("summary(count: Decimal)\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale(
            "summary = {fn(Plural(count)) {\n\
               one => one {count}\n\
               other => many {count}\n\
             }}\n",
        )
        .expect("locale"),
    );

    validate_ir(&schema, &locale).expect("Plural selector accepts Decimal");
}

#[test]
fn inline_function_rejects_unresolved_binding_rhs() {
    let schema = lower_schema(&parse_schema("summary(count: String)\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale("summary = {fn(label: missing) {\n  _ => {label}\n}}\n").expect("locale"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("binding RHS must resolve");
    assert!(errors
        .iter()
        .any(|error| error.code == "IR015" && error.message == "unresolved reference `missing`"));
}

#[test]
fn inline_function_branch_is_a_lexical_closure_over_outer_parameters() {
    let schema = lower_schema(
        &parse_schema("enum Tone { formal, casual }\ngreeting(tone: Tone, hidden: String)\n")
            .expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale("greeting = {fn(tone) {\n  formal => {hidden}\n  casual => visible\n}}\n")
            .expect("locale"),
    );

    validate_ir(&schema, &locale).expect("outer parameters remain visible in branch text");
}

#[test]
fn inline_function_binding_can_evaluate_a_global_in_lexical_scope() {
    let schema = lower_schema(&parse_schema("greeting\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale("let label = global\ngreeting = {fn(value: label) {\n  _ => {value}\n}}\n")
            .expect("locale"),
    );

    validate_ir(&schema, &locale).expect("global can initialize local payload binding");
}

#[test]
fn binding_only_and_zero_input_inline_functions_use_the_structural_wildcard() {
    let schema = lower_schema(&parse_schema("greeting(name: String)\nplain\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale(
            "greeting = {fn(label: name) {\n  _ => {label} / {name}\n}}\n\
             plain = {fn() {\n  _ => ready\n}}\n",
        )
        .expect("locale"),
    );

    validate_ir(&schema, &locale).expect("selectorless inline functions validate");
}

#[test]
fn inline_bindings_have_simultaneous_rhs_scope() {
    let schema = lower_schema(&parse_schema("greeting(name: String)\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale(
            "greeting = {fn(first: name, second: first) {\n  _ => {first} {second}\n}}\n",
        )
        .expect("locale"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("sibling RHS must stay invisible");
    assert!(errors
        .iter()
        .any(|error| error.code == "IR015" && error.message == "unresolved reference `first`"));
}

#[test]
fn ir_boundary_rejects_selector_after_inline_binding() {
    let schema = lower_schema(
        &parse_schema("enum Tone { formal, casual }\ngreeting(name: String, tone: Tone)\n")
            .expect("schema"),
    );
    let mut locale = lower_locale(
        &parse_locale(
            "greeting = {fn(tone, label: name) {\n  formal => {label}\n  casual => {label}\n}}\n",
        )
        .expect("locale"),
    );
    let body = locale.messages[0].body.as_mut().expect("body");
    let IrTextPart::Placeholder(expression) = &mut body.parts[0] else {
        panic!("inline placeholder");
    };
    let IrExpressionKind::InlineFunction { inputs, .. } = &mut expression.kind else {
        panic!("inline function");
    };
    inputs.swap(0, 1);

    let errors = validate_ir(&schema, &locale).expect_err("input order must be validated in IR");
    assert!(errors.iter().any(|error| {
        error.code == "IR038"
            && error
                .message
                .contains("selectors must precede named payload bindings")
    }));
}

#[test]
fn named_function_dispatch_uses_only_leading_unnamed_parameters() {
    let schema = lower_schema(
        &parse_schema("enum Tone { formal, casual }\ngreeting(tone: Tone)\n").expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale(
            "fn Payload(tone: Tone) {\n  _ => {tone}\n}\n\
             greeting = {Payload(tone)}\n",
        )
        .expect("locale"),
    );

    validate_ir(&schema, &locale).expect("named enum payload does not add dispatch");
}

#[test]
fn ir_boundary_rejects_named_payload_before_dispatch_parameter() {
    let schema = lower_schema(
        &parse_schema("enum Tone { formal, casual }\ngreeting(label: String, tone: Tone)\n")
            .expect("schema"),
    );
    let mut locale = lower_locale(
        &parse_locale(
            "fn Ordered(Tone, label: String) {\n\
               formal => {label}\n\
               casual => {label}\n\
             }\n\
             greeting = {Ordered(tone, label)}\n",
        )
        .expect("locale"),
    );
    locale.functions[0].parameters.swap(0, 1);

    let errors = validate_ir(&schema, &locale).expect_err("parameter order must validate in IR");
    assert!(errors.iter().any(|error| {
        error.code == "IR038"
            && error
                .message
                .contains("dispatch parameters must precede named payload parameters")
    }));
}

#[test]
fn ir_boundary_rejects_implicit_plural_form_maps() {
    let schema = lower_schema(&parse_schema("enum Fruit { apple }\n").expect("schema"));
    let mut locale = lower_locale(
        &parse_locale(
            "impl Fruit {\n\
               apple {\n\
                 form nom(Plural) {\n\
                   one => apple\n\
                   other => apples\n\
                 }\n\
               }\n\
             }\n",
        )
        .expect("locale"),
    );
    let IrFormEntry::Attribute { parameters, .. } = &mut locale.forms[0].variants[0].entries[0]
    else {
        panic!("form attribute");
    };
    parameters.clear();

    let errors = validate_ir(&schema, &locale).expect_err("implicit dispatch must fail in IR");
    assert!(errors.iter().any(|error| {
        error.code == "IR039"
            && error
                .message
                .contains("requires exactly one dispatch parameter, got 0")
    }));
}

#[test]
fn inline_function_closes_over_named_form_attribute_parameters() {
    let schema = lower_schema(&parse_schema("enum Fruit { apple }\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale(
            "enum Gender { male, female }\n\
             impl Fruit {\n\
               apple {\n\
                 form label(gender: Gender) {\n\
                   male => {fn(gender) {\n\
                     male => male\n\
                     female => female\n\
                   }}\n\
                   female => female\n\
                 }\n\
               }\n\
             }\n",
        )
        .expect("locale"),
    );

    validate_ir(&schema, &locale).expect("form attribute parameter is in lexical scope");
}

#[test]
fn ir_validation_rejects_unknown_inline_enum_branch_even_with_wildcard() {
    let schema = lower_schema(
        &parse_schema("enum Gender { masculine, other }\ngreeting(gender: Gender)\n")
            .expect("schema"),
    );
    let locale = lower_locale(
        &parse_locale("greeting = {fn(gender) {\n  typo => Wrong\n  _ => Friend\n}}\n")
            .expect("locale"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("unknown branch must fail");

    assert!(errors.iter().any(|error| {
        error.code == "IR036"
            && error
                .message
                .contains("unknown enum `Gender` branch `typo`")
    }));
}

#[test]
fn ir_validation_rejects_inline_branches_after_wildcard() {
    let schema = lower_schema(&parse_schema("greeting(value: String)\n").expect("schema"));
    let locale = lower_locale(
        &parse_locale("greeting = {fn(value) {\n  _ => Friend\n  later => Unreachable\n}}\n")
            .expect("locale"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("wildcard ordering must fail");

    assert!(errors
        .iter()
        .any(|error| error.code == "IR037" && error.message.contains("unreachable after `_`")));
}

#[test]
fn lowering_preserves_recursive_group_paths() {
    let schema = parse_schema("shop { main { title } local { title } }\n").expect("schema parses");
    let locale = parse_locale("shop { main { title = Main } local { title = Local } }\n")
        .expect("locale parses");

    assert_eq!(
        lower_schema(&schema)
            .messages
            .iter()
            .map(|message| message.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.main.title", "shop.local.title"]
    );
    assert_eq!(
        lower_locale(&locale)
            .messages
            .iter()
            .map(|message| message.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.main.title", "shop.local.title"]
    );
}

#[test]
fn lowering_preserves_group_metadata_for_nested_and_empty_groups() {
    let schema = parse_schema(
        "/// Root docs\nroot {\n  /// Empty docs\n  empty {}\n  /// Nested docs\n  nested {\n    title\n  }\n}\n",
    )
    .expect("schema parses");
    let SchemaDeclaration::Group(root) = &schema.declarations()[0] else {
        panic!("expected schema group");
    };
    let module = lower_schema(&schema);

    assert_eq!(
        module
            .groups
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["root", "root.empty", "root.nested"]
    );
    assert_eq!(module.groups[0].docs, ["Root docs"]);
    assert_eq!(module.groups[1].docs, ["Empty docs"]);
    assert_eq!(module.groups[2].docs, ["Nested docs"]);
    assert_eq!(module.groups[0].span, root.span);
    assert_eq!(module.groups[1].span, root.groups[0].span);
    assert_eq!(module.groups[2].span, root.groups[1].span);
    assert!(module
        .messages
        .iter()
        .any(|message| message.name == "root.nested.title"));
}

#[test]
fn lowering_preserves_locale_group_docs_and_spans() {
    let locale = parse_locale(
        "/// Root docs\nroot {\n  /// Nested docs\n  nested {\n    title = Title\n  }\n}\n",
    )
    .expect("locale parses");
    let LocaleDeclaration::Group(root) = &locale.declarations()[0] else {
        panic!("expected locale group");
    };
    let module = lower_locale(&locale);

    assert_eq!(
        module
            .groups
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["root", "root.nested"]
    );
    assert_eq!(module.groups[0].docs, ["Root docs"]);
    assert_eq!(module.groups[1].docs, ["Nested docs"]);
    assert_eq!(module.groups[0].span, root.span);
    assert_eq!(module.groups[1].span, root.groups[0].span);
    assert_eq!(module.messages[0].name, "root.nested.title");
}

#[test]
fn project_namespace_keeps_local_group_named_like_filesystem_namespace_distinct() {
    let schema = parse_schema("shop { title }\n").expect("schema parses");
    let mut module = lower_schema(&schema);
    qualify_module(&mut module, "shop.checkout");

    assert_eq!(module.groups[0].name, "shop.checkout.shop");
    assert_eq!(module.messages[0].name, "shop.checkout.shop.title");
    assert_eq!(module.origins[0].name, "shop.checkout.shop");
}

#[test]
fn lowering_preserves_multiline_mode_after_normalization() {
    let locale =
        parse_locale("dedented = \"\"\"\n  Hello\n\"\"\"\nraw_value = raw\"\"\"  exact\n\"\"\"\n")
            .expect("locale parses");
    let module = lower_locale(&locale);

    assert_eq!(
        module.messages[0].body.as_ref().expect("body").mode,
        IrTextBlockMode::Dedented
    );
    assert_eq!(
        module.messages[1].body.as_ref().expect("body").mode,
        IrTextBlockMode::Raw
    );
}

#[test]
fn locale_enums_survive_lowering() {
    let locale = parse_locale("enum Gender { male, female }\n").expect("locale parses");
    let module = lower_locale(&locale);

    assert_eq!(module.enums.len(), 1);
    assert_eq!(module.enums[0].name, "Gender");
}

#[test]
fn validated_ir_rejects_duplicate_vectors() {
    let schema_file = parse_schema("message\n").expect("schema parses");
    let mut schema = lower_schema(&schema_file);
    schema.messages.push(schema.messages[0].clone());
    let locale = lower_locale(&parse_locale("message = ok\n").expect("locale parses"));

    let errors = validate_ir(&schema, &locale).expect_err("duplicate IR is invalid");
    assert!(errors.iter().any(|error| error.code == "IR001"));
}

#[test]
fn validated_ir_walks_form_text_and_nested_objects() {
    let schema = lower_schema(&parse_schema("enum Fruit { apple }\n").expect("schema parses"));
    let locale = lower_locale(
        &parse_locale("impl Fruit { apple { display { short = {missing} } } }\n")
            .expect("locale parses"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("nested reference is invalid");
    assert!(errors
        .iter()
        .any(|error| error.message == "unresolved reference `missing`"));
}

#[test]
fn validated_ir_reports_variable_cycle_once_with_edges() {
    let schema = lower_schema(&parse_schema("").expect("schema parses"));
    let locale = lower_locale(&parse_locale("let a = {b}\nlet b = {a}\n").expect("locale parses"));

    let errors = validate_ir(&schema, &locale).expect_err("cycle is invalid");
    let cycles = errors
        .iter()
        .filter(|error| error.code == "IR033")
        .collect::<Vec<_>>();
    assert_eq!(cycles.len(), 1, "{errors:#?}");
    assert_eq!(cycles[0].related.len(), 2);
}

#[test]
fn validated_ir_rejects_formatter_type_mismatch() {
    let schema = lower_schema(&parse_schema("message(value: String)\n").expect("schema parses"));
    let locale = lower_locale(
        &parse_locale("message = {value @date(style = \"short\")}\n").expect("locale parses"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("formatter is invalid");
    assert!(errors.iter().any(|error| error.code == "IR032"));
}

#[test]
fn validated_ir_rejects_non_exhaustive_plural_dispatch() {
    let schema = lower_schema(&parse_schema("").expect("schema parses"));
    let locale = lower_locale(
        &parse_locale("form Count(Plural) {\n  one => item\n}\n").expect("locale parses"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("dispatch must be exhaustive");
    assert!(errors.iter().any(|error| {
        error.code == "IR034"
            && error.message
                == "function `Count` is not exhaustive for `Plural`; add an `other` or `_` branch"
    }));
}

#[test]
fn validated_ir_accepts_explicitly_exhaustive_enum_dispatch() {
    let schema = lower_schema(&parse_schema("").expect("schema parses"));
    let locale = lower_locale(
        &parse_locale(
            "enum Tone { formal, casual }\n\
             form Greeting(Tone) {\n\
               formal => Hello\n\
               casual => Hi\n\
             }\n",
        )
        .expect("locale parses"),
    );

    validate_ir(&schema, &locale).expect("all enum variants are covered");
}

#[test]
fn validated_ir_rejects_non_exhaustive_nested_enum_dispatch() {
    let schema = lower_schema(&parse_schema("").expect("schema parses"));
    let locale = lower_locale(
        &parse_locale(
            "enum Gender { male, female }\n\
             form Delivered(Plural, Gender) {\n\
               one {\n\
                 male => Delivered\n\
               }\n\
               _ => Delivered\n\
             }\n",
        )
        .expect("locale parses"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("nested dispatch must be exhaustive");
    assert!(errors.iter().any(|error| {
        error.code == "IR034"
            && error.message
                == "function `Delivered` is not exhaustive for enum `Gender`; missing branch `female`"
    }));
}

#[test]
fn validated_ir_rejects_non_exhaustive_impl_map() {
    let schema = lower_schema(&parse_schema("").expect("schema parses"));
    let locale = lower_locale(
        &parse_locale(
            "enum Fruit { apple }\n\
             impl Fruit {\n\
               apple {\n\
                 form nom(Plural) {\n\
                   one => apple\n\
                 }\n\
               }\n\
             }\n",
        )
        .expect("locale parses"),
    );

    let errors = validate_ir(&schema, &locale).expect_err("impl map must be exhaustive");
    assert!(errors.iter().any(|error| {
        error.code == "IR034"
            && error.message
                == "form `Fruit.nom` is not exhaustive for `Plural`; add an `other` or `_` branch"
    }));
}

#[test]
fn override_resolution_replaces_value_but_preserves_provenance() {
    let locale =
        parse_locale("message = first\noverride message = second\n").expect("locale parses");
    assert!(matches!(
        locale.declarations()[1],
        LocaleDeclaration::Override(_)
    ));
    let module = lower_locale(&locale);

    assert_eq!(module.messages.len(), 1);
    assert_eq!(
        module
            .origins
            .iter()
            .filter(|origin| origin.name == "message")
            .count(),
        2
    );
    assert!(module
        .origins
        .last()
        .is_some_and(|origin| origin.is_override));
}

#[test]
fn override_group_removes_descendants_but_preserves_group_provenance() {
    let locale = parse_locale(
        "shop {\n  title = First\n  nested { old = Old }\n}\n\
         override shop {\n  title = Second\n}\n",
    )
    .expect("locale parses");
    let module = lower_locale(&locale);

    assert_eq!(
        module
            .groups
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>(),
        ["shop"]
    );
    assert_eq!(
        module
            .messages
            .iter()
            .map(|message| message.name.as_str())
            .collect::<Vec<_>>(),
        ["shop.title"]
    );
    let origins = module
        .origins
        .iter()
        .map(|origin| (origin.name.as_str(), origin.is_override))
        .collect::<Vec<_>>();
    assert_eq!(
        origins,
        [
            ("shop", false),
            ("shop.title", false),
            ("shop.nested", false),
            ("shop.nested.old", false),
            ("shop", true),
            ("shop.title", true),
        ]
    );
}

#[test]
fn validation_rejects_duplicate_and_cross_kind_group_paths() {
    let duplicate =
        lower_schema(&parse_schema("one { first }\none { second }\n").expect("schema parses"));
    let locale = lower_locale(&parse_locale("").expect("locale parses"));
    let errors = validate_ir(&duplicate, &locale).expect_err("duplicate groups must fail");
    assert!(errors.iter().any(|error| error.code == "IR001"));

    let cross_kind = lower_schema(&parse_schema("same\nsame { child }\n").expect("schema parses"));
    let errors = validate_ir(&cross_kind, &locale).expect_err("cross-kind path must fail");
    assert!(errors
        .iter()
        .any(|error| { error.code == "IR002" && error.message.contains("same") }));
}

#[test]
fn project_namespace_qualifies_declarations_types_references_and_origins() {
    let schema = parse_schema(
        "enum Size { small, big }\ntype ChosenSize = Size\nsummary(size: ChosenSize)\n",
    )
    .expect("schema parses");
    let mut schema = lower_schema(&schema);
    qualify_module(&mut schema, "shop.checkout");

    assert_eq!(schema.enums[0].name, "shop.checkout.Size");
    assert_eq!(schema.type_aliases[0].name, "shop.checkout.ChosenSize");
    assert_eq!(schema.type_aliases[0].target, "shop.checkout.Size");
    assert_eq!(schema.messages[0].name, "shop.checkout.summary");
    assert_eq!(
        schema.messages[0].parameters[0].ty,
        "shop.checkout.ChosenSize"
    );
    assert!(schema
        .origins
        .iter()
        .all(|origin| origin.name.starts_with("shop.checkout.")));

    let locale = parse_locale(
        "form SizeWord(Size) { small => small, big => big }\n\
         let label = Size\n\
         summary = {label}: {SizeWord(size)} {fruit.nom(count)}\n",
    )
    .expect("locale parses");
    let mut locale = lower_locale(&locale);
    qualify_module(&mut locale, "shop.checkout");

    assert_eq!(locale.functions[0].name, "shop.checkout.SizeWord");
    assert_eq!(locale.functions[0].parameters[0].ty, "shop.checkout.Size");
    assert_eq!(locale.variables[0].name, "shop.checkout.label");
    assert_eq!(locale.messages[0].name, "shop.checkout.summary");
    let paths = locale.messages[0]
        .body
        .as_ref()
        .expect("message body")
        .parts
        .iter()
        .filter_map(|part| match part {
            IrTextPart::Placeholder(expression) => Some(expression.path.join(".")),
            IrTextPart::Text(_) => None,
        })
        .collect::<Vec<_>>();
    let path_parts = locale.messages[0]
        .body
        .as_ref()
        .expect("message body")
        .parts
        .iter()
        .filter_map(|part| match part {
            IrTextPart::Placeholder(expression) => Some(expression.path.clone()),
            IrTextPart::Text(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        ["shop.checkout.label", "shop.checkout.SizeWord", "fruit.nom"]
    );
    assert_eq!(
        path_parts,
        [
            vec!["shop.checkout.label"],
            vec!["shop.checkout.SizeWord"],
            vec!["fruit", "nom"]
        ]
    );
}

#[test]
fn project_namespace_qualifies_inline_input_calls_but_keeps_bindings_local() {
    let locale = parse_locale(
        "enum Tone { formal, casual }\n\
         fn Wrap(value: String) { _ => {value} }\n\
         greeting = {fn(tone, label: Wrap(name)) {\n\
           formal => {label}\n\
           casual => {label}\n\
         }}\n",
    )
    .expect("locale parses");
    let mut locale = lower_locale(&locale);
    qualify_module(&mut locale, "shop.checkout");

    let body = locale.messages[0].body.as_ref().expect("message body");
    let IrTextPart::Placeholder(expression) = &body.parts[0] else {
        panic!("inline placeholder");
    };
    let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind else {
        panic!("inline kind");
    };

    let IrInlineFunctionInput::Selector { value, .. } = &inputs[0] else {
        panic!("selector input");
    };
    assert_eq!(value.path, ["tone"]);
    let IrInlineFunctionInput::Binding { name, value, .. } = &inputs[1] else {
        panic!("payload binding");
    };
    assert_eq!(name, "label");
    assert_eq!(value.path, ["shop.checkout.Wrap"]);
    let crate::IrFunctionBranchValue::Text(text) = &branches[0].value else {
        panic!("text branch");
    };
    let IrTextPart::Placeholder(reference) = &text.parts[0] else {
        panic!("capture reference");
    };
    assert_eq!(reference.path, ["label"]);
}

#[test]
fn reference_validation_resolves_qualified_functions_and_variables() {
    let mut schema = lower_schema(
        &parse_schema(
            "enum Size { small, big }\n\
             enum Fruit { apple }\n\
             enum Gender { male, other }\n\
             summary(size: Size, fruit: Fruit)\n",
        )
        .expect("schema parses"),
    );
    qualify_module(&mut schema, "shop.checkout");
    let mut locale = lower_locale(
        &parse_locale(
            "form SizeWord(Size) { small => small\n_ => big }\n\
             impl Fruit { apple { Gender = male } }\n\
             let label = Size\n\
             summary = {label}: {SizeWord(size)} {fruit.Gender}\n",
        )
        .expect("locale parses"),
    );
    qualify_module(&mut locale, "shop.checkout");

    ensure_no_unresolved_references(&schema, &locale)
        .expect("qualified locale dependencies resolve");
}

fn assert_snapshot(path: &str, snapshot: &str) {
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        fs::write(repo_root().join(path), snapshot).expect("write snapshot");
    }

    let expected = fs::read_to_string(repo_root().join(path)).expect("read snapshot");
    assert_eq!(snapshot, expected);
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}
