//! Build-time decimal digit symbols shared by target renderers.

/// Resolve the ten Unicode scalar symbols of a positional decimal numbering system.
/// Targets decide how to represent/index these symbols without parsing CLDR data at runtime.
pub fn decimal_digit_symbols(digits: &str) -> Option<[char; 10]> {
    let mut input = digits.chars();
    let mut symbols = ['\0'; 10];
    for symbol in &mut symbols {
        *symbol = input.next()?;
    }
    input.next().is_none().then_some(symbols)
}

/// Whether one substitution pass can preserve every literal and textual date symbol.
/// ASCII digits in quoted text, month/quarter names, eras, or period labels must stay literal.
pub fn can_localize_date_result<'a>(
    pattern: &[crate::DatePatternPart],
    textual_symbols: impl IntoIterator<Item = &'a str>,
) -> bool {
    let contains_digits = |value: &str| value.bytes().any(|byte| byte.is_ascii_digit());
    !pattern.iter().any(
        |part| matches!(part, crate::DatePatternPart::Literal(value) if contains_digits(value)),
    ) && !textual_symbols.into_iter().any(contains_digits)
}

#[cfg(test)]
mod tests {
    use super::{can_localize_date_result, decimal_digit_symbols};

    #[test]
    fn decimal_symbols_count_unicode_scalars_instead_of_bytes_or_utf16_units() {
        for digits in ["0123456789", "٠١٢٣٤٥٦٧٨٩", "𝟢𝟣𝟤𝟥𝟦𝟧𝟨𝟩𝟪𝟫"]
        {
            let symbols = decimal_digit_symbols(digits).unwrap();
            assert_eq!(symbols.into_iter().collect::<String>(), digits);
        }
        for digits in ["", "012345678", "01234567890", "𝟢𝟣𝟤𝟥𝟦𝟧𝟨𝟩𝟪"] {
            assert!(decimal_digit_symbols(digits).is_none());
        }
        for locale in linguini_cldr::locale_resolution_candidates() {
            if let Some(dates) = linguini_cldr::compiled_date_formatting(locale) {
                assert!(decimal_digit_symbols(dates.digits).is_some(), "{locale}");
            }
        }
    }

    #[test]
    fn whole_date_substitution_preserves_digits_in_literals_and_textual_symbols() {
        let field = crate::DatePatternPart::Field {
            symbol: 'y',
            width: 4,
        };
        assert!(can_localize_date_result(
            std::slice::from_ref(&field),
            ["January", "AD", "٢"]
        ));
        assert!(!can_localize_date_result(
            std::slice::from_ref(&field),
            ["January", "Q1"]
        ));
        assert!(!can_localize_date_result(
            &[field, crate::DatePatternPart::Literal(" 123 ".into())],
            []
        ));
    }
}
