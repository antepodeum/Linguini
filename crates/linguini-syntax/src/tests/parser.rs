use crate::{
    parse_locale, parse_locale_in, parse_locale_with_recovery, parse_schema,
    parse_schema_with_recovery, validate_locale_ast, validate_schema_ast, FormEntry, FormatterKind,
    FunctionBranchValue, FunctionKind, LocaleDeclaration, LocaleValue, SchemaDeclaration, SourceId,
    TextBlockMode, TextPart,
};
use std::fs;
use std::path::Path;

#[test]
fn parses_schema_fixture() {
    let source = include_str!("../../../../tests/fixtures/golden/schema/shop.lgs");
    let schema = parse_schema(source).expect("schema fixture parses");

    assert_eq!(schema.declarations.len(), 8);
    match &schema.declarations[0] {
        SchemaDeclaration::Enum(declaration) => {
            assert_eq!(declaration.name.value, "Fruit");
            assert_eq!(declaration.variants.len(), 3);
        }
        other => panic!("expected enum, got {other:?}"),
    }
    match &schema.declarations[4] {
        SchemaDeclaration::Message(declaration) => {
            assert_eq!(declaration.name.value, "delivery");
            assert_eq!(declaration.parameters[0].name.value, "fruit");
            assert_eq!(declaration.parameters[0].ty.value, "Fruit");
        }
        other => panic!("expected message, got {other:?}"),
    }
    match &schema.declarations[7] {
        SchemaDeclaration::Group(declaration) => {
            assert_eq!(declaration.name.value, "email_input");
            assert_eq!(declaration.messages.len(), 3);
        }
        other => panic!("expected group, got {other:?}"),
    }
}

#[test]
fn parses_syntax_coverage_fixtures() {
    let schema = include_str!("../../../../tests/fixtures/golden/syntax/all.lgs");
    let locale = include_str!("../../../../tests/fixtures/golden/syntax/all.lgl");

    assert!(parse_schema(schema).is_ok());
    assert!(parse_locale(locale).is_ok());
}

#[test]
fn schema_parser_recovery_reports_invalid_fixture_diagnostics() {
    let source =
        include_str!("../../../../tests/fixtures/invalid/schema/missing-message-paren.lgs");
    let output = parse_schema_with_recovery(source);

    assert!(!output.errors.is_empty());
    assert!(output.errors.iter().any(|error| error.span.start >= 25));
    assert!(parse_schema(source).is_err());
}

#[test]
fn parser_recovery_reports_all_invalid_fixture_diagnostics() {
    let root = repo_root().join("tests/fixtures/invalid");
    for path in fixture_files(&root) {
        let source = fs::read_to_string(&path).expect("read invalid fixture");
        let extension = path.extension().and_then(|extension| extension.to_str());
        let errors = match extension {
            Some("lgs") => parse_schema_with_recovery(&source).errors,
            Some("lgl") => parse_locale_with_recovery(&source).errors,
            _ => continue,
        };

        assert!(!errors.is_empty(), "{} produced no errors", path.display());
    }
}

#[test]
fn parses_schema_docs_type_alias_annotations_and_groups() {
    let source = r#"/// money amount
type Money = Decimal @currency

type ShortDate = Date @date(style = "short")

nav_label

email_input {
  label
  placeholder
}
"#;
    let schema = parse_schema(source).expect("schema parses");

    assert_eq!(schema.declarations.len(), 4);
    match &schema.declarations[0] {
        SchemaDeclaration::TypeAlias(declaration) => {
            assert_eq!(declaration.docs[0].text, " money amount");
            assert_eq!(declaration.name.value, "Money");
            assert_eq!(declaration.target.value, "Decimal");
            assert_eq!(declaration.annotations[0].kind, FormatterKind::Currency);
        }
        other => panic!("expected type alias, got {other:?}"),
    }
    match &schema.declarations[1] {
        SchemaDeclaration::TypeAlias(declaration) => {
            assert_eq!(declaration.annotations[0].arguments[0].name.value, "style");
            assert_eq!(declaration.annotations[0].arguments[0].value.value, "short");
        }
        other => panic!("expected type alias, got {other:?}"),
    }
    match &schema.declarations[2] {
        SchemaDeclaration::Message(declaration) => {
            assert_eq!(declaration.name.value, "nav_label");
            assert!(declaration.parameters.is_empty());
        }
        other => panic!("expected message, got {other:?}"),
    }
    match &schema.declarations[3] {
        SchemaDeclaration::Group(declaration) => {
            assert_eq!(declaration.name.value, "email_input");
            assert_eq!(declaration.messages.len(), 2);
            assert_eq!(declaration.messages[0].name.value, "label");
        }
        other => panic!("expected group, got {other:?}"),
    }
}

