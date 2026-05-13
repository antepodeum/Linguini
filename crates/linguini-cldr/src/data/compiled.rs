use super::{
    CurrencyFormatData, DateFormatData, FormatWidths, NumberFormatData, PluralCategoryRule,
    PluralRules,
};
use crate::plural::{
    Condition, Operand, OperandExpression, PluralOperands, PluralRule, Range, RangeList, Relation,
    RelationOperator,
};

#[derive(Debug, Clone, Copy)]
pub struct CompiledPluralRules {
    pub locale: &'static str,
    pub categories: &'static [CompiledPluralCategory],
}

impl CompiledPluralRules {
    pub fn category_for(&self, sample: &str) -> Result<&'static str, String> {
        let operands = PluralOperands::parse(sample).map_err(|error| error.to_string())?;
        Ok(self.category_for_operands(&operands))
    }

    pub fn category_for_operands(&self, operands: &PluralOperands) -> &'static str {
        self.categories
            .iter()
            .find(|category| (category.matches)(operands))
            .map(|category| category.category)
            .unwrap_or("other")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledPluralCategory {
    pub category: &'static str,
    pub matches: fn(&PluralOperands) -> bool,
}

pub fn compiled_plural_rules(locale: &str) -> Option<CompiledPluralRules> {
    generated_plural_rules(locale)
}

pub fn built_in_plural_rules(locale: &str) -> Option<PluralRules> {
    generated_plural_rule_sources(locale)
}

include!(concat!(env!("OUT_DIR"), "/linguini_generated_plural_rules.rs"));

pub fn compiled_number_formatting(locale: &str) -> Option<NumberFormatData> {
    match locale {
        "en" => Some(NumberFormatData {
            locale: "en".to_owned(),
            decimal_symbol: ".".to_owned(),
            group_symbol: ",".to_owned(),
            decimal_pattern: "#,##0.###".to_owned(),
            percent_pattern: "#,##0%".to_owned(),
        }),
        "ru" => Some(NumberFormatData {
            locale: "ru".to_owned(),
            decimal_symbol: ",".to_owned(),
            group_symbol: "\u{a0}".to_owned(),
            decimal_pattern: "#,##0.###".to_owned(),
            percent_pattern: "#,##0%".to_owned(),
        }),
        _ => None,
    }
}

pub fn compiled_currency_formatting(locale: &str) -> Option<CurrencyFormatData> {
    match locale {
        "en" => Some(CurrencyFormatData {
            locale: "en".to_owned(),
            standard_pattern: "\u{a4}#,##0.00".to_owned(),
            accounting_pattern: Some("(\u{a4}#,##0.00)".to_owned()),
        }),
        "ru" => Some(CurrencyFormatData {
            locale: "ru".to_owned(),
            standard_pattern: "#,##0.00\u{a0}\u{a4}".to_owned(),
            accounting_pattern: None,
        }),
        _ => None,
    }
}

pub fn compiled_date_formatting(locale: &str) -> Option<DateFormatData> {
    match locale {
        "en" => Some(DateFormatData {
            locale: "en".to_owned(),
            date_formats: widths("EEEE, MMMM d, y", "MMMM d, y", "MMM d, y", "M/d/yy"),
            time_formats: widths("h:mm:ss a zzzz", "h:mm:ss a z", "h:mm:ss a", "h:mm a"),
            date_time_formats: widths("{1}, {0}", "{1}, {0}", "{1}, {0}", "{1}, {0}"),
        }),
        "ru" => Some(DateFormatData {
            locale: "ru".to_owned(),
            date_formats: widths(
                "EEEE, d MMMM y\u{a0}'\u{433}'.",
                "d MMMM y\u{a0}'\u{433}'.",
                "d MMM y\u{a0}'\u{433}'.",
                "dd.MM.y",
            ),
            time_formats: widths("HH:mm:ss zzzz", "HH:mm:ss z", "HH:mm:ss", "HH:mm"),
            date_time_formats: widths("{1}, {0}", "{1}, {0}", "{1}, {0}", "{1}, {0}"),
        }),
        _ => None,
    }
}

fn widths(full: &str, long: &str, medium: &str, short: &str) -> FormatWidths {
    FormatWidths {
        full: full.to_owned(),
        long: long.to_owned(),
        medium: medium.to_owned(),
        short: short.to_owned(),
    }
}
