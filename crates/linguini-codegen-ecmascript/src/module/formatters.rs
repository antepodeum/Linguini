//! ECMAScript bindings for statically used formatter requests.

pub use linguini_codegen::{
    formatter_requests, formatter_requirements, module_uses_inline_functions, plural_required,
    FormatterRequest,
};
use linguini_ir::IrModule;

pub(super) fn formatter_binding(request: &FormatterRequest) -> String {
    let mut name = match request.formatter.kind {
        linguini_ir::IrFormatterKind::Number => "formatNumber",
        linguini_ir::IrFormatterKind::Currency => "formatCurrency",
        linguini_ir::IrFormatterKind::Date => "formatDate",
        linguini_ir::IrFormatterKind::Unknown(_) => unreachable!("validated formatter"),
    }
    .to_owned();
    for argument in &request.formatter.arguments {
        name.push('_');
        name.push_str(&argument.name);
        name.push('_');
        for byte in argument.value.bytes() {
            if byte.is_ascii_alphanumeric() {
                name.push(char::from(byte));
            } else {
                use std::fmt::Write;
                write!(name, "_{byte:02x}").expect("string write");
            }
        }
    }
    name
}

pub(super) fn formatter_helper_names(schema: &IrModule, locale: &IrModule) -> Vec<String> {
    formatter_requests(schema, locale)
        .iter()
        .map(formatter_binding)
        .collect()
}