#[test]
fn supports_empty_schema_message_parentheses() {
    let schema = parse_schema("nav_label()\n").expect("zero-argument message");
    let SchemaDeclaration::Message(message) = &schema.declarations[0] else {
        panic!("expected message");
    };
    assert!(message.parameters.is_empty());
}

#[test]
fn supports_empty_locale_function_parentheses_and_preserves_kind() {
    let locale = parse_locale("fn Label() { _ => Label }\nform FormLabel() { _ => Label }\n")
        .expect("zero-argument functions");

    let LocaleDeclaration::Function(function) = &locale.declarations[0] else {
        panic!("expected function");
    };
    assert!(function.parameters.is_empty());
    assert_eq!(function.kind, FunctionKind::Function);

    let LocaleDeclaration::Function(form) = &locale.declarations[1] else {
        panic!("expected form");
    };
    assert!(form.parameters.is_empty());
    assert_eq!(form.kind, FunctionKind::Form);
}

#[test]
fn parses_locale_fixture() {
    let source = include_str!("../../../../tests/fixtures/golden/locale/ru.lgl");
    let locale = parse_locale(source).expect("locale fixture parses");

    assert_eq!(locale.declarations.len(), 10);
    match &locale.declarations[1] {
        LocaleDeclaration::Variable(variable) => {
            assert_eq!(variable.name.value, "cart_label");
        }
        other => panic!("expected variable declaration, got {other:?}"),
    }
    match &locale.declarations[2] {
        LocaleDeclaration::Form(form) => {
            assert_eq!(form.name.value, "Fruit");
            assert_eq!(form.variants.len(), 3);
            let apple_display = &form.variants[0].entries[4];
            match apple_display {
                FormEntry::Attribute(attribute) => match &attribute.value {
                    LocaleValue::Object(entries) => assert_eq!(entries.len(), 2),
                    other => panic!("expected nested object, got {other:?}"),
                },
                other => panic!("expected nested display attribute, got {other:?}"),
            }
        }
        other => panic!("expected form, got {other:?}"),
    }
    match &locale.declarations[6] {
        LocaleDeclaration::Message(message) => {
            assert_eq!(message.name.value, "delivery");
            let placeholders = message
                .value
                .parts
                .iter()
                .filter(|part| matches!(part, TextPart::Placeholder(_)))
                .count();
            assert_eq!(placeholders, 3);
        }
        other => panic!("expected message, got {other:?}"),
    }
}

#[test]
fn locale_parser_recovery_reports_invalid_fixture_diagnostics() {
    let source = include_str!("../../../../tests/fixtures/invalid/locale/broken-placeholder.lgl");
    let output = parse_locale_with_recovery(source);

    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].message, "unterminated placeholder");
    assert!(parse_locale(source).is_err());
}

