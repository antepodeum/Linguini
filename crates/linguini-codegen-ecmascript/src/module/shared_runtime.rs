//! Shared schema helpers, rendered only during project generation.

use crate::templates;
use minijinja::context;

pub(super) fn shared_runtime() -> String {
    templates::render("shared", context! {})
}

pub(super) fn shared_declarations() -> String {
    templates::render("shared_declarations", context! {})
}
