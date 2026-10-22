//! Build-time rendering of shared ECMAScript source templates.

use std::sync::OnceLock;

use crate::EcmaTemplateEngine;
use minijinja::Value;

fn environment() -> &'static EcmaTemplateEngine {
    static ENVIRONMENT: OnceLock<EcmaTemplateEngine> = OnceLock::new();
    ENVIRONMENT.get_or_init(|| {
        EcmaTemplateEngine::new([
            (
                "types/documentation",
                include_str!("../templates/types/documentation.jinja"),
            ),
            (
                "types/object",
                include_str!("../templates/types/object.jinja"),
            ),
            (
                "types/doc_tag",
                include_str!("../templates/types/doc_tag.jinja"),
            ),
            (
                "types/signature",
                include_str!("../templates/types/signature.jinja"),
            ),
            (
                "declarations/constant",
                include_str!("../templates/declarations/constant.jinja"),
            ),
            (
                "declarations/function",
                include_str!("../templates/declarations/function.jinja"),
            ),
            (
                "declarations/callable",
                include_str!("../templates/declarations/callable.jinja"),
            ),
            (
                "declarations/alias",
                include_str!("../templates/declarations/alias.jinja"),
            ),
            (
                "declarations/default",
                include_str!("../templates/declarations/default.jinja"),
            ),
            (
                "declarations/property",
                include_str!("../templates/declarations/property.jinja"),
            ),
            (
                "module/exports",
                include_str!("../templates/module/exports.jinja"),
            ),
            (
                "module/import_typedefs",
                include_str!("../templates/module/import_typedefs.jinja"),
            ),
            (
                "index_declarations",
                include_str!("../templates/index_declarations.jinja"),
            ),
            (
                "module/constant",
                include_str!("../templates/module/constant.jinja"),
            ),
            (
                "module/form",
                include_str!("../templates/module/form.jinja"),
            ),
            (
                "module/message",
                include_str!("../templates/module/message.jinja"),
            ),
            (
                "module/message_property",
                include_str!("../templates/module/message_property.jinja"),
            ),
            (
                "module/object",
                include_str!("../templates/module/object.jinja"),
            ),
            (
                "module/locale_default",
                include_str!("../templates/module/locale_default.jinja"),
            ),
            (
                "module/normalization",
                include_str!("../templates/module/normalization.jinja"),
            ),
            ("functions", include_str!("../templates/functions.jinja")),
            ("expression", include_str!("../templates/expression.jinja")),
            (
                "formatters/date",
                include_str!("../templates/formatters/date.jinja"),
            ),
            (
                "formatters/date_parts",
                include_str!("../templates/formatters/date_parts.jinja"),
            ),
            (
                "formatters/types",
                include_str!("../templates/formatters/types.jinja"),
            ),
            (
                "formatter_call",
                include_str!("../templates/formatter_call.jinja"),
            ),
            (
                "formatters/number",
                include_str!("../templates/formatters/number.jinja"),
            ),
            ("dispatch", include_str!("../templates/dispatch.jinja")),
            (
                "dispatch_function",
                include_str!("../templates/dispatch_function.jinja"),
            ),
            ("selector", include_str!("../templates/selector.jinja")),
            (
                "local_function",
                include_str!("../templates/local_function.jinja"),
            ),
            ("plural", include_str!("../templates/plural.jinja")),
            (
                "plural_predicates",
                include_str!("../templates/plural_predicates.jinja"),
            ),
            (
                "project/locale",
                include_str!("../templates/project/locale.jinja"),
            ),
            (
                "project/locale_declarations",
                include_str!("../templates/project/locale_declarations.jinja"),
            ),
            (
                "project/index",
                include_str!("../templates/project/index.jinja"),
            ),
            (
                "number_rounding",
                include_str!("../templates/number_rounding.jinja"),
            ),
            (
                "number_helpers",
                include_str!("../templates/number_helpers.jinja"),
            ),
            (
                "digit_helpers",
                include_str!("../templates/digit_helpers.jinja"),
            ),
            (
                "date_helpers",
                include_str!("../templates/date_helpers.jinja"),
            ),
            (
                "plural_input",
                include_str!("../templates/plural_input.jinja"),
            ),
            (
                "plural_operands",
                include_str!("../templates/plural_operands.jinja"),
            ),
            (
                "provider_runtime",
                include_str!("../templates/provider_runtime.jinja"),
            ),
            ("shared", include_str!("../templates/shared.jinja")),
            (
                "shared_declarations",
                include_str!("../templates/shared_declarations.jinja"),
            ),
        ])
    })
}

pub(crate) fn render(name: &str, values: Value) -> String {
    environment().render(name, values)
}