#[test]
fn parses_locale_impls_forms_functions_messages_and_placeholders() {
    let source = r#"enum Gender { male, female, neuter, other }

impl Fruit {
  apple {
    Gender = neuter
    form nom(Plural) {
      one => яблоко
      few => яблока
      _ => яблок
    }
  }
}

form Adjective(Size, Gender) {
  small {
    male => маленький
    _ => обычное
  }
}

fn note(item: String, Gender) {
  female => Доставлена {item}
  _ => Доставлен {item}
}

delivery = {Adjective(size, fruit.Gender)} {fruit.nom(count)} {amount @currency(code = "USD")}

email_input {
  label = Email
}
"#;
    let locale = parse_locale(source).expect("locale parses");

    assert_eq!(locale.declarations.len(), 6);
    match &locale.declarations[1] {
        LocaleDeclaration::Form(form) => {
            assert_eq!(form.name.value, "Fruit");
            assert_eq!(form.variants[0].name.value, "apple");
            assert_eq!(form.variants[0].entries.len(), 2);
            match &form.variants[0].entries[1] {
                FormEntry::Attribute(attribute) => match &attribute.value {
                    LocaleValue::Map(branches) => assert_eq!(branches.len(), 3),
                    other => panic!("expected map value, got {other:?}"),
                },
                other => panic!("expected form attribute, got {other:?}"),
            }
        }
        other => panic!("expected form, got {other:?}"),
    }
    match &locale.declarations[2] {
        LocaleDeclaration::Function(function) => {
            assert_eq!(function.name.value, "Adjective");
            assert_eq!(function.parameters[0].ty.value, "Size");
            assert!(matches!(
                function.branches[0].value,
                FunctionBranchValue::Dispatch(_)
            ));
        }
        other => panic!("expected form function, got {other:?}"),
    }
    match &locale.declarations[3] {
        LocaleDeclaration::Function(function) => {
            assert_eq!(function.name.value, "note");
            assert_eq!(function.parameters.len(), 2);
            assert_eq!(
                function.parameters[0]
                    .name
                    .as_ref()
                    .expect("named parameter")
                    .value,
                "item"
            );
            assert!(matches!(
                function.branches[1].value,
                FunctionBranchValue::Text(_)
            ));
        }
        other => panic!("expected function, got {other:?}"),
    }
    match &locale.declarations[4] {
        LocaleDeclaration::Message(message) => {
            let placeholders = message
                .value
                .parts
                .iter()
                .filter(|part| matches!(part, TextPart::Placeholder(_)))
                .count();
            assert_eq!(placeholders, 3);
            let annotated_amount = message.value.parts.iter().any(|part| match part {
                TextPart::Placeholder(placeholder) => {
                    placeholder.expression.path[0].value == "amount"
                        && placeholder.expression.annotations[0].kind == FormatterKind::Currency
                }
                TextPart::Text(_) => false,
            });
            assert!(annotated_amount);
        }
        other => panic!("expected message, got {other:?}"),
    }
    match &locale.declarations[5] {
        LocaleDeclaration::Group(group) => {
            assert_eq!(group.name.value, "email_input");
            assert_eq!(group.messages[0].name.value, "label");
        }
        other => panic!("expected group, got {other:?}"),
    }
}

#[test]
fn parses_single_line_form_impl_attribute() {
    let locale = parse_locale("impl Plan { starter { label = Starter } }\n").expect("locale");

    match &locale.declarations[0] {
        LocaleDeclaration::Form(form) => {
            assert_eq!(form.name.value, "Plan");
            assert_eq!(form.variants[0].name.value, "starter");
            match &form.variants[0].entries[0] {
                FormEntry::Attribute(attribute) => {
                    assert_eq!(attribute.name.value, "label");
                    assert!(
                        matches!(&attribute.value, LocaleValue::Text(text) if matches!(
                            text.parts.as_slice(),
                            [TextPart::Text(raw)] if raw.value == "Starter"
                        ))
                    );
                }
                other => panic!("expected attribute, got {other:?}"),
            }
        }
        other => panic!("expected form, got {other:?}"),
    }
}

#[test]
fn parses_locale_override_declaration() {
    let locale = parse_locale("override enum Gender { other }\n").expect("locale parses");

    match &locale.declarations[0] {
        LocaleDeclaration::Override(declaration) => match declaration.as_ref() {
            LocaleDeclaration::Enum(declaration) => {
                assert_eq!(declaration.name.value, "Gender");
                assert_eq!(declaration.span.start, 0);
            }
            other => panic!("expected enum override, got {other:?}"),
        },
        other => panic!("expected override, got {other:?}"),
    }
}

