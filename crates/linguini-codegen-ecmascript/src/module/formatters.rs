//! ECMAScript helper bindings for backend-independent capability requirements.

pub use linguini_codegen::{
    formatter_requirements, module_uses_inline_functions, plural_required, FormatterRequirements,
};

pub(super) fn formatter_helper_names(requirements: FormatterRequirements) -> Vec<&'static str> {
    let mut names = Vec::new();
    if requirements.number {
        names.push("formatNumber");
    }
    if requirements.currency {
        names.push("formatCurrency");
    }
    if requirements.date {
        names.push("formatDate");
    }
    names
}
