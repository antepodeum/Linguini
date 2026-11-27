use super::*;

#[test]
fn compiled_padding_preserves_positions_and_literal_codepoints() {
    let mut digit_bindings = DigitBindings::default();
    let mut source = String::from("import assert from 'node:assert/strict';\n");
    let mut plans = Vec::new();
    let mut checks = String::new();
    for (locale_index, locale) in ["en", "ff-Adlm"].into_iter().enumerate() {
        let numbers = compiled_number_formatting(locale).unwrap();
        let digits = linguini_codegen::decimal_digit_symbols(numbers.digits).unwrap();
        let localize = |text: &str| {
            text.chars()
                .map(|character| {
                    character
                        .to_digit(10)
                        .map_or(character, |digit| digits[digit as usize])
                })
                .collect::<String>()
        };
        for (position_index, position) in [
            NumberPaddingPosition::BeforePrefix,
            NumberPaddingPosition::AfterPrefix,
            NumberPaddingPosition::BeforeSuffix,
            NumberPaddingPosition::AfterSuffix,
        ]
        .into_iter()
        .enumerate()
        {
            for (character_index, character) in ['\'', ';', '💠', '*'].into_iter().enumerate() {
                let mut pattern = numbers.decimal_pattern;
                pattern.negative = None;
                pattern.positive.prefix = NumberAffix::Literal("pre");
                pattern.positive.suffix = NumberAffix::Literal("tail");
                pattern.positive.min_integer_digits = 1;
                pattern.positive.min_fraction_digits = 1;
                pattern.positive.max_fraction_digits = 1;
                pattern.positive.primary_group_size = None;
                pattern.positive.secondary_group_size = None;
                pattern.positive.padding = Some(NumberPadding {
                    character,
                    width: 14,
                    position,
                });
                let name = format!("padding{locale_index}_{position_index}_{character_index}");
                let plan = number_plan(&pattern, None, &numbers, None, &mut digit_bindings);
                source.push_str(&crate::templates::render(
                    "formatters/number",
                    minijinja::context! { name => &name, plan => plan.clone(), exported => false },
                ));
                plans.push(plan);
                for (input, prefix, number, count) in [
                    ("1", "pre", "1.0", 4),
                    ("-1", "-pre", "1.0", 3),
                    ("123456", "pre", "123456.0", 0),
                ] {
                    let padding = character.to_string().repeat(count);
                    let number = localize(number);
                    let expected = match position {
                        NumberPaddingPosition::BeforePrefix => {
                            format!("{padding}{prefix}{number}tail")
                        }
                        NumberPaddingPosition::AfterPrefix => {
                            format!("{prefix}{padding}{number}tail")
                        }
                        NumberPaddingPosition::BeforeSuffix => {
                            format!("{prefix}{number}{padding}tail")
                        }
                        NumberPaddingPosition::AfterSuffix => {
                            format!("{prefix}{number}tail{padding}")
                        }
                    };
                    checks.push_str(&format!("assert.equal({name}({input:?}), {expected:?});\n"));
                }
            }
        }
    }
    assert!(!source.contains("formatGeneratedNumber"));
    assert!(!source.contains("padding.position"));
    source.push_str(&number_helpers(&plans));
    source.push_str(&checks);
    source.push_str("console.log('ok');\n");
    assert_eq!(execute(&source), "ok");
}
