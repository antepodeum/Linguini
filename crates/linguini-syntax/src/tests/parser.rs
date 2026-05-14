use crate::{
    parse_locale, parse_locale_with_recovery, parse_schema, parse_schema_with_recovery, FormEntry,
    FunctionBranchValue, LocaleDeclaration, LocaleValue, SchemaDeclaration, TextPart,
};

#[test]
fn parses_schema_fixture() {
    let source = include_str!("../../../../tests/fixtures/golden/schema/shop.lqs");
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
fn schema_parser_recovery_reports_invalid_fixture_diagnostics() {
    let source =
        include_str!("../../../../tests/fixtures/invalid/schema/missing-message-paren.lqs");
    let output = parse_schema_with_recovery(source);

    assert!(!output.errors.is_empty());
    assert!(output.errors.iter().any(|error| error.span.start >= 25));
    assert!(parse_schema(source).is_err());
}

#[test]
fn parses_schema_docs_type_alias_annotations_and_groups() {
    let source = r#"/// money amount
type Money = Decimal @currency

type ShortDate = Date @date(style = "short")

email_input {
  label
  placeholder
}
"#;
    let schema = parse_schema(source).expect("schema parses");

    assert_eq!(schema.declarations.len(), 3);
    match &schema.declarations[0] {
        SchemaDeclaration::TypeAlias(declaration) => {
            assert_eq!(declaration.docs[0].text, " money amount");
            assert_eq!(declaration.name.value, "Money");
            assert_eq!(declaration.target.value, "Decimal");
            assert_eq!(declaration.annotations[0].name.value, "currency");
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
        SchemaDeclaration::Group(declaration) => {
            assert_eq!(declaration.name.value, "email_input");
            assert_eq!(declaration.messages.len(), 2);
            assert_eq!(declaration.messages[0].name.value, "label");
        }
        other => panic!("expected group, got {other:?}"),
    }
}

#[test]
fn parses_locale_fixture() {
    let source = include_str!("../../../../tests/fixtures/golden/locale/ru.lgl");
    let locale = parse_locale(source).expect("locale fixture parses");

    assert_eq!(locale.declarations.len(), 9);
    match &locale.declarations[1] {
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
    match &locale.declarations[5] {
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
                        && placeholder.expression.annotations[0].name.value == "currency"
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
fn parses_locale_override_declaration() {
    let locale = parse_locale("override enum Gender { other }\n").expect("locale parses");

    match &locale.declarations[0] {
        LocaleDeclaration::Override(declaration) => match declaration.as_ref() {
            LocaleDeclaration::Enum(declaration) => {
                assert_eq!(declaration.name.value, "Gender")
            }
            other => panic!("expected enum override, got {other:?}"),
        },
        other => panic!("expected override, got {other:?}"),
    }
}