#[test]
fn distinguishes_reference_from_zero_argument_call() {
    let locale = parse_locale("value = {reference} {call()}\n").expect("locale parses");
    let LocaleDeclaration::Message(message) = &locale.declarations[0] else {
        panic!("expected message");
    };
    let expressions: Vec<_> = message
        .value
        .parts
        .iter()
        .filter_map(|part| match part {
            TextPart::Placeholder(placeholder) => Some(&placeholder.expression),
            TextPart::Text(_) => None,
        })
        .collect();

    assert_eq!(expressions[0].kind, crate::ExpressionKind::Reference);
    assert_eq!(expressions[1].kind, crate::ExpressionKind::Call);
    assert!(expressions[1].arguments.is_empty());
}

#[test]
fn parses_recursive_schema_and_locale_groups() {
    let schema = parse_schema("shop {\n  main {\n    checkout {\n      title\n    }\n  }\n}\n")
        .expect("nested schema");
    let SchemaDeclaration::Group(shop) = &schema.declarations[0] else {
        panic!("expected schema group");
    };
    assert_eq!(shop.groups[0].groups[0].messages[0].name.value, "title");

    let locale =
        parse_locale("shop {\n  main {\n    checkout {\n      title = Checkout\n    }\n  }\n}\n")
            .expect("nested locale");
    let LocaleDeclaration::Group(shop) = &locale.declarations[0] else {
        panic!("expected locale group");
    };
    assert_eq!(shop.groups[0].groups[0].messages[0].name.value, "title");
}

#[test]
fn parses_dedented_and_raw_multiline_patterns() {
    let source = concat!(
        "receipt = \"\"\"\r\n",
        "\tOrder {order_id}\r\n",
        "\t  {item_count} items  \r\n",
        "\tThank you.\r\n",
        "\"\"\"\n",
        "wire = raw\"\"\"  leading\r\n",
        "\tindentation is data\r\n",
        "trailing spaces stay  \"\"\"\n",
    );
    let locale = parse_locale(source).expect("multiline locale");

    let LocaleDeclaration::Message(receipt) = &locale.declarations[0] else {
        panic!("expected receipt");
    };
    assert_eq!(receipt.value.mode, TextBlockMode::Dedented);
    assert_eq!(
        render_pattern(&receipt.value),
        "Order {order_id}\n  {item_count} items  \nThank you."
    );

    let LocaleDeclaration::Message(wire) = &locale.declarations[1] else {
        panic!("expected wire");
    };
    assert_eq!(wire.value.mode, TextBlockMode::Raw);
    assert_eq!(
        render_pattern(&wire.value),
        "  leading\r\n\tindentation is data\r\ntrailing spaces stay  "
    );
}

#[test]
fn supports_empty_one_line_and_quote_rich_multiline_blocks() {
    let locale = parse_locale(
        "empty = \"\"\"\"\"\"\none = \"\"\"hello\"\"\"\nquotes = \"\"\"a \"\" b\"\"\"\n",
    )
    .expect("multiline variants");

    let values: Vec<_> = locale
        .declarations
        .iter()
        .map(|declaration| match declaration {
            LocaleDeclaration::Message(message) => render_pattern(&message.value),
            other => panic!("expected message, got {other:?}"),
        })
        .collect();
    assert_eq!(values, ["", "hello", "a \"\" b"]);
}

#[test]
fn parses_literal_brace_escapes() {
    let locale = parse_locale("value = {{name}} = {name}\nblock = \"\"\"{{x}}\"\"\"\n")
        .expect("escaped braces");

    let LocaleDeclaration::Message(value) = &locale.declarations[0] else {
        panic!("expected message");
    };
    assert_eq!(render_pattern(&value.value), "{name} = {name}");
    let LocaleDeclaration::Message(block) = &locale.declarations[1] else {
        panic!("expected message");
    };
    assert_eq!(render_pattern(&block.value), "{x}");
}

