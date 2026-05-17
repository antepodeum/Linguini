mod decl;
mod emit;
mod expr;
mod formatters;
mod names;
mod project;
mod tree;

use std::fmt;

use linguini_cldr::built_in_plural_rules;
use linguini_ir::IrModule;

use self::emit::{
    emit_enums, emit_forms, emit_imports, emit_index, emit_local_functions, emit_messages,
    emit_shared, emit_type_aliases,
};
use super::plural::generate_plural_function;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptOptions {
    pub locale: String,
    pub plural_function: String,
    pub plural_import: Option<String>,
    pub plural_source: Option<String>,
    pub included_messages: Vec<String>,
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            locale: "ru".to_owned(),
            plural_function: "plural".to_owned(),
            plural_import: Some("./plurals".to_owned()),
            plural_source: None,
            included_messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptLocaleModule {
    pub locale: String,
    pub module: IrModule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptProjectOptions {
    pub declaration: bool,
    pub tree_shaking: bool,
    pub included_messages: Vec<String>,
    pub base_locale: Option<String>,
}

impl Default for TypeScriptProjectOptions {
    fn default() -> Self {
        Self {
            declaration: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptGeneratedFile {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptCodegenError {
    message: String,
}

impl TypeScriptCodegenError {
    fn missing_plural_rules(locale: &str) -> Self {
        Self {
            message: format!("missing built-in CLDR plural rules for configured locale `{locale}`"),
        }
    }
}

impl fmt::Display for TypeScriptCodegenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TypeScriptCodegenError {}

pub fn generate_typescript_project_files(
    schema: &IrModule,
    locales: &[TypeScriptLocaleModule],
    options: &TypeScriptProjectOptions,
) -> Result<Vec<TypeScriptGeneratedFile>, TypeScriptCodegenError> {
    let mut files = vec![TypeScriptGeneratedFile {
        path: "shared.ts".to_owned(),
        contents: generate_shared_module(),
    }];

    if options.declaration {
        files.push(TypeScriptGeneratedFile {
            path: "shared.d.ts".to_owned(),
            contents: decl::generate_shared_declaration(),
        });
    }

    for locale in locales {
        let locale_options = project_locale_options(&locale.locale, options)?;
        let visible_schema = visible_schema(schema, &locale_options);
        files.push(TypeScriptGeneratedFile {
            path: format!("locales/{}.ts", locale.locale),
            contents: generate_typescript_module(&visible_schema, &locale.module, &locale_options),
        });
        if options.declaration {
            files.push(TypeScriptGeneratedFile {
                path: format!("locales/{}.d.ts", locale.locale),
                contents: decl::generate_locale_declaration(&visible_schema),
            });
        }
    }

    files.push(TypeScriptGeneratedFile {
        path: "index.ts".to_owned(),
        contents: project::generate_project_index(locales, options.base_locale.as_deref()),
    });
    if options.declaration {
        files.push(TypeScriptGeneratedFile {
            path: "index.d.ts".to_owned(),
            contents: project::generate_project_index_declaration(
                locales,
                options.base_locale.as_deref(),
            ),
        });
    }

    Ok(files)
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
            path: "shared.d.ts".to_owned(),
            contents: decl::generate_shared_declaration(),
        },
        TypeScriptGeneratedFile {
            path: format!("locales/{}.ts", options.locale),
            contents: generate_typescript_module(schema, locale, options),
        },
        TypeScriptGeneratedFile {
            path: format!("locales/{}.d.ts", options.locale),
            contents: decl::generate_locale_declaration(schema),
        },
        TypeScriptGeneratedFile {
            path: "index.ts".to_owned(),
            contents: generate_index_module(options),
        },
        TypeScriptGeneratedFile {
            path: "index.d.ts".to_owned(),
            contents: decl::generate_index_declaration(options),
        },
    ]
}

pub fn generate_typescript_module(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
) -> String {
    let schema = visible_schema(schema, options);
    let mut output = String::new();
    emit_imports(locale, options, &mut output);
    emit::emit_plural_helpers(options, &mut output);
    emit_enums(&schema, &mut output);
    emit_type_aliases(&schema, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, options, &mut output);
    let exports = emit_messages(&schema, locale, options, &mut output);
    emit_locale_default(&exports, &mut output);
    output
}

fn project_locale_options(
    locale: &str,
    project_options: &TypeScriptProjectOptions,
) -> Result<TypeScriptOptions, TypeScriptCodegenError> {
    let plural_function = plural_function_name(locale);
    let plural_rules = built_in_plural_rules(locale)
        .ok_or_else(|| TypeScriptCodegenError::missing_plural_rules(locale))?;
    Ok(TypeScriptOptions {
        locale: locale.to_owned(),
        plural_function: plural_function.clone(),
        plural_import: None,
        plural_source: Some(generate_plural_function(&plural_function, &plural_rules)),
        included_messages: if project_options.tree_shaking {
            project_options.included_messages.clone()
        } else {
            Vec::new()
        },
    })
}

fn visible_schema(schema: &IrModule, options: &TypeScriptOptions) -> IrModule {
    if options.included_messages.is_empty() {
        return schema.clone();
    }

    let mut visible = schema.clone();
    visible.messages.retain(|message| {
        options.included_messages.iter().any(|selected| {
            selected == &message.name
                || message
                    .name
                    .strip_prefix(selected)
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    });
    visible
}

fn plural_function_name(locale: &str) -> String {
    format!("plural{}", pascal_identifier(locale))
}

fn pascal_identifier(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut output = String::new();
            output.push(first.to_ascii_uppercase());
            output.extend(chars.map(|character| character.to_ascii_lowercase()));
            output
        })
        .collect::<String>()
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
