//! Shared schema helpers, rendered only during project generation.

use crate::ecmascript::EcmaScriptTarget;
use crate::templates;
use minijinja::context;

pub(super) fn shared_runtime(target: EcmaScriptTarget) -> String {
    templates::render("shared", target, context! {})
}

pub(super) fn shared_declarations() -> String {
    templates::render(
        "shared_declarations",
        EcmaScriptTarget::TypeScript,
        context! {},
    )
}
