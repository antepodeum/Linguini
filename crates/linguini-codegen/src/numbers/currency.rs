//! Currency-specific CLDR selection shared by every backend and native preview.

use linguini_cldr::{
    compiled_currency_formatting, compiled_currency_fraction, compiled_currency_symbol,
    compiled_number_formatting, CurrencyFormatData, NumberAffix, NumberAffixPart, NumberPattern,
};
use unicode_general_category::{get_general_category, GeneralCategory};

use super::NumberFormatPlan;

impl NumberFormatPlan {
    /// Resolve currency style, symbol-boundary pattern, precision, rounding, and separators.
    /// Unsupported locales, malformed currency codes, or invalid numeric plans return `None`.
    pub fn currency(locale: &str, code: &str, accounting: bool) -> Option<Self> {
        let code = code.to_ascii_uppercase();
        let numbers = compiled_number_formatting(locale)?;
        let currency = compiled_currency_formatting(locale)?;
        let fraction = compiled_currency_fraction(&code)?;
        let symbol = compiled_currency_symbol(locale, &code).unwrap_or(&code);
        let mut pattern = select_pattern(&currency, symbol, accounting);
        pattern.positive.min_fraction_digits = fraction.digits;
        pattern.positive.max_fraction_digits = fraction.digits;
        let numbers = linguini_cldr::NumberFormatData {
            decimal_symbol: currency.decimal_symbol,
            group_symbol: currency.group_symbol,
            ..numbers
        };
        Self::new(
            &pattern,
            Some(symbol),
            &numbers,
            Some(&fraction.rounding.to_string()),
        )
    }
}

fn select_pattern(currency: &CurrencyFormatData, symbol: &str, accounting: bool) -> NumberPattern {
    let (base, alternative) = match (accounting, currency.accounting_pattern) {
        (true, Some(pattern)) => (pattern, currency.accounting_alpha_next_to_number),
        _ => (
            currency.standard_pattern,
            currency.standard_alpha_next_to_number,
        ),
    };
    alternative
        .copied()
        .filter(|_| {
            (has_currency_slot(base.positive.prefix)
                && symbol.chars().next_back().is_some_and(is_letter))
                || (has_currency_slot(base.positive.suffix)
                    && symbol.chars().next().is_some_and(is_letter))
        })
        .unwrap_or(base)
}

fn has_currency_slot(affix: NumberAffix) -> bool {
    match affix {
        NumberAffix::Literal(_) => false,
        NumberAffix::CurrencySymbol => true,
        NumberAffix::Parts(parts) => parts
            .iter()
            .any(|part| matches!(part, NumberAffixPart::CurrencySymbol)),
    }
}

fn is_letter(character: char) -> bool {
    matches!(
        get_general_category(character),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_selection_uses_category_l_at_the_correct_symbol_end() {
        assert_eq!(unicode_general_category::UNICODE_VERSION, (16, 0, 0));
        let currency = compiled_currency_formatting("en").unwrap();
        let alternative = *currency.standard_alpha_next_to_number.unwrap();
        for (symbol, letter) in [
            ("CHF", true),
            ("$US", true),
            ("US$", false),
            ("CN¥", false),
            ("ǅ", true),
            ("ʰ", true),
            ("𐐀", true),
            ("日", true),
            ("Ⅰ", false),
            ("\u{345}", false),
            ("\u{903}", false),
            ("A\u{301}", false),
            ("A\u{200f}", false),
            ("", false),
        ] {
            assert_eq!(
                select_pattern(&currency, symbol, false),
                if letter {
                    alternative
                } else {
                    currency.standard_pattern
                },
                "prefix {symbol:?}"
            );
        }
        let currency = compiled_currency_formatting("km").unwrap();
        for (symbol, letter) in [
            ("US$", true),
            ("$US", false),
            ("\u{301}A", false),
            ("A\u{301}", true),
        ] {
            assert_eq!(
                select_pattern(&currency, symbol, false),
                if letter {
                    *currency.standard_alpha_next_to_number.unwrap()
                } else {
                    currency.standard_pattern
                },
                "suffix {symbol:?}"
            );
        }
    }

    #[test]
    fn currency_style_fallback_and_literal_marks_do_not_select_unrelated_alternatives() {
        let mut currency = compiled_currency_formatting("en").unwrap();
        let base = currency.standard_pattern;
        currency.standard_alpha_next_to_number = None;
        assert_eq!(select_pattern(&currency, "CHF", false), base);
        currency.accounting_pattern = None;
        assert_eq!(select_pattern(&currency, "CHF", true), base);
        let mut currency = compiled_currency_formatting("en").unwrap();
        currency.standard_pattern.positive.prefix = NumberAffix::Literal("¤");
        assert_eq!(
            select_pattern(&currency, "CHF", false),
            currency.standard_pattern
        );
        currency.standard_pattern.positive.prefix = NumberAffix::Parts(&[
            NumberAffixPart::Literal("v1:"),
            NumberAffixPart::CurrencySymbol,
        ]);
        assert_eq!(
            select_pattern(&currency, "CHF", false),
            *currency.standard_alpha_next_to_number.unwrap()
        );
        assert_eq!(
            NumberFormatPlan::currency("en", "chf", true),
            NumberFormatPlan::currency("en", "CHF", true)
        );
        let unknown = NumberFormatPlan::currency("en", "zzz", false).unwrap();
        assert_eq!(unknown.affixes.prefix, "ZZZ\u{a0}");
        assert!(NumberFormatPlan::currency("en", "bad-code", false).is_none());
    }
}
