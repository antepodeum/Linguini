use super::*;

#[test]
fn scientific_symbols_and_numeric_digits_are_compiled_independently() {
    let mut script = String::from(
        "import assert from 'node:assert/strict';\nIntl.NumberFormat=class {constructor(){throw Error('no host formatter');}};\n",
    );
    let mut plans = Vec::new();
    let mut checks = String::new();
    for (index, (locale, digits, separator, exponential, minus, plus)) in [
        ("en", "0123456789", ".", "E", "-", "+"),
        ("sv", "0123456789", ",", "×10^", "−", "+"),
        ("fa", "۰۱۲۳۴۵۶۷۸۹", "٫", "×۱۰^", "\u{200e}−", "\u{200e}+"),
        ("ar-EG", "٠١٢٣٤٥٦٧٨٩", "٫", "أس", "\u{61c}-", "\u{61c}+"),
        ("ff-Adlm", "𞥐𞥑𞥒𞥓𞥔𞥕𞥖𞥗𞥘𞥙", ".", "𞤉", "-", "+"),
        ("ccp", "𑄶𑄷𑄸𑄹𑄺𑄻𑄼𑄽𑄾𑄿", ".", "E", "-", "+"),
        // Literal digits in exponent symbols/signs must never be substituted.
        ("synthetic", "abcdefghij", ".", "x10^", "-34", "+12"),
        ("syntheticMixed", "0𝟙2𝟛4𝟝6𝟟8𝟡", ".", "x10^", "-34", "+12"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut numbers = compiled_number_formatting(if locale.starts_with("synthetic") {
            "en"
        } else {
            locale
        })
        .unwrap();
        if locale.starts_with("synthetic") {
            numbers.digits = digits;
            numbers.exponential_symbol = exponential;
            numbers.minus_sign = minus;
            numbers.plus_sign = plus;
        }
        assert_eq!(numbers.digits, digits, "{locale}");
        assert_eq!(numbers.decimal_symbol, separator, "{locale}");
        assert_eq!(numbers.exponential_symbol, exponential, "{locale}");
        assert_eq!(numbers.minus_sign, minus, "{locale}");
        assert_eq!(numbers.plus_sign, plus, "{locale}");
        let alphabet: Vec<_> = digits.chars().collect();
        let localize = |ascii: &str| {
            ascii
                .chars()
                .map(|digit| alphabet[digit.to_digit(10).unwrap() as usize])
                .collect::<String>()
        };
        for always_sign in [false, true] {
            let mut pattern = numbers.decimal_pattern;
            pattern.positive.primary_group_size = None;
            pattern.positive.secondary_group_size = None;
            pattern.positive.min_fraction_digits = 2;
            pattern.positive.max_fraction_digits = 2;
            pattern.positive.exponent_digits = Some(2);
            pattern.positive.exponent_sign_always = always_sign;
            let plan = number_plan(&pattern, None, &numbers, None);
            let name = format!("scientific{index}{}", u8::from(always_sign));
            script.push_str(&crate::templates::render(
                "formatters/number",
                minijinja::context! { name, plan => plan.clone(), exported => false },
            ));
            plans.push(plan);
            for (input, negative, integer, fraction, exponent, exponent_negative) in [
                ("1234", false, "1", "23", "03", false),
                ("0.001234", false, "1", "23", "03", true),
                ("0", false, "0", "00", "00", false),
                ("-0", true, "0", "00", "00", false),
                ("-1234", true, "1", "23", "03", false),
                ("9.999", false, "1", "00", "01", false),
            ] {
                let expected = format!(
                    "{}{}{separator}{}{exponential}{}{}",
                    if negative { minus } else { "" },
                    localize(integer),
                    localize(fraction),
                    if exponent_negative {
                        minus
                    } else if always_sign {
                        plus
                    } else {
                        ""
                    },
                    localize(exponent),
                );
                checks.push_str(&format!(
                    "assert.equal({name}({input:?}), {expected:?}, {locale:?} + ':' + {input:?});\n"
                ));
            }
        }
    }
    script.push_str(&number_helpers(&plans));
    script.push_str(&checks);
    script.push_str("console.log('ok');\n");
    assert_eq!(execute(&script), "ok");
    for locale in ["en", "sv", "fa", "ar-EG", "ff-Adlm", "ccp"] {
        let ordinary = formatter_data_declaration(
            locale,
            &[
                request(IrFormatterKind::Number, &[]),
                request(IrFormatterKind::Currency, &[]),
            ],
            false,
        );
        assert!(!ordinary.contains("scientificGeneratedDecimal"), "{locale}");
        assert!(!ordinary.contains("const exponentDigits"), "{locale}");
    }
}

#[test]
fn scientific_runtime_snapshot_is_strict_checked() {
    let mut output = crate::templates::render(
        "formatters/types",
        minijinja::context! { needs_number => true },
    );
    let mut plans = Vec::new();
    for (locale, name, digits) in [
        ("en", "scientificLatin", "0123456789"),
        ("fa", "scientificPersian", "۰۱۲۳۴۵۶۷۸۹"),
        ("ff-Adlm", "scientificAdlam", "𞥐𞥑𞥒𞥓𞥔𞥕𞥖𞥗𞥘𞥙"),
        ("en", "scientificMixed", "0𝟙2𝟛4𝟝6𝟟8𝟡"),
    ] {
        let mut numbers = compiled_number_formatting(locale).unwrap();
        numbers.digits = digits;
        let mut pattern = numbers.decimal_pattern;
        pattern.positive.primary_group_size = None;
        pattern.positive.secondary_group_size = None;
        pattern.positive.exponent_digits = Some(2);
        pattern.positive.exponent_sign_always = true;
        let plan = number_plan(&pattern, None, &numbers, None);
        output.push_str(&crate::templates::render(
            "formatters/number",
            minijinja::context! { name, plan => plan.clone(), exported => true },
        ));
        plans.push(plan);
    }
    output.push_str(&number_helpers(&plans));
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-runtime/scientific.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &output).unwrap();
    }
    assert_eq!(output, std::fs::read_to_string(snapshot).unwrap());
}
