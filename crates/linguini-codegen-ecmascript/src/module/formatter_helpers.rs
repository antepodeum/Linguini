//! Demand-selected numeric, date, and digit helper templates.

use crate::ecmascript::EcmaScriptTarget;
use crate::templates;
use minijinja::context;

pub(super) fn number_formatter_helpers(target: EcmaScriptTarget) -> String {
    templates::render("number_helpers", target, context! {})
}

pub(super) fn digit_formatter_helper(target: EcmaScriptTarget) -> String {
    templates::render("digit_helpers", target, context! {})
}

pub(super) fn date_formatter_helpers(target: EcmaScriptTarget) -> String {
    templates::render("date_helpers", target, context! {})
}
