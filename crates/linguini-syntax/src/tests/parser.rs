use crate::{
    lexer::{lexer_invocation_count, reset_lexer_invocation_count},
    parse_locale, parse_locale_in, parse_locale_with_recovery, parse_locale_with_tokens,
    parse_schema, parse_schema_with_recovery, parse_schema_with_tokens, validate_locale_ast,
    validate_schema_ast, FormEntry, FormatterKind, FunctionBranchValue, FunctionKind,
    LocaleDeclaration, LocaleValue, SchemaDeclaration, SourceId, TextBlockMode, TextPart,
    TokenKind,
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
fn parsing_with_tokens_lexes_once_and_retains_trivia() {
    reset_lexer_invocation_count();
    let schema =
        parse_schema_with_tokens("// note\n/// docs\ndelivery(count: Number)\n").expect("schema");
    assert_eq!(lexer_invocation_count(), 1);
    assert!(schema
        .tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Comment(_))));
    assert!(schema
        .tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Whitespace)));

    reset_lexer_invocation_count();
    let locale = parse_locale_with_tokens("// note\nmessage = Hello {name}\n").expect("locale");
    assert_eq!(lexer_invocation_count(), 1);
    assert!(locale
        .tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Comment(_))));
    assert!(locale
        .tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::RawText(_))));
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
fn annotations_use_bare_syntax_when_no_arguments_are_present() {
    assert!(parse_schema("type Amount = Number @number\n").is_ok());
    assert!(parse_locale("amount = {value @number}\n").is_ok());

    assert!(parse_schema("type Amount = Number @number()\n").is_err());
    assert!(parse_locale("amount = {value @number()}\n").is_err());
}

