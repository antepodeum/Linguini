mod emit;
mod expr;
mod names;

use linguini_ir::IrModule;

use self::emit::{
    emit_enums, emit_forms, emit_imports, emit_index, emit_local_functions, emit_messages,
    emit_shared, emit_type_aliases,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptOptions {
    pub locale: String,
    pub plural_function: String,
    pub plural_import: Option<String>,
    pub plural_source: Option<String>,
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            locale: "ru".to_owned(),
            plural_function: "plural".to_owned(),
            plural_import: Some("./plurals".to_owned()),
            plural_source: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptGeneratedFile {
    pub path: String,
    pub contents: String,
}

pub fn generate_typescript_files(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
) -> Vec<TypeScriptGeneratedFile> {
    vec![
        TypeScriptGeneratedFile {
            path: "shared.ts".to_owned(),
            contents: generate_shared_module(),
        },
        TypeScriptGeneratedFile {
            path: format!("locales/{}.ts", options.locale),
            contents: generate_typescript_module(schema, locale, options),
        },
        TypeScriptGeneratedFile {
            path: "index.ts".to_owned(),
            contents: generate_index_module(options),
        },
    ]
}

pub fn generate_typescript_module(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
) -> String {
    let mut output = String::new();
    emit_imports(locale, options, &mut output);
    emit::emit_plural_helpers(options, &mut output);
    emit_enums(schema, &mut output);
    emit_type_aliases(schema, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, &mut output);
    let exports = emit_messages(schema, locale, &mut output);
    emit_locale_default(&exports, &mut output);
    output
}

fn generate_shared_module() -> String {
    let mut output = String::new();
    emit_shared(&mut output);
    output
}

fn generate_index_module(options: &TypeScriptOptions) -> String {
    let mut output = String::new();
    emit_index(options, &mut output);
    output
}

fn emit_locale_default(exports: &emit::ModuleExports, output: &mut String) {
    output.push_str("const lgl = {\n");
    for name in exports.top_level.iter().chain(exports.groups.iter()) {
        output.push_str(&format!("  {name},\n"));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export default lgl;\n");
}
