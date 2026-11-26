//! Demand-selected numeric, date, and digit helper templates.

use crate::templates;
use minijinja::context;

pub(super) fn digit_lookup(digits: &str) -> minijinja::Value {
    let symbols =
        linguini_codegen::decimal_digit_symbols(digits).expect("validated CLDR decimal symbols");
    let single_unit = symbols.iter().all(|symbol| symbol.len_utf16() == 1);
    let code_point_offset = linguini_codegen::contiguous_decimal_digit_zero(&symbols)
        .filter(|_| !single_unit)
        .map(|zero| u32::from(zero) - u32::from('0'));
    let symbols = symbols.map(|symbol| symbol.to_string());
    context! { digits, symbols, single_unit, code_point_offset }
}

pub(super) fn digit_formatter_helper(digits: &str) -> String {
    templates::render("digit_helpers", context! { lookup => digit_lookup(digits) })
}

pub(super) fn date_formatter_helpers(padding: bool) -> String {
    templates::render("date_helpers", context! { padding })
}

pub(super) fn week_year_formatter_helper(locale: &str) -> String {
    let plan = linguini_codegen::GregorianWeekYearPlan::for_locale(locale)
        .expect("validated Gregorian week-year data");
    templates::render(
        "week_year_helpers",
        context! {
            weekday_shift => plan.weekday_shift, anchor_offset => plan.anchor_offset,
            previous_last_day => plan.previous_last_day, next_first_day => plan.next_first_day,
        },
    )
}
