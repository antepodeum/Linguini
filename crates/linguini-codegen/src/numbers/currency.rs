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
        Self::currency_with_display(locale, code, accounting, "symbol")
    }

    /// Resolve a localized symbol or ISO-code display before selecting the complete pattern.
    /// Unsupported displays, locales, malformed codes, or invalid plans return `None`.
    pub fn currency_with_display(
        locale: &str,
        code: &str,
        accounting: bool,
        display: &str,
    ) -> Option<Self> {
        let code = code.to_ascii_uppercase();
        let numbers = compiled_number_formatting(locale)?;
        let currency = compiled_currency_formatting(locale)?;
        let fraction = compiled_currency_fraction(&code)?;
        let symbol = match display {
            "symbol" => compiled_currency_symbol(locale, &code).unwrap_or(&code),
            "code" => &code,
            _ => return None,
        };
        let mut pattern = select_pattern(&currency, symbol, accounting);
        pattern.positive.min_fraction_digits = fraction.digits;
        pattern.positive.max_fraction_digits = fraction.digits;
        let numbers = linguini_cldr::NumberFormatData {
            decimal_symbol: currency.decimal_symbol,
            group_symbol: currency.group_symbol,
            ..numbers
        };
        Self::compile(
            &pattern,
            Some(symbol),
            Some(&code),
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
            currency_boundary_is_letter(base.positive.prefix, symbol, true)
                || currency_boundary_is_letter(base.positive.suffix, symbol, false)
        })
        .unwrap_or(base)
}

fn currency_boundary_is_letter(affix: NumberAffix, symbol: &str, prefix: bool) -> bool {
    let symbol_is_letter = if prefix {
        symbol.chars().next_back()
    } else {
        symbol.chars().next()
    }
    .is_some_and(is_letter);
    let slot = |part: &NumberAffixPart| match part {
        NumberAffixPart::Literal(_) => None,
        NumberAffixPart::CurrencySymbol => Some(symbol_is_letter),
        // Resolved ISO currency codes consist of three ASCII letters.
        NumberAffixPart::CurrencyCode => Some(true),
    };
    match affix {
        NumberAffix::Literal(_) => false,
        NumberAffix::CurrencySymbol => symbol_is_letter,
        NumberAffix::CurrencyCode => true,
        NumberAffix::Parts(parts) => if prefix {
            parts.iter().rev().find_map(slot)
        } else {
            parts.iter().find_map(slot)
        }
        .unwrap_or(false),
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
    fn currency_codes_use_the_slot_nearest_the_number_for_letter_boundaries() {
        for (affix, prefix, suffix) in [
            (NumberAffix::CurrencyCode, true, true),
            (NumberAffix::CurrencySymbol, false, false),
            (NumberAffix::Literal("¤¤"), false, false),
            (
                NumberAffix::Parts(&[
                    NumberAffixPart::CurrencyCode,
                    NumberAffixPart::CurrencySymbol,
                ]),
                false,
                true,
            ),
            (
                NumberAffix::Parts(&[
                    NumberAffixPart::CurrencySymbol,
                    NumberAffixPart::CurrencyCode,
                ]),
                true,
                false,
            ),
        ] {
            let mut currency = compiled_currency_formatting("en").unwrap();
            let alternative = *currency.standard_alpha_next_to_number.unwrap();
            currency.standard_pattern.positive.prefix = affix;
            currency.standard_pattern.positive.suffix = NumberAffix::Literal("");
            assert_eq!(
                select_pattern(&currency, "$", false),
                if prefix {
                    alternative
                } else {
                    currency.standard_pattern
                }
            );
            currency.standard_pattern.positive.prefix = NumberAffix::Literal("");
            currency.standard_pattern.positive.suffix = affix;
            assert_eq!(
                select_pattern(&currency, "$", false),
                if suffix {
                    alternative
                } else {
                    currency.standard_pattern
                }
            );
        }
    }

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
