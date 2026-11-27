use super::*;

mod cases {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codegen/currency-affixes.rs"
    ));
}

#[test]
fn numeric_previews_replace_only_compiled_currency_slots() {
    let numbers = compiled_number_formatting("en").unwrap();
    for case in cases::CASES {
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.prefix = case.prefix;
        pattern.positive.suffix = case.suffix;
        pattern.positive.min_fraction_digits = 0;
        pattern.positive.max_fraction_digits = 0;
        pattern.positive.primary_group_size = None;
        pattern.positive.secondary_group_size = None;
        pattern.negative = case
            .negative
            .map(|(prefix, suffix)| linguini_cldr::NumberPatternPart {
                prefix,
                suffix,
                ..pattern.positive
            });
        for (input, expected) in [("1", case.positive), ("-1", case.negative_text)] {
            assert_eq!(
                format_number_pattern(input, &pattern, &numbers, Some("$"), None).unwrap(),
                expected,
                "{}: {}",
                case.name,
                case.pattern
            );
        }
    }
}
