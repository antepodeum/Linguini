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
use linguini_ir::{validate_ir, IrModule, IrReferenceError, ValidatedIr};

use self::emit::{
    emit_formatter_data, emit_forms, emit_imports, emit_local_functions, emit_messages,
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
pub enum TypeScriptCodegenError {
    InvalidIr {
        scope: String,
        errors: Vec<IrReferenceError>,
    },
    MissingPluralRules {
        locale: String,
    },
}

impl TypeScriptCodegenError {
    fn missing_plural_rules(locale: &str) -> Self {
        Self::MissingPluralRules {
            locale: locale.to_owned(),
        }
    }

    fn invalid_ir(scope: impl Into<String>, errors: Vec<IrReferenceError>) -> Self {
        Self::InvalidIr {
            scope: scope.into(),
            errors,
        }
    }
}

impl fmt::Display for TypeScriptCodegenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIr { scope, errors } => {
                write!(formatter, "invalid IR for {scope}")?;
                for error in errors {
                    write!(formatter, "\n{}: {}", error.code, error.message)?;
                }
                Ok(())
            }
            Self::MissingPluralRules { locale } => write!(
                formatter,
                "missing built-in CLDR plural rules for configured locale `{locale}`"
            ),
        }
    }
}

impl std::error::Error for TypeScriptCodegenError {}

/// Validated input for project-level TypeScript generation.
///
/// Construction validates the schema and every fallback-composed locale module. Its private
/// fields prevent production callers from bypassing the IR validation boundary.
#[derive(Debug)]
pub struct ValidatedTypeScriptProject<'a> {
    schema: &'a IrModule,
    locales: Vec<TypeScriptLocaleModule>,
    options: TypeScriptProjectOptions,
}

impl<'a> ValidatedTypeScriptProject<'a> {
    pub fn try_new(
        schema: &'a IrModule,
        locales: &[TypeScriptLocaleModule],
        options: &TypeScriptProjectOptions,
    ) -> Result<Self, TypeScriptCodegenError> {
        let empty_locale = IrModule::default();
        validate_codegen_ir(schema, &empty_locale, "schema")?;

        let locales = fallback_locale_modules(locales, options.base_locale.as_deref());
        for locale in &locales {
            validate_codegen_ir(
                schema,
                &locale.module,
                format!("locale `{}`", locale.locale),
            )?;
        }

        Ok(Self {
            schema,
            locales,
            options: options.clone(),
        })
    }
}

pub fn generate_typescript_project_files(
    project: &ValidatedTypeScriptProject<'_>,
) -> Result<Vec<TypeScriptGeneratedFile>, TypeScriptCodegenError> {
    let schema = project.schema;
    let locales = &project.locales;
    let options = &project.options;
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

    for locale in locales {
        let locale_options = project_locale_options(&locale.locale, options)?;
        let visible_schema = visible_schema(schema, &locale_options);
        let visible_locale = locale_module_for_schema(&locale.module, &visible_schema);
        let namespaces = top_level_namespaces(&visible_schema);
        for namespace in &namespaces {
            let namespace_schema = namespace_module(&visible_schema, namespace);
            let namespace_locale = namespace_module(&visible_locale, namespace);
            let validated = validate_codegen_ir(
                &namespace_schema,
                &namespace_locale,
                format!("locale `{}` namespace `{namespace}`", locale.locale),
            )?;
            files.push(TypeScriptGeneratedFile {
                path: format!("locales/{}/{}.ts", locale.locale, namespace),
                contents: generate_typescript_module_with_shared_import(
                    &validated,
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
            (visible_schema.clone(), visible_locale)
        } else {
            (
                root_module(&visible_schema),
                root_module_with_locale_items(&visible_locale),
            )
        };
        let validated = validate_codegen_ir(
            &barrel_schema,
            &barrel_locale,
            format!("locale `{}` barrel", locale.locale),
        )?;
        files.push(TypeScriptGeneratedFile {
            path: format!("locales/{}.ts", locale.locale),
            contents: generate_typescript_module_with_namespaces(
                &validated,
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

fn generate_typescript_module_with_namespaces(
    ir: &ValidatedIr<'_>,
    options: &TypeScriptOptions,
    namespaces: &[String],
) -> String {
    generate_typescript_module_unchecked(ir.schema(), ir.locale(), options, namespaces)
}

fn generate_typescript_module_unchecked(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    namespaces: &[String],
) -> String {
    let mut output = String::new();
    for namespace in namespaces {
        output.push_str(&format!(
            "import {{ {} }} from \"./{}/{}\";\n",
            namespace, options.locale, namespace
        ));
    }
    emit_imports(schema, locale, options, "../shared", &mut output);
    if !namespaces.is_empty() {
        output.push('\n');
    }
    emit::emit_plural_helpers(options, &mut output);
    emit_formatter_data(schema, locale, options, &mut output);
    emit_schema_type_reexports(schema, "../shared", &mut output);
    for namespace in namespaces {
        output.push_str(&format!("export {{ {namespace} }};\n\n"));
    }
    emit_variables(locale, options, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, options, &mut output);
    let exports = emit_messages(schema, locale, options, &mut output);
    emit_locale_default(&exports, namespaces, &mut output);
    output
}

#[cfg(test)]
pub(crate) fn generate_unvalidated_typescript_module_for_test(
    schema: &IrModule,
    locale: &IrModule,
    locale_name: &str,
) -> String {
    let options = project_locale_options(locale_name, &TypeScriptProjectOptions::default())
        .expect("test locale must have built-in plural rules");
    generate_typescript_module_unchecked(schema, locale, &options, &[])
}

fn generate_typescript_module_with_shared_import(
    ir: &ValidatedIr<'_>,
    options: &TypeScriptOptions,
    shared_import_path: &str,
    namespace_alias: Option<&str>,
) -> String {
    let schema = ir.schema();
    let locale = ir.locale();
    let mut output = String::new();
    emit_imports(schema, locale, options, shared_import_path, &mut output);
    emit::emit_plural_helpers(options, &mut output);
    emit_formatter_data(schema, locale, options, &mut output);
    emit_schema_type_reexports(schema, shared_import_path, &mut output);
    emit_variables(locale, options, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, options, &mut output);
    let exports = emit_messages(schema, locale, options, &mut output);
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

fn validate_codegen_ir<'a>(
    schema: &'a IrModule,
    locale: &'a IrModule,
    scope: impl Into<String>,
) -> Result<ValidatedIr<'a>, TypeScriptCodegenError> {
    validate_ir(schema, locale).map_err(|errors| TypeScriptCodegenError::invalid_ir(scope, errors))
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

fn locale_module_for_schema(locale: &IrModule, schema: &IrModule) -> IrModule {
    let mut visible = locale.clone();
    visible.messages.retain(|message| {
        schema
            .messages
            .iter()
            .any(|schema_message| schema_message.name == message.name)
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
    root_module(module)
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
    merge_named_items(&mut target.enums, &source.enums, |item| &item.name);
    merge_named_items(&mut target.type_aliases, &source.type_aliases, |item| {
        &item.name
    });
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
    for source_origin in &source.origins {
        let mut origin = source_origin.clone();
        if target
            .origins
            .iter()
            .any(|existing| existing.kind == origin.kind && existing.name == origin.name)
        {
            // Locale fallback precedence is an explicit semantic replacement. Retain both
            // provenance records while making that replacement visible to IR validation.
            origin.is_override = true;
        }
        target.origins.push(origin);
    }
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
