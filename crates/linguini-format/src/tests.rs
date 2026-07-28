use super::{
    engine, format_path_source, format_source, semantics::FormatSemantics, FormatError,
    FormatOptions, SourceKind, CRATE_PURPOSE,
};
use linguini_syntax::{
    lex_schema_with_recovery, lex_with_recovery, parse_locale, Expression, LocaleDeclaration,
    SourceId, Span, TextPart, TextPattern, Token, TokenKind,
};
use std::path::Path;

#[test]
fn crate_has_unit_test_structure() {
    assert_eq!(CRATE_PURPOSE, "Linguini source formatting");
}

#[test]
fn formats_schema_idempotently_and_preserves_doc_comments() {
    let source = "/// Delivery label\ndelivery(count:Number)\nemail_input{\n/// Label\nlabel\n}\n";
    let formatted =
        format_source(SourceKind::Schema, source, &FormatOptions::default()).expect("format");
    let second = format_source(SourceKind::Schema, &formatted, &FormatOptions::default())
        .expect("format again");

    assert_eq!(
        formatted,
        "/// Delivery label\ndelivery(count: Number)\nemail_input {\n  /// Label\n  label\n}\n"
    );
    assert_eq!(formatted, second);
}

#[test]
fn formats_locale_idempotently_and_preserves_ordinary_comments() {
    let source = "enum Fruit{apple,pear}\n// keep me\nimpl Fruit{\napple{\nform nom (Plural){\none=>яблоко\n_=>яблок\n}\n}\n}\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");
    let second = format_source(SourceKind::Locale, &formatted, &FormatOptions::default())
        .expect("format again");

    assert_eq!(
        formatted,
        "enum Fruit { apple, pear }\n// keep me\nimpl Fruit {\n  apple {\n    form nom(Plural) {\n      one => яблоко\n      _   => яблок\n    }\n  }\n}\n"
    );
    assert_eq!(formatted, second);
}

#[test]
fn preserves_raw_text_spacing_and_single_blank_line() {
    let source = "message = Hello  {name}  !\n\nnext = Bye\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert_eq!(formatted, source);
}

#[test]
fn preserves_adjacent_locale_placeholders_without_inner_spaces() {
    let source = "message = {first}{second} {amount @currency(code = \"RUB\")}\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert_eq!(formatted, source);
}

#[test]
fn preserves_punctuation_after_locale_placeholders_without_extra_space() {
    let source =
        "point = Точка здесь {val}.\nlist = Значения: {a}, {b}; {c}!\nquestion = Это {value}?\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert_eq!(formatted, source);
}

#[test]
fn preserves_branch_arm_placeholders_without_inner_spaces_or_punctuation_gaps() {
    let source = "form Product(Plural) {\n  one => {amount}{item}.\n  _ => {amount} {item}!\n}\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert_eq!(
        formatted,
        "form Product(Plural) {\n  one => {amount}{item}.\n  _   => {amount} {item}!\n}\n"
    );
}

#[test]
fn collapses_multiple_blank_lines_to_one_blank_line() {
    let source = "first = One\n\n\nsecond = Two\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert_eq!(formatted, "first = One\n\nsecond = Two\n");
}

#[test]
fn collapses_structural_newlines_in_form_headers_and_arguments() {
    let source = "form \n\nDelivered(\n  Plural,\n  Gender\n) {\n  one {\n    male   => Доставлен\n    female => Доставлена\n    neuter => Доставлено\n    _      => Доставлено\n  }\n  _ => Доставлены\n}\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert_eq!(
        formatted,
        "form Delivered(Plural, Gender) {\n  one {\n    male   => Доставлен\n    female => Доставлена\n    neuter => Доставлено\n    _      => Доставлено\n  }\n  _ => Доставлены\n}\n"
    );
}

#[test]
fn collapses_structural_newlines_in_schema_headers_and_arguments() {
    let source = "type \nUserId \n= \nString\n\ndelivery\n(\n  count\n  :\n  Number\n)\n";
    let formatted =
        format_source(SourceKind::Schema, source, &FormatOptions::default()).expect("format");

    assert_eq!(
        formatted,
        "type UserId = String\n\ndelivery(count: Number)\n"
    );
}

#[test]
fn collapses_structural_newlines_around_annotations() {
    let schema = "type Money = Decimal\n  @\n  currency(\n    code\n    =\n    \"USD\"\n  )\n";
    let formatted_schema =
        format_source(SourceKind::Schema, schema, &FormatOptions::default()).expect("format");

    assert_eq!(
        formatted_schema,
        "type Money = Decimal @currency(code = \"USD\")\n"
    );

    let locale = "price = Цена: {amount\n  @\n  currency(\n    code\n    =\n    \"USD\"\n  )}\n";
    let formatted_locale =
        format_source(SourceKind::Locale, locale, &FormatOptions::default()).expect("format");

    assert_eq!(
        formatted_locale,
        "price = Цена: {amount @currency(code = \"USD\")}\n"
    );
}