#[test]
fn rejects_redundant_empty_schema_message_parentheses() {
    assert!(parse_schema("nav_label()\n").is_err());
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
fn form_attribute_maps_require_canonical_explicit_dispatch_syntax() {
    let implicit = parse_locale(
        "impl Fruit {\n  apple {\n    nom {\n      one => apple\n      other => apples\n    }\n  }\n}\n",
    )
    .expect_err("implicit plural maps are not canonical");
    assert!(implicit.iter().any(|error| error
        .message
        .contains("requires exactly one dispatch parameter")));

    assert!(parse_locale(
        "impl Fruit {\n  apple {\n    nom(Plural) {\n      one => apple\n      other => apples\n    }\n  }\n}\n",
    )
    .is_err());

    parse_locale(
        "impl Fruit {\n  apple {\n    form nom(Plural) {\n      one => apple\n      other => apples\n    }\n  }\n}\n",
    )
    .expect("canonical form attribute syntax parses");
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

fn note(Gender, item: String) {
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
                FormEntry::Attribute(attribute) => {
                    assert_eq!(attribute.parameters.len(), 1);
                    assert_eq!(attribute.parameters[0].ty.value, "Plural");
                    match &attribute.value {
                        LocaleValue::Map(branches) => assert_eq!(branches.len(), 3),
                        other => panic!("expected map value, got {other:?}"),
                    }
                }
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
            assert_eq!(function.parameters[0].ty.value, "Gender");
            assert!(function.parameters[0].name.is_none());
            assert_eq!(
                function.parameters[1]
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
fn parses_inline_function_with_selectors_and_trailing_bindings() {
    let source = "greeting = Hello {fn(gender, name: GreetingName(name)) {\n  masculine => dear, kind {name}\n  feminine => thoughtful {name}\n  _ => friend {name}\n}}!\n";
    let locale = parse_locale(source).expect("inline fn parses");
    let LocaleDeclaration::Message(message) = &locale.declarations[0] else {
        panic!("expected message");
    };
    let expression = message
        .value
        .parts
        .iter()
        .find_map(|part| match part {
            TextPart::Placeholder(placeholder)
                if matches!(
                    &placeholder.expression.kind,
                    crate::ExpressionKind::InlineFunction { .. }
                ) =>
            {
                Some(&placeholder.expression)
            }
            TextPart::Text(_) | TextPart::Placeholder(_) => None,
        })
        .expect("inline expression");

    assert!(expression.arguments.is_empty());
    let crate::ExpressionKind::InlineFunction { inputs, branches } = &expression.kind else {
        panic!("expected inline function");
    };
    assert!(expression.path.is_empty());
    assert_eq!(
        inline_input_shape(inputs),
        [
            "selector:Reference:gender()".to_owned(),
            "binding:name=Call:GreetingName(Reference:name())".to_owned(),
        ]
    );
    assert_eq!(
        branches
            .iter()
            .map(|branch| branch.key.value.as_str())
            .collect::<Vec<_>>(),
        ["masculine", "feminine", "_"]
    );
    let FunctionBranchValue::Text(masculine) = &branches[0].value else {
        panic!("expected text branch");
    };
    assert_eq!(render_pattern(masculine), "dear, kind {name}");
}

#[test]
fn parses_nested_inline_selectors_bindings_and_quoted_delimiters() {
    let source = "summary = {fn(gender, Plural(count), name: GreetingName(tone, count)) {\n  masculine {\n    one => \"Dear, } \"{name}\n    other => Dear {name}\n  }\n  _ {\n    _ => Friend {name}\n  }\n}}\n";
    let locale = parse_locale(source).expect("nested inline fn parses");
    let LocaleDeclaration::Message(message) = &locale.declarations[0] else {
        panic!("expected message");
    };
    let TextPart::Placeholder(placeholder) = &message.value.parts[0] else {
        panic!("expected inline placeholder");
    };
    let expression = &placeholder.expression;

    assert!(expression.arguments.is_empty());
    let crate::ExpressionKind::InlineFunction { inputs, branches } = &expression.kind else {
        panic!("expected inline function");
    };
    assert_eq!(
        inline_input_shape(inputs),
        [
            "selector:Reference:gender()".to_owned(),
            "selector:Call:Plural(Reference:count())".to_owned(),
            "binding:name=Call:GreetingName(Reference:tone(),Reference:count())".to_owned(),
        ]
    );
    let FunctionBranchValue::Dispatch(children) = &branches[0].value else {
        panic!("expected nested dispatch");
    };
    let FunctionBranchValue::Text(one) = &children[0].value else {
        panic!("expected leaf text");
    };
    assert_eq!(render_pattern(one), "Dear, } {name}");
}

#[test]
fn parses_multiline_leaf_inside_inline_function() {
    let source = "summary = {fn(tone, name: name) {\n  formal => \"\"\"\n    Dear {name}\n  \"\"\"\n  _ => Friend\n}}\n";
    let locale = parse_locale(source).expect("inline multiline leaf parses");
    let LocaleDeclaration::Message(message) = &locale.declarations[0] else {
        panic!("expected message");
    };
    let TextPart::Placeholder(placeholder) = &message.value.parts[0] else {
        panic!("expected inline function");
    };
    let crate::ExpressionKind::InlineFunction { branches, .. } = &placeholder.expression.kind
    else {
        panic!("expected inline function");
    };
    let FunctionBranchValue::Text(formal) = &branches[0].value else {
        panic!("expected multiline text branch");
    };

    assert_eq!(formal.mode, TextBlockMode::Dedented);
    assert_eq!(render_pattern(formal), "Dear {name}");
}

#[test]
fn named_and_inline_functions_share_branch_ast_shapes() {
    let source = "fn Greeting(Tone, Plural, name: String) {\n  formal {\n    one => \"Dear, } \"{name}\n    _ => Dear {name}\n  }\n  _ {\n    _ => Friend\n  }\n}\nsummary = {fn(tone, Plural(count), name: name) {\n  formal {\n    one => \"Dear, } \"{name}\n    _ => Dear {name}\n  }\n  _ {\n    _ => Friend\n  }\n}}\n";
    let locale = parse_locale(source).expect("named and inline functions parse");
    let LocaleDeclaration::Function(named) = &locale.declarations[0] else {
        panic!("expected named function");
    };
    let LocaleDeclaration::Message(message) = &locale.declarations[1] else {
        panic!("expected message");
    };
    let TextPart::Placeholder(placeholder) = &message.value.parts[0] else {
        panic!("expected inline function");
    };
    let crate::ExpressionKind::InlineFunction {
        inputs,
        branches: inline_branches,
    } = &placeholder.expression.kind
    else {
        panic!("expected inline function");
    };

    assert_eq!(
        function_parameter_shape(&named.parameters),
        [
            (None, "Tone".to_owned()),
            (None, "Plural".to_owned()),
            (Some("name".to_owned()), "String".to_owned()),
        ]
    );
    assert_eq!(
        inline_input_shape(inputs),
        [
            "selector:Reference:tone()".to_owned(),
            "selector:Call:Plural(Reference:count())".to_owned(),
            "binding:name=Reference:name()".to_owned(),
        ]
    );
    assert_eq!(
        function_branch_shape(&named.branches),
        function_branch_shape(inline_branches)
    );
}

#[test]
fn parses_nested_calls_commas_and_inline_binding_values() {
    let source = "summary = {fn(Outer(first(a, b), second(c)), label: Wrap(left(a, b), right), nested: fn(tone) {\n  formal => inner, value\n  _ => fallback\n}) {\n  formal => {label}\n  _ => outer, fallback\n}}\n";
    let locale = parse_locale(source).expect("nested inline input expressions parse");
    let LocaleDeclaration::Message(message) = &locale.declarations[0] else {
        panic!("expected message");
    };
    let TextPart::Placeholder(placeholder) = &message.value.parts[0] else {
        panic!("expected inline function");
    };
    let crate::ExpressionKind::InlineFunction { inputs, branches } = &placeholder.expression.kind
    else {
        panic!("expected inline function");
    };

    assert_eq!(inputs.len(), 3);
    assert!(matches!(
        &inputs[2],
        crate::InlineFunctionInput::Binding {
            value: crate::Expression {
                kind: crate::ExpressionKind::InlineFunction { .. },
                ..
            },
            ..
        }
    ));
    let FunctionBranchValue::Text(outer_fallback) = &branches[1].value else {
        panic!("expected outer text branch");
    };
    assert_eq!(render_pattern(outer_fallback), "outer, fallback");
}

#[test]
fn validates_inline_input_and_named_function_parameter_order() {
    parse_locale("summary = {fn() { _ => value }}\n")
        .expect("zero-input inline function is valid syntax");
    parse_locale("summary = {fn(value: source) { _ => {value} }}\n")
        .expect("binding-only inline function is valid syntax");
    assert!(
        parse_locale("summary = {fn(tone, name: first, name: second) { _ => value }}\n").is_err()
    );
    assert!(parse_locale("summary = {fn(tone, Name: value) { _ => value }}\n").is_err());

    let selector_after_binding =
        parse_locale("summary = {fn(tone, name: value, Plural(count)) { _ => value }}\n")
            .expect_err("selector after binding must fail validation");
    assert!(
        selector_after_binding.iter().any(|error| {
            error.message == "inline fn selector inputs must precede named bindings"
        }),
        "unexpected diagnostics: {:?}",
        selector_after_binding
    );

    let dispatch_after_payload = parse_locale("fn Broken(value: String, Tone) { _ => value }\n")
        .expect_err("dispatch parameter after payload must fail validation");
    assert!(dispatch_after_payload.iter().any(|error| {
        error.message == "unnamed dispatch parameter `Tone` must precede named payload parameters"
    }));

    parse_locale(
        "fn Wrap(Tone, value: String) { formal => {value} _ => {value} }\nsummary = {fn(tone, Plural(count), greet: Greeting(tone, count)) { _ => {greet} }}\n",
    )
    .expect("leading dispatch inputs and trailing payload bindings are valid");
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
fn rejects_detached_docs_and_recovers_independent_declarations_after_lex_errors() {
    assert!(parse_schema("/// docs\n\nmessage\n").is_err());

    let output = parse_locale_with_recovery("first = ok\n#\nsecond = fine\n");
    assert_eq!(output.errors.len(), 1);
    let locale = output.ast.expect("surviving declarations recover");
    assert_eq!(locale.declarations.len(), 2);
}

#[test]
fn schema_recovery_keeps_independent_declarations_after_lex_errors() {
    let output =
        parse_schema_with_recovery("enum Fruit { apple, apple }\n#\ndelivery(count: Missing)\n");

    assert_eq!(output.errors.len(), 1);
    let schema = output.ast.expect("surviving declarations recover");
    assert_eq!(schema.declarations.len(), 2);
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

fn function_parameter_shape(
    parameters: &[crate::FunctionParameter],
) -> Vec<(Option<String>, String)> {
    parameters
        .iter()
        .map(|parameter| {
            (
                parameter.name.as_ref().map(|name| name.value.clone()),
                parameter.ty.value.clone(),
            )
        })
        .collect()
}

fn inline_input_shape(inputs: &[crate::InlineFunctionInput]) -> Vec<String> {
    inputs
        .iter()
        .map(|input| match input {
            crate::InlineFunctionInput::Binding { name, value, .. } => {
                format!("binding:{}={}", name.value, expression_shape(value))
            }
            crate::InlineFunctionInput::Selector { value, .. } => {
                format!("selector:{}", expression_shape(value))
            }
        })
        .collect()
}

fn expression_shape(expression: &crate::Expression) -> String {
    let kind = match &expression.kind {
        crate::ExpressionKind::Reference => "Reference",
        crate::ExpressionKind::Call => "Call",
        crate::ExpressionKind::InlineFunction { .. } => "InlineFunction",
    };
    let path = expression
        .path
        .iter()
        .map(|name| name.value.as_str())
        .collect::<Vec<_>>()
        .join(".");
    let arguments = expression
        .arguments
        .iter()
        .map(expression_shape)
        .collect::<Vec<_>>()
        .join(",");
    format!("{kind}:{path}({arguments})")
}

fn function_branch_shape(branches: &[crate::FunctionBranch]) -> Vec<String> {
    branches
        .iter()
        .map(|branch| match &branch.value {
            FunctionBranchValue::Text(text) => format!(
                "{}=>{:?}:{}",
                branch.key.value,
                text.mode,
                render_pattern(text)
            ),
            FunctionBranchValue::Dispatch(children) => format!(
                "{}{{{}}}",
                branch.key.value,
                function_branch_shape(children).join("|")
            ),
        })
        .collect()
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
