//! Shared lazy locale preparation and message provider runtime.

use crate::ecmascript::EcmaScriptTarget;
use crate::templates;
use minijinja::context;

pub(super) fn runtime(target: EcmaScriptTarget) -> String {
    templates::render("provider_runtime", target, context! {})
}