#[test]
fn aligns_consecutive_match_arms_by_text_column() {
    let source = "form case(Plural){\none=>One\nother_long=>Many\n_=>Fallback\n}\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert_eq!(
        formatted,
        "form case(Plural) {\n  one        => One\n  other_long => Many\n  _          => Fallback\n}\n"
    );
}

#[test]
fn wraps_long_structural_argument_lists() {
    let source = "delivery(count: Number, fruit: Fruit, size: Size, price: Money)\n";
    let formatted = format_source(
        SourceKind::Schema,
        source,
        &FormatOptions {
            max_line_width: 32,
            ..FormatOptions::default()
        },
    )
    .expect("format");

    assert_eq!(
        formatted,
        "delivery(\n  count: Number,\n  fruit: Fruit,\n  size: Size,\n  price: Money\n)\n"
    );
}

#[test]
fn leaves_long_raw_text_lines_unchanged() {
    let source = "message = This raw text has commas, spaces, and enough words to exceed the configured width\n";
    let formatted = format_source(
        SourceKind::Locale,
        source,
        &FormatOptions {
            max_line_width: 24,
            ..FormatOptions::default()
        },
    )
    .expect("format");

    assert_eq!(formatted, source);
}

#[test]
fn refuses_invalid_source() {
    let error = format_source(
        SourceKind::Schema,
        "delivery(count: Number\n",
        &FormatOptions::default(),
    )
    .expect_err("invalid source");

    assert!(matches!(error, FormatError::Parse(errors) if !errors.is_empty()));
}

#[test]
fn rejects_unknown_extensions() {
    let error = format_path_source(Path::new("notes.txt"), "hello = world\n")
        .expect_err("unknown extension must not default to locale");
    assert!(matches!(error, FormatError::UnsupportedExtension(_)));
}

#[test]
fn preserves_crlf_newlines() {
    let source = "first = One\r\n\r\nsecond = Two\r\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");
    assert_eq!(formatted, source);
}

#[test]
fn preserves_private_use_characters() {
    let source = "message = Keep \u{E000} and \u{E001}\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");
    assert_eq!(formatted, source);
}

#[test]
fn aligns_using_display_columns() {
    let source = "form case(Plural){\n短=>One\nlatin=>Many\n}\n";
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");
    assert_eq!(
        formatted,
        "form case(Plural) {\n  短    => One\n  latin => Many\n}\n"
    );
}

#[test]
fn rejects_pathological_options() {
    for options in [
        FormatOptions {
            indent_width: usize::MAX,
            max_line_width: 100,
        },
        FormatOptions {
            indent_width: 2,
            max_line_width: usize::MAX,
        },
    ] {
        let error = format_source(SourceKind::Locale, "message = text\n", &options)
            .expect_err("pathological option must be rejected");
        assert!(matches!(error, FormatError::InvalidOptions(_)));
    }
}

#[test]
fn preserves_dedented_and_raw_multiline_semantics() {
    let source = concat!(
        "receipt = \"\"\"\r\n",
        "\tOrder {order_id}\r\n",
        "\t  {item_count} items  \r\n",
        "\tThank you.\r\n",
        "\"\"\"\r\n",
        "wire = raw\"\"\"  leading\r\n",
        "\tindentation is data\r\n",
        "trailing spaces stay  \"\"\"\r\n",
    );

    let before = message_semantics(source);
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");
    let after = message_semantics(&formatted);
    let second = format_source(SourceKind::Locale, &formatted, &FormatOptions::default())
        .expect("format twice");

    assert_eq!(before, after);
    assert_eq!(formatted, source);
    assert_eq!(formatted, second);
    assert!(!formatted.contains("\r\r\n"));
}

#[test]
fn raw_and_dedented_blocks_keep_distinct_whitespace_values() {
    let source = concat!(
        "dedented=\"\"\"\n",
        "    first\n",
        "      second  \n",
        "    \"\"\"\n",
        "raw_block=raw\"\"\"\n",
        "    first\n",
        "      second  \n",
        "    \"\"\"\n",
    );
    let before = message_semantics(source);
    assert_ne!(before[0].1, before[1].1);

    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");
    assert_eq!(message_semantics(&formatted), before);
    assert_eq!(
        format_source(SourceKind::Locale, &formatted, &FormatOptions::default())
            .expect("format twice"),
        formatted
    );
}