#[test]
fn validates_empty_enums_duplicates_names_and_paths() {
    for source in [
        "enum Empty {}\n",
        "enum Fruit { apple, apple }\n",
        "enum fruit { apple }\n",
        "enum Fruit { Apple }\n",
        "delivery(count: Number, count: Number)\n",
        "fn\n",
    ] {
        assert!(parse_schema(source).is_err(), "{source}");
    }

    assert!(parse_locale("value = {a.b.c}\n").is_err());
    assert!(parse_locale("fn note(item: String, item: String) { _ => x }\n").is_err());
    assert!(parse_locale("empty {}\n").is_err());
}

#[test]
fn rejects_group_member_collisions_and_excessive_nesting() {
    assert!(parse_schema("shop { main main { title } }\n").is_err());
    assert!(parse_locale("shop { main = X\nmain { title = Y } }\n").is_err());

    let mut source = String::new();
    for depth in 0..70 {
        source.push_str(&format!("g{depth} {{ "));
    }
    source.push_str("title = X");
    for _ in 0..70 {
        source.push_str(" }");
    }
    assert!(parse_locale(&source).is_err());
}

#[test]
fn rejects_detached_docs_and_does_not_stitch_lex_errors() {
    assert!(parse_schema("/// docs\n\nmessage\n").is_err());

    let output = parse_locale_with_recovery("first = ok\n#\nsecond = fine\n");
    assert!(output.ast.is_none());
    assert!(!output.errors.is_empty());
}

#[test]
fn source_id_propagates_to_all_spans() {
    let source_id = SourceId(42);
    let locale = parse_locale_in("value = text\n", source_id).expect("locale parses");
    assert_eq!(locale.span.source, source_id);
    let LocaleDeclaration::Message(message) = &locale.declarations[0] else {
        panic!("expected message");
    };
    assert_eq!(message.name.span.source, source_id);
    assert_eq!(message.value.span.source, source_id);
}

#[test]
fn explicit_ast_validation_catches_invalid_manual_mutation() {
    let mut schema = parse_schema("enum Fruit { apple }\n").expect("schema");
    let SchemaDeclaration::Enum(item) = &mut schema.declarations[0] else {
        panic!("expected enum");
    };
    item.variants.clear();
    assert!(validate_schema_ast(&schema)
        .iter()
        .any(|error| error.message.contains("at least one variant")));

    let mut locale = parse_locale("value = {name}\n").expect("locale");
    let LocaleDeclaration::Message(message) = &mut locale.declarations[0] else {
        panic!("expected message");
    };
    let TextPart::Placeholder(placeholder) = &mut message.value.parts[0] else {
        panic!("expected placeholder");
    };
    placeholder
        .expression
        .arguments
        .push(placeholder.expression.clone());
    assert!(validate_locale_ast(&locale)
        .iter()
        .any(|error| error.message.contains("reference expression")));
}

#[test]
fn parser_is_panic_free_for_unicode_recovery_corpus() {
    let mut corpus = vec![
        "",
        "\0",
        "🔥",
        "value = {",
        "value = \"\"\"\r\n\tй\n",
        "enum E { а, а }",
        "/// docs\n// detached\nvalue",
        "value = {{literal}}",
    ];
    let large = "message = value\n".repeat(256);
    corpus.push(&large);

    for source in corpus {
        let result = std::panic::catch_unwind(|| {
            let _ = parse_locale_with_recovery(source);
            let _ = parse_schema_with_recovery(source);
        });
        assert!(result.is_ok(), "parser panicked for {source:?}");
    }
}

fn render_pattern(pattern: &crate::TextPattern) -> String {
    pattern
        .parts
        .iter()
        .map(|part| match part {
            TextPart::Text(text) => text.value.clone(),
            TextPart::Placeholder(placeholder) => format!(
                "{{{}}}",
                placeholder
                    .expression
                    .path
                    .iter()
                    .map(|name| name.value.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            ),
        })
        .collect()
}

fn fixture_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).expect("read fixture dir") {
        let path = entry.expect("fixture entry").path();
        if path.is_dir() {
            paths.extend(fixture_files(&path));
        } else {
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}
