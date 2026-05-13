mod emit;
mod expr;
mod names;

use linguini_ir::IrModule;

use self::emit::{
    emit_branch_helper, emit_enums, emit_forms, emit_imports, emit_local_functions,
    emit_locale_facade, emit_messages, emit_type_aliases,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptOptions {
    pub locale: String,
    pub plural_function: String,
    pub plural_import: Option<String>,
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            locale: "ru".to_owned(),
            plural_function: "plural".to_owned(),
            plural_import: Some("./plurals".to_owned()),
        }
    }
}

pub fn generate_typescript_module(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
) -> String {
    let mut output = String::new();
    emit_imports(locale, options, &mut output);
    emit_enums(schema, &mut output);
    emit_type_aliases(schema, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, &mut output);
    let exports = emit_messages(schema, locale, &mut output);
    emit_locale_facade(&exports, options, &mut output);
    emit_branch_helper(locale, &mut output);
    output
}
