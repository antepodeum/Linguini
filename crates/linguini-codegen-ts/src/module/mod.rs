mod decl;
mod emit;
mod expr;
mod formatters;
mod names;
mod project;
mod shared;
mod templates;
mod tree;

use std::fmt;

use linguini_cldr::built_in_plural_rules;
use linguini_ir::IrModule;

use self::emit::{
    emit_formatter_data, emit_forms, emit_imports, emit_index, emit_local_functions, emit_messages,
    emit_schema_type_reexports, emit_variables,
};
use self::shared::emit_shared;
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
    pub gitignore: bool,
    pub tree_shaking: bool,
    pub included_messages: Vec<String>,
    pub base_locale: Option<String>,
    pub web: Option<TypeScriptWebOptions>,
    pub framework: Option<TypeScriptFramework>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeScriptFramework {
    Svelte,
    SvelteKit,
}

impl TypeScriptFramework {
    pub fn from_config(value: Option<&str>) -> Option<Self> {
        match value {
            Some("svelte") => Some(Self::Svelte),
            Some("sveltekit") => Some(Self::SvelteKit),
            _ => None,
        }
    }

    fn needs_svelte_module(self) -> bool {
        matches!(self, Self::Svelte | Self::SvelteKit)
    }

    fn needs_sveltekit_module(self) -> bool {
        matches!(self, Self::SvelteKit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptWebOptions {
    pub strategy: Vec<String>,
    pub cookie_name: String,
    pub cookie_path: String,
    pub cookie_domain: Option<String>,
    pub cookie_max_age: u64,
    pub cookie_same_site: String,
    pub cookie_secure: bool,
    pub cookie_http_only: bool,
    pub local_storage_key: String,
    pub global_variable_name: Option<String>,
    pub prefix_default_locale: bool,
    pub base_path: String,
    pub trailing_slash: String,
    pub redirect: bool,
    pub origin: Option<String>,
    pub exclude: Vec<String>,
    pub localize_links: bool,
}

impl Default for TypeScriptProjectOptions {
    fn default() -> Self {
        Self {
            declaration: true,
            gitignore: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: None,
            web: None,
            framework: None,
        }
    }
}

impl Default for TypeScriptWebOptions {
    fn default() -> Self {
        Self {
            strategy: vec![
                "url".to_owned(),
                "cookie".to_owned(),
                "localStorage".to_owned(),
                "preferredLanguage".to_owned(),
                "baseLocale".to_owned(),
            ],
            cookie_name: "LINGUINI_LOCALE".to_owned(),
            cookie_path: "/".to_owned(),
            cookie_domain: None,
            cookie_max_age: 60 * 60 * 24 * 365,
            cookie_same_site: "lax".to_owned(),
            cookie_secure: false,
            cookie_http_only: false,
            local_storage_key: "LINGUINI_LOCALE".to_owned(),
            global_variable_name: None,
            prefix_default_locale: false,
            base_path: String::new(),
            trailing_slash: "ignore".to_owned(),
            redirect: true,
            origin: None,
            exclude: Vec::new(),
            localize_links: true,
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
        contents: generate_shared_module(schema),
    }];

    if options.gitignore {
        files.push(TypeScriptGeneratedFile {
            path: ".gitignore".to_owned(),
            contents: "# Generated by Linguini. Do not edit.\n*\n".to_owned(),
        });
    }

    if options.declaration {
        files.push(TypeScriptGeneratedFile {
            path: "shared.d.ts".to_owned(),
            contents: decl::generate_shared_declaration(schema),
        });
    }

    let fallback_locales = fallback_locale_modules(locales, options.base_locale.as_deref());

    for locale in &fallback_locales {
        let locale_options = project_locale_options(&locale.locale, options)?;
        let visible_schema = visible_schema(schema, &locale_options);
        let namespaces = top_level_namespaces(&visible_schema);
        for namespace in &namespaces {
            let namespace_schema = namespace_module(&visible_schema, namespace);
            let namespace_locale = namespace_module(&locale.module, namespace);
            files.push(TypeScriptGeneratedFile {
                path: format!("locales/{}/{}.ts", locale.locale, namespace),
                contents: generate_typescript_module_with_shared_import(
                    &namespace_schema,
                    &namespace_locale,
                    &locale_options,
                    "../../shared",
                    Some(namespace),
                ),
            });
            if options.declaration {
                files.push(TypeScriptGeneratedFile {
                    path: format!("locales/{}/{}.d.ts", locale.locale, namespace),
                    contents: decl::generate_locale_declaration_with_shared_import(
                        &namespace_schema,
                        "../../shared",
                        Some(namespace),
                    ),
                });
            }
        }

        let (barrel_schema, barrel_locale) = if namespaces.is_empty() {
            (visible_schema.clone(), locale.module.clone())
        } else {
            (
                root_module(&visible_schema),
                root_module_with_locale_items(&locale.module),
            )
        };
        files.push(TypeScriptGeneratedFile {
            path: format!("locales/{}.ts", locale.locale),
            contents: generate_typescript_module_with_namespaces(
                &barrel_schema,
                &barrel_locale,
                &locale_options,
                &namespaces,
            ),
        });
        if options.declaration {
            files.push(TypeScriptGeneratedFile {
                path: format!("locales/{}.d.ts", locale.locale),
                contents: decl::generate_locale_declaration_with_namespaces(
                    &barrel_schema,
                    &locale.locale,
                    &namespaces,
                ),
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

    if options
        .framework
        .is_some_and(TypeScriptFramework::needs_svelte_module)
    {
        if options.web.is_some() {
            files.push(TypeScriptGeneratedFile {
                path: "web.ts".to_owned(),
                contents: project::generate_project_web_module(),
            });
            if options.declaration {
                files.push(TypeScriptGeneratedFile {
                    path: "web.d.ts".to_owned(),
                    contents: project::generate_project_web_declaration(),
                });
            }
        }
        files.push(TypeScriptGeneratedFile {
            path: "svelte.ts".to_owned(),
            contents: project::generate_project_svelte_module(options.web.as_ref()),
        });
        if options.declaration {
            files.push(TypeScriptGeneratedFile {
                path: "svelte.d.ts".to_owned(),
                contents: project::generate_project_svelte_declaration(options.web.is_some()),
            });
        }
    }

    if let Some(web) = options.web.as_ref().filter(|_| {
        options
            .framework
            .is_some_and(TypeScriptFramework::needs_sveltekit_module)
    }) {
        files.push(TypeScriptGeneratedFile {
            path: "sveltekit.ts".to_owned(),
            contents: project::generate_project_sveltekit_module(web),
        });
        if options.declaration {
            files.push(TypeScriptGeneratedFile {
                path: "sveltekit.d.ts".to_owned(),
                contents: project::generate_project_sveltekit_declaration(),
            });
        }
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
            contents: generate_shared_module(schema),
        },
        TypeScriptGeneratedFile {
            path: "shared.d.ts".to_owned(),
            contents: decl::generate_shared_declaration(schema),
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
    generate_typescript_module_with_namespaces(schema, locale, options, &[])
}

fn generate_typescript_module_with_namespaces(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    namespaces: &[String],
) -> String {
    let schema = visible_schema(schema, options);
    let mut output = String::new();
    for namespace in namespaces {
        output.push_str(&format!(
            "import {{ {} }} from \"./{}/{}\";\n",
            namespace, options.locale, namespace
        ));
    }
    emit_imports(&schema, locale, options, "../shared", &mut output);
    if !namespaces.is_empty() {
        output.push('\n');
    }
    emit::emit_plural_helpers(options, &mut output);
    emit_formatter_data(&schema, locale, options, &mut output);
    emit_schema_type_reexports(&schema, "../shared", &mut output);
    for namespace in namespaces {
        output.push_str(&format!("export {{ {namespace} }};\n\n"));
    }
    emit_variables(locale, options, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, options, &mut output);
    let exports = emit_messages(&schema, locale, options, &mut output);
    emit_locale_default(&exports, namespaces, &mut output);
    output
}

fn generate_typescript_module_with_shared_import(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    shared_import_path: &str,
    namespace_alias: Option<&str>,
) -> String {
    let schema = visible_schema(schema, options);
    let mut output = String::new();
    emit_imports(&schema, locale, options, shared_import_path, &mut output);
    emit::emit_plural_helpers(options, &mut output);
    emit_formatter_data(&schema, locale, options, &mut output);
    emit_schema_type_reexports(&schema, shared_import_path, &mut output);
    emit_variables(locale, options, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, options, &mut output);
    let exports = emit_messages(&schema, locale, options, &mut output);
    emit_locale_default(&exports, &[], &mut output);
    if let Some(namespace_alias) = namespace_alias {
        let alias_is_exported = exports
            .top_level
            .iter()
            .chain(exports.groups.iter())
            .any(|export| export == namespace_alias);
        if !alias_is_exported {
            output.push_str(&format!("\nexport const {namespace_alias} = lgl;\n"));
        }
    }
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

fn top_level_namespaces(module: &IrModule) -> Vec<String> {
    let mut namespaces = module
        .messages
        .iter()
        .filter_map(|message| message.name.split_once('.').map(|(namespace, _)| namespace))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    namespaces.sort();
    namespaces.dedup();
    namespaces
}

fn namespace_module(module: &IrModule, namespace: &str) -> IrModule {
    let prefix = format!("{namespace}.");
    let mut output = module.clone();
    output.messages = module
        .messages
        .iter()
        .filter(|message| message.name.starts_with(&prefix))
        .cloned()
        .collect();
    output
}

fn root_module(module: &IrModule) -> IrModule {
    let mut output = module.clone();
    output
        .messages
        .retain(|message| !message.name.contains('.'));
    output
}

fn root_module_with_locale_items(module: &IrModule) -> IrModule {
    let mut output = root_module(module);
    output.forms.clear();
    output.functions.clear();
    output.variables.clear();
    output
}

fn fallback_locale_modules(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> Vec<TypeScriptLocaleModule> {
    locales
        .iter()
        .map(|locale| TypeScriptLocaleModule {
            locale: locale.locale.clone(),
            module: fallback_locale_module(locales, &locale.locale, base_locale),
        })
        .collect()
}

fn fallback_locale_module(
    locales: &[TypeScriptLocaleModule],
    locale: &str,
    base_locale: Option<&str>,
) -> IrModule {
    let mut chain = locale_fallback_chain(locales, locale, base_locale);
    chain.reverse();

    let mut merged = IrModule::default();
    for fallback_locale in chain {
        if let Some(source) = locales.iter().find(|entry| entry.locale == fallback_locale) {
            merge_locale_module(&mut merged, &source.module);
        }
    }
    merged
}

fn locale_fallback_chain(
    locales: &[TypeScriptLocaleModule],
    locale: &str,
    base_locale: Option<&str>,
) -> Vec<String> {
    let mut chain = Vec::new();
    for tag in locale_fallback_tags(locale) {
        if let Some(exact) = locales
            .iter()
            .find(|entry| entry.locale.eq_ignore_ascii_case(&tag))
            .map(|entry| entry.locale.clone())
        {
            if !chain.contains(&exact) {
                chain.push(exact);
            }
        }
    }
    let base = base_locale
        .filter(|candidate| locales.iter().any(|entry| entry.locale == *candidate))
        .or_else(|| locales.first().map(|entry| entry.locale.as_str()));
    if let Some(base) = base {
        if !chain.iter().any(|entry| entry == base) {
            chain.push(base.to_owned());
        }
    }
    chain
}

fn locale_fallback_tags(locale: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut tag = locale.to_owned();
    while !tag.is_empty() {
        tags.push(tag.clone());
        let Some(dash) = tag.rfind('-') else {
            break;
        };
        if dash == 0 {
            break;
        }
        tag.truncate(dash);
    }
    tags
}

fn merge_locale_module(target: &mut IrModule, source: &IrModule) {
    merge_named_items(&mut target.messages, &source.messages, |message| {
        &message.name
    });
    merge_named_items(&mut target.variables, &source.variables, |variable| {
        &variable.name
    });
    merge_named_items(&mut target.forms, &source.forms, |form| &form.name);
    merge_named_items(&mut target.functions, &source.functions, |function| {
        &function.name
    });
}

fn merge_named_items<T: Clone>(target: &mut Vec<T>, source: &[T], key: impl Fn(&T) -> &str) {
    for item in source {
        let name = key(item);
        if let Some(existing) = target.iter_mut().find(|existing| key(existing) == name) {
            *existing = item.clone();
        } else {
            target.push(item.clone());
        }
    }
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

fn generate_shared_module(schema: &IrModule) -> String {
    let mut output = String::new();
    emit_shared(schema, &mut output);
    output
}

fn generate_index_module(options: &TypeScriptOptions) -> String {
    let mut output = String::new();
    emit_index(options, &mut output);
    output
}

fn emit_locale_default(exports: &emit::ModuleExports, namespaces: &[String], output: &mut String) {
    output.push_str("const lgl = {\n");
    for name in exports
        .top_level
        .iter()
        .chain(exports.groups.iter())
        .chain(namespaces.iter())
    {
        output.push_str(&format!("  {name},\n"));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export default lgl;\n");
}
