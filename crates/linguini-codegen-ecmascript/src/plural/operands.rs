//! CLDR plural operands with one runtime body for both ECMAScript targets.

use crate::ecmascript::EcmaScriptTarget;
use crate::templates;
use minijinja::context;

pub(super) fn render(target: EcmaScriptTarget) -> String {
    templates::render("plural_operands", target, context! {})
}
