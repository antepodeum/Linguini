//! Demand-selected numeric, date, and digit helper templates.

use crate::templates;
use minijinja::context;

pub(super) fn digit_formatter_helper() -> String {
    templates::render("digit_helpers", context! {})
}

pub(super) fn date_formatter_helpers() -> String {
    templates::render("date_helpers", context! {})
}
