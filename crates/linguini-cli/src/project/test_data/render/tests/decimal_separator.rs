use super::*;

#[test]
fn numeric_preview_decimal_visibility_matches_the_generated_contract_corpus() {
    let corpus: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/codegen/decimal-separator.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let numbers = compiled_number_formatting("en").unwrap();
    for case in corpus.as_array().unwrap() {
        for visible in [false, true] {
            let mut pattern = numbers.decimal_pattern;
            pattern.negative = None;
            let part = &mut pattern.positive;
            part.min_fraction_digits = case["min_fraction"].as_u64().unwrap() as u8;
            part.max_fraction_digits = case["max_fraction"].as_u64().unwrap() as u8;
            part.min_significant_digits = case["min_significant"].as_u64().map(|n| n as u8);
            part.max_significant_digits = case["max_significant"].as_u64().map(|n| n as u8);
            part.decimal_separator_always = visible;
            if case["scientific"].as_bool().unwrap() {
                part.exponent_digits = Some(2);
            }
            let unsupported =
                part.max_significant_digits.is_some() || part.exponent_digits.is_some();
            for sample in case["samples"].as_array().unwrap() {
                let result = format_number_pattern(
                    sample["input"].as_str().unwrap(),
                    &pattern,
                    &numbers,
                    None,
                    None,
                );
                if unsupported {
                    assert!(matches!(result, Err(RenderError::Unsupported { .. })));
                } else {
                    assert_eq!(
                        result.unwrap(),
                        sample[if visible { "visible" } else { "ordinary" }]
                            .as_str()
                            .unwrap(),
                        "{}: {sample}",
                        case["name"]
                    );
                }
            }
        }
    }
}

#[test]
fn numeric_preview_visible_decimal_preserves_unicode_and_currency_overrides() {
    for (locale, input, expected) in [
        ("ar-EG", "-0", "\u{061c}-v12:٠٫:v34"),
        ("fa", "1234", "v12:۱٬۲۳۴٫:v34"),
        ("ff-Adlm", "1", "v12:𞥑.:v34"),
    ] {
        let numbers = compiled_number_formatting(locale).unwrap();
        let mut pattern = numbers.decimal_pattern;
        pattern.negative = None;
        pattern.positive.prefix = NumberAffix::Literal("v12:");
        pattern.positive.suffix = NumberAffix::Literal(":v34");
        pattern.positive.min_fraction_digits = 0;
        pattern.positive.max_fraction_digits = 0;
        pattern.positive.decimal_separator_always = true;
        assert_eq!(
            format_number_pattern(input, &pattern, &numbers, None, None).unwrap(),
            expected
        );
    }
    let numbers = compiled_number_formatting("en").unwrap();
    let mut pattern = numbers.decimal_pattern;
    pattern.negative = None;
    pattern.positive.prefix = NumberAffix::CurrencySymbol;
    pattern.positive.suffix = NumberAffix::Literal(" k");
    pattern.positive.decimal_separator_always = true;
    assert_eq!(
        format_number_pattern("1.2", &pattern, &numbers, Some("$"), Some((0, 0))).unwrap(),
        "$1. k"
    );
    pattern.positive.decimal_separator_always = false;
    assert_eq!(
        format_number_pattern("1.2", &pattern, &numbers, Some("$"), Some((0, 0))).unwrap(),
        "$1 k"
    );
}