#[test]
fn multiline_opening_delimiters_use_structural_indentation() {
    let source = concat!(
        "panel{\n",
        "value =\n",
        "\"\"\"\n",
        "  first\n",
        "    second\n",
        "\"\"\"\n",
        "}\n",
    );
    let before = message_semantics(source);
    let formatted =
        format_source(SourceKind::Locale, source, &FormatOptions::default()).expect("format");

    assert!(formatted.contains("  value =\n  \"\"\"\n"));
    assert_eq!(message_semantics(&formatted), before);
    assert_eq!(
        format_source(SourceKind::Locale, &formatted, &FormatOptions::default())
            .expect("format twice"),
        formatted
    );
}

#[test]
fn property_idempotence_and_semantics_hold_for_unicode_text_matrix() {
    let fragments = [
        "ASCII",
        "短い文",
        "e\u{301}cole",
        "🙂👩‍💻",
        "\u{E000}private\u{E001}",
        "  leading and trailing  ",
        "comma, parens (stay), arrow => stays",
    ];

    for left in fragments {
        for right in fragments {
            let source = format!("message = raw\"\"\"{left}\r\n{right}\"\"\"\r\n");
            let before = message_semantics(&source);
            let once = format_source(SourceKind::Locale, &source, &FormatOptions::default())
                .unwrap_or_else(|error| panic!("first format failed for {source:?}: {error}"));
            let twice = format_source(SourceKind::Locale, &once, &FormatOptions::default())
                .unwrap_or_else(|error| panic!("second format failed for {source:?}: {error}"));

            assert_eq!(message_semantics(&once), before, "source: {source:?}");
            assert_eq!(twice, once, "source: {source:?}");
            assert!(!once.contains("\r\r\n"), "source: {source:?}");
        }
    }
}

#[test]
fn property_comment_order_and_contents_survive_formatting() {
    let schema = concat!(
        "/// Delivery (count, locale)\n",
        "delivery(count:Number,locale:Locale)\n",
        "// between declarations: (a, b)\n",
        "labels{\n",
        "/// Nested label\n",
        "title\n",
        "// before close => keep\n",
        "}\n",
    );
    let locale = concat!(
        "/// Fruit forms\n",
        "form fruit(Plural){\n",
        "// singular (one, item)\n",
        "one=>Apple\n",
        "// fallback => many\n",
        "_=>Apples\n",
        "}\n",
        "// between\n",
        "message=Hello\n",
    );

    for (kind, source) in [(SourceKind::Schema, schema), (SourceKind::Locale, locale)] {
        let before = comments(kind, source);
        let formatted =
            format_source(kind, source, &FormatOptions::default()).expect("format comments");
        let second =
            format_source(kind, &formatted, &FormatOptions::default()).expect("format twice");
        assert_eq!(comments(kind, &formatted), before);
        assert_eq!(second, formatted);
    }
}

#[test]
fn property_malformed_inputs_return_errors_without_partial_output() {
    let cases = [
        (
            SourceKind::Schema,
            "enum Broken {\n  one,\n  // comment remains owned by caller\n",
        ),
        (
            SourceKind::Schema,
            "/// detached\n\n// ordinary\nmessage(value: String)\n",
        ),
        (
            SourceKind::Locale,
            "message = raw\"\"\"unterminated\r\n// this is text\r\n",
        ),
        (
            SourceKind::Locale,
            "message = before {name @currency(code = \"USD\")\n// still malformed\n",
        ),
    ];

    for (kind, source) in cases {
        let error = format_source(kind, source, &FormatOptions::default())
            .expect_err("malformed input must not produce formatted output");
        let FormatError::Parse(errors) = error else {
            panic!("expected parse error for {source:?}, got {error:?}");
        };
        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .all(|error| error.span.start <= error.span.end && error.span.end <= source.len()));
    }
}

#[test]
fn invalid_token_spans_are_reported_instead_of_dropping_text() {
    let options = FormatOptions::default();
    let semantics = FormatSemantics::schema();
    for span in [
        Span::new(1, 2),
        Span::new(0, usize::MAX),
        Span::in_source(SourceId(7), 0, 2),
    ] {
        let token = Token::new(TokenKind::Ident("invalid".to_owned()), span);
        let error = engine::render_tokens("é", &[token], &semantics, &options, "\n")
            .expect_err("invalid span");
        assert_eq!(error, FormatError::InvalidTokenSpan(span));

        let trivia = Token::new(TokenKind::Whitespace, span);
        let error = semantics
            .validate_tokens("é", &[trivia])
            .expect_err("invalid trivia span");
        assert_eq!(error, FormatError::InvalidTokenSpan(span));
    }
}

