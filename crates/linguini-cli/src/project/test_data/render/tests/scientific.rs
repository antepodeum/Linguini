use super::*;

#[test]
fn native_scientific_and_engineering_previews_match_the_exact_icu_corpus() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../../tests/fixtures/codegen/scientific-widths.json"
    ))
    .unwrap();
    let numbers = compiled_number_formatting("en").unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let mut pattern = numbers.decimal_pattern;
        pattern.negative = None;
        let part = &mut pattern.positive;
        part.primary_group_size = None;
        part.secondary_group_size = None;
        part.min_integer_digits = case["min_integer"].as_u64().unwrap() as u8;
        part.max_integer_digits = case["max_integer"].as_u64().map(|value| value as u8);
        part.min_fraction_digits = case["min_fraction"].as_u64().unwrap() as u8;
        part.max_fraction_digits = case["max_fraction"].as_u64().unwrap() as u8;
        part.min_significant_digits = case["min_significant"].as_u64().map(|value| value as u8);
        part.max_significant_digits = case["max_significant"].as_u64().map(|value| value as u8);
        part.rounding_increment = case["rounding_increment"]
            .as_str()
            .map(|value| &*Box::leak(value.to_owned().into_boxed_str()));
        part.exponent_digits = Some(case["exponent_digits"].as_u64().unwrap() as u8);
        part.exponent_sign_always = case["exponent_sign_always"].as_bool().unwrap();
        part.decimal_separator_always = case["decimal_separator_always"].as_bool().unwrap();
        for sample in case["samples"].as_array().unwrap() {
            let input = sample["input"].as_str().unwrap();
            assert_eq!(
                format_number_pattern(input, &pattern, &numbers, None, None).unwrap(),
                sample["expected"].as_str().unwrap(),
                "{}: {input}",
                case["pattern"]
            );
        }
    }
    let mut pattern = numbers.decimal_pattern;
    pattern.positive.max_significant_digits = Some(3);
    assert!(matches!(
        format_number_pattern("1e-8191", &pattern, &numbers, None, None),
        Err(RenderError::InvalidNumber { value }) if value == "1e-8191"
    ));
}
