use crate::{
    ensure_no_unresolved_references, lower_locale, lower_schema, qualify_module, validate_ir,
    IrExpressionKind, IrTextBlockMode, IrTextPart,
};
use linguini_syntax::{parse_locale, parse_schema, LocaleDeclaration};
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
        locale.declarations[1],
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
    assert_eq!(
        paths,
        ["shop.checkout.label", "shop.checkout.SizeWord", "fruit.nom"]
    );
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
