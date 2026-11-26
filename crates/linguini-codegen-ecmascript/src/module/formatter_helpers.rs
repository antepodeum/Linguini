//! Demand-selected numeric, date, and digit helper templates.

use crate::templates;
use minijinja::context;

pub(super) fn digit_formatter_helper(digits: &str) -> String {
    let symbols =
        linguini_codegen::decimal_digit_symbols(digits).expect("validated CLDR decimal symbols");
    let single_unit = symbols.iter().all(|symbol| symbol.len_utf16() == 1);
    let symbols = symbols.map(|symbol| symbol.to_string());
    templates::render("digit_helpers", context! { digits, symbols, single_unit })
}

pub(super) fn date_formatter_helpers(padding: bool) -> String {
    templates::render("date_helpers", context! { padding })
}
