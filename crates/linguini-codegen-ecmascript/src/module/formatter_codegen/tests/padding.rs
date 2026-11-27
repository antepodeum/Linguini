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

#[test]
fn padding_width_uses_compiled_scalar_widths_without_iterating_text() {
    let mut bindings = DigitBindings::default();
    let types = crate::templates::render(
        "formatters/types",
        minijinja::context! { needs_number => true },
    );
    let mut source = types.clone();
    let mut snapshot = types;
    let mut plans = Vec::new();
    let mut snapshot_plans = Vec::new();
    let mut checks = String::new();
    for (
        index,
        (locale, decimal, group, prefix, suffix, negative_prefix, negative_suffix, style),
    ) in [
        ("en", ".", ",", "pre", "tail", "-pre", "tail", "fixed"),
        (
            "ar-EG",
            "٫",
            "٬",
            "💠0pre",
            "tail💠1",
            "➖pre",
            "💠tail",
            "fixed",
        ),
        ("ff-Adlm", ".", ",", "pre", "tail", "-pre", "tail", "fixed"),
        ("ccp", "💠.", "🟦,", "pre", "tail", "-pre", "tail", "fixed"),
        (
            "en", "💠.", "🟦,", "💠pre", "tail", "➖pre", "💠tail", "fixed",
        ),
        ("en", ".", ",", "p", "s", "➖", "💠", "fixed"),
        (
            "en",
            "💠.",
            "🟦,",
            "pre",
            "tail",
            "-pre",
            "tail",
            "scientific",
        ),
        (
            "ff-Adlm",
            ".",
            ",",
            "pre",
            "tail",
            "-pre",
            "tail",
            "scientific_unsigned",
        ),
        (
            "en",
            ".",
            ",",
            "mixed0",
            "mixed1",
            "➖mixed0",
            "mixed1",
            "mixed",
        ),
        ("en", "💠.💠", "🟦,🟦", "p", "s", "-p", "s", "optional"),
        ("en", "💠.💠", "🟦,🟦", "p", "s", "-p", "s", "integer"),
        ("en", "💠.💠", "🟦,🟦", "p", "s", "-p", "s", "decimal"),
        ("en", "💠.💠", "🟦,🟦", "p", "s", "-p", "s", "significant"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut numbers = compiled_number_formatting(locale).unwrap();
        numbers.decimal_symbol = decimal;
        numbers.group_symbol = group;
        let scientific = matches!(style, "scientific" | "scientific_unsigned");
        // Unselected scientific symbols must not retain width corrections.
        numbers.exponential_symbol = "E💠";
        if scientific {
            numbers.plus_sign = "➕💠";
            numbers.minus_sign = "➖💠";
        }
        if style == "mixed" {
            numbers.digits = "0𝟙2𝟛4𝟝6𝟟8𝟡";
        }
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.prefix = NumberAffix::Literal(prefix);
        pattern.positive.suffix = NumberAffix::Literal(suffix);
        pattern.positive.min_fraction_digits = 1;
        pattern.positive.max_fraction_digits = 1;
        if matches!(style, "optional" | "integer" | "decimal" | "significant") {
            // Unequal groups and delayed grouping exercise the selected separator count.
            numbers.minimum_grouping_digits = 2;
            pattern.positive.primary_group_size = Some(3);
            pattern.positive.secondary_group_size = Some(2);
            pattern.positive.min_fraction_digits = 0;
            if matches!(style, "integer" | "decimal") {
                pattern.positive.max_fraction_digits = 0;
                pattern.positive.decimal_separator_always = style == "decimal";
            }
            if style == "significant" {
                pattern.positive.min_significant_digits = Some(2);
                pattern.positive.max_significant_digits = Some(3);
            }
        }
        if scientific {
            pattern.positive.exponent_digits = Some(2);
            pattern.positive.exponent_sign_always = style == "scientific";
            pattern.positive.primary_group_size = None;
            pattern.positive.secondary_group_size = None;
        }
        let mut negative = pattern.positive;
        negative.prefix = NumberAffix::Literal(negative_prefix);
        negative.suffix = NumberAffix::Literal(negative_suffix);
        pattern.negative = Some(negative);
        let base_name = format!("paddingBase{index}");
        let base_plan = number_plan(&pattern, None, &numbers, None, &mut bindings);
        source.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! { name => &base_name, plan => base_plan.clone(), exported => false },
        ));
        plans.push(base_plan);
        for width in [0, 6, 14, 24] {
            for (position_index, position) in [
                NumberPaddingPosition::BeforePrefix,
                NumberPaddingPosition::AfterPrefix,
                NumberPaddingPosition::BeforeSuffix,
                NumberPaddingPosition::AfterSuffix,
            ]
            .into_iter()
            .enumerate()
            {
                pattern.positive.padding = Some(NumberPadding {
                    character: '_',
                    width,
                    position,
                });
                let name = format!("padding{index}_{width}_{position_index}");
                let plan = number_plan(&pattern, None, &numbers, None, &mut bindings);
                let emitted = crate::templates::render(
                    "formatters/number",
                    minijinja::context! { name => &name, plan => plan.clone(), exported => true },
                );
                if width == 0 {
                    assert!(!emitted.contains("const padding"));
                    assert!(!emitted.contains("paddingWidth"));
                }
                source.push_str(&emitted);
                if width == 14 && position == NumberPaddingPosition::AfterPrefix {
                    snapshot.push_str(&emitted);
                    snapshot_plans.push(plan.clone());
                }
                plans.push(plan);
                checks.push_str(&format!(
                    "check({name}, {base_name}, {width}, {position_index}, {});\n",
                    serde_json::to_string(&[prefix, suffix, negative_prefix, negative_suffix])
                        .unwrap(),
                ));
            }
        }
    }
    snapshot.push_str(&number_helpers(&snapshot_plans));
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-runtime/padding.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&path, &snapshot).unwrap();
    }
    assert_eq!(snapshot, std::fs::read_to_string(path).unwrap());
    source.push_str(&number_helpers(&plans));
    source.push_str(
        r#"
import assert from 'node:assert/strict';
function check(format, base, width, position, affixes) {
  for (const input of ['0', '-0', '1', '-1', '1234.5', '12345.5', '999.96', '9999.96', '123456.5', '-123456.5', '0.04', '9'.repeat(1000)]) {
    const value = base(input);
    const negative = input.startsWith('-');
    const [prefix, suffix] = affixes.slice(negative ? 2 : 0, negative ? 4 : 2);
    const padding = '_'.repeat(Math.max(0, width - Array.from(value).length));
    const body = value.slice(prefix.length, value.length - suffix.length);
    const expected = [
      padding + prefix + body + suffix,
      prefix + padding + body + suffix,
      prefix + body + padding + suffix,
      prefix + body + suffix + padding,
    ][position];
    const iterator = String.prototype[Symbol.iterator];
    const charCodeAt = String.prototype.charCodeAt;
    let scanned = 0;
    String.prototype[Symbol.iterator] = function() { throw Error('padding iterated a string'); };
    String.prototype.charCodeAt = function(index) { scanned++; return charCodeAt.call(this, index); };
    let actual;
    try {
      base(input);
      const numericReads = scanned;
      scanned = 0;
      actual = format(input);
      scanned -= numericReads;
    }
    finally {
      String.prototype[Symbol.iterator] = iterator;
      String.prototype.charCodeAt = charCodeAt;
    }
    assert.equal(actual, expected);
    assert.equal(scanned, 0, 'padding must not scan numeric output');
  }
}
"#,
    );
    source.push_str(&checks);
    source.push_str("console.log('ok');\n");
    assert_eq!(execute(&source), "ok");
}