#[test]
fn max_width_zero_disables_wrapping_and_long_tokens_remain_indivisible() {
    let source = "delivery(first: ExtremelyLongUnbreakableTypeName, second: Number)\n";
    let unlimited = format_source(
        SourceKind::Schema,
        source,
        &FormatOptions {
            max_line_width: 0,
            ..FormatOptions::default()
        },
    )
    .expect("unlimited formatting");
    let narrow = format_source(
        SourceKind::Schema,
        source,
        &FormatOptions {
            max_line_width: 12,
            ..FormatOptions::default()
        },
    )
    .expect("narrow formatting");

    assert_eq!(unlimited, source);
    assert!(narrow.contains("ExtremelyLongUnbreakableTypeName"));
    assert_eq!(
        format_source(
            SourceKind::Schema,
            &narrow,
            &FormatOptions {
                max_line_width: 12,
                ..FormatOptions::default()
            }
        )
        .expect("format twice"),
        narrow
    );
}

#[test]
fn structural_wrapping_does_not_split_commas_inside_strings() {
    let source =
        "type Money = Decimal @currency(code = \"USD,EUR\", style = \"accounting,long\")\n";
    let options = FormatOptions {
        max_line_width: 36,
        ..FormatOptions::default()
    };
    let formatted =
        format_source(SourceKind::Schema, source, &options).expect("format annotation arguments");

    assert!(formatted.contains("code = \"USD,EUR\","));
    assert!(formatted.contains("style = \"accounting,long\""));
    assert_eq!(
        format_source(SourceKind::Schema, &formatted, &options).expect("format twice"),
        formatted
    );
}

#[test]
fn source_kind_reports_missing_extensions_explicitly() {
    let error =
        format_path_source(Path::new("README"), "").expect_err("extensionless path must fail");
    assert_eq!(
        error,
        FormatError::UnsupportedExtension("<none>".to_owned())
    );
}

fn message_semantics(source: &str) -> Vec<(String, String, Vec<String>)> {
    let locale = parse_locale(source).expect("valid locale");
    let mut output = Vec::new();
    for declaration in &locale.declarations {
        match declaration {
            LocaleDeclaration::Message(message) => push_message_semantics(message, &mut output),
            LocaleDeclaration::Group(group) => group_semantics(group, &mut output),
            _ => {}
        }
    }
    output
}

fn group_semantics(
    group: &linguini_syntax::MessageImplementationGroup,
    output: &mut Vec<(String, String, Vec<String>)>,
) {
    for message in &group.messages {
        push_message_semantics(message, output);
    }
    for nested in &group.groups {
        group_semantics(nested, output);
    }
}

fn push_message_semantics(
    message: &linguini_syntax::MessageImplementation,
    output: &mut Vec<(String, String, Vec<String>)>,
) {
    output.push((
        message.name.value.clone(),
        format!("{:?}", message.value.mode),
        pattern_semantics(&message.value),
    ));
}

fn pattern_semantics(pattern: &TextPattern) -> Vec<String> {
    pattern
        .parts
        .iter()
        .map(|part| match part {
            TextPart::Text(text) => format!("text:{:?}", text.value),
            TextPart::Placeholder(placeholder) => {
                format!(
                    "placeholder:{}",
                    expression_semantics(&placeholder.expression)
                )
            }
        })
        .collect()
}

fn expression_semantics(expression: &Expression) -> String {
    let path = expression
        .path
        .iter()
        .map(|part| part.value.as_str())
        .collect::<Vec<_>>()
        .join(".");
    let arguments = expression
        .arguments
        .iter()
        .map(expression_semantics)
        .collect::<Vec<_>>()
        .join(",");
    let annotations = expression
        .annotations
        .iter()
        .map(|annotation| {
            let arguments = annotation
                .arguments
                .iter()
                .map(|argument| format!("{}={:?}", argument.name.value, argument.value.value))
                .collect::<Vec<_>>()
                .join(",");
            format!("{:?}({arguments})", annotation.kind)
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{:?}:{path}({arguments})[{annotations}]", expression.kind)
}

fn comments(kind: SourceKind, source: &str) -> Vec<(bool, String)> {
    let tokens = match kind {
        SourceKind::Schema => lex_schema_with_recovery(source).tokens,
        SourceKind::Locale => lex_with_recovery(source).tokens,
    };
    tokens
        .into_iter()
        .filter_map(|token| match token.kind {
            TokenKind::Comment(text) => Some((false, text)),
            TokenKind::DocComment(text) => Some((true, text)),
            _ => None,
        })
        .collect()
}
