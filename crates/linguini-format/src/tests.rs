use super::{format_source, FormatOptions, SourceKind, CRATE_PURPOSE};

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

    assert_eq!(formatted, "type UserId = String\n\ndelivery(count: Number)\n");
}


#[test]
fn collapses_structural_newlines_around_annotations() {
    let schema = "type Money = Decimal\n  @\n  currency(\n    code\n    =\n    \"USD\"\n  )\n";
    let formatted_schema =
        format_source(SourceKind::Schema, schema, &FormatOptions::default()).expect("format");

    assert_eq!(formatted_schema, "type Money = Decimal @currency(code = \"USD\")\n");

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
fn refuses_invalid_source() {
    let error = format_source(
        SourceKind::Schema,
        "delivery(count: Number\n",
        &FormatOptions::default(),
    )
    .expect_err("invalid source");

    assert!(!error.errors.is_empty());
}
