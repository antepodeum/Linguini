mod artifacts;
mod decl;
mod deps;
mod emit;
mod expr;
mod formatters;
mod message;
mod messages;
mod names;
mod project;
mod semantic;
mod shared;
mod templates;
mod tree;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use linguini_cldr::{
    built_in_plural_rules, canonicalize_locale, compiled_currency_formatting,
    compiled_date_formatting, compiled_number_formatting,
    locale_fallback_chain as cldr_locale_fallback_chain,
};
use linguini_ir::{validate_ir, IrModule, IrReferenceError, ValidatedIr};

use self::emit::{
    emit_forms, emit_imports, emit_local_functions, emit_locale_enum_types, emit_messages,
    emit_schema_type_reexports, emit_variables,
};
use self::formatters::{formatter_requirements, plural_required};
use self::names::{escape_string, portable_path_component_error, safe_file_stem, safe_identifier};
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
    pub sources: Vec<TypeScriptLocaleSource>,
    pub cookie_name: String,
    pub cookie_path: String,
    pub cookie_domain: Option<String>,
    pub cookie_max_age: u64,
    pub cookie_same_site: String,
    pub cookie_secure: bool,
    pub cookie_http_only: bool,
    pub local_storage_key: String,
    pub prefix_default_locale: bool,
    pub base_path: String,
    pub redirect: bool,
    pub origin: Option<String>,
    pub exclude: Vec<String>,
    pub localize_links: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeScriptLocaleSource {
    Path,
    Cookie,
    LocalStorage,
    AcceptLanguage,
}

impl TypeScriptLocaleSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Cookie => "cookie",
            Self::LocalStorage => "local-storage",
            Self::AcceptLanguage => "accept-language",
        }
    }
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
            sources: vec![
                TypeScriptLocaleSource::Path,
                TypeScriptLocaleSource::Cookie,
                TypeScriptLocaleSource::AcceptLanguage,
            ],
            cookie_name: "LINGUINI_LOCALE".to_owned(),
            cookie_path: "/".to_owned(),
            cookie_domain: None,
            cookie_max_age: 60 * 60 * 24 * 365,
            cookie_same_site: "lax".to_owned(),
            cookie_secure: false,
            cookie_http_only: false,
            local_storage_key: "LINGUINI_LOCALE".to_owned(),
            prefix_default_locale: false,
            base_path: String::new(),
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
    EmptyLocaleSet,
    DuplicateLocale {
        locale: String,
        conflicts_with: String,
    },
    InvalidLocalePathComponent {
        locale: String,
        reason: &'static str,
    },
    InvalidNamespacePathComponent {
        namespace: String,
        reason: &'static str,
    },
    OutputPathCollision {
        path: String,
        conflicts_with: String,
    },
    MissingBaseLocale,
    UnknownBaseLocale {
        base_locale: String,
    },
    UnknownIncludedMessage {
        message: String,
    },
    UnknownLocale {
        locale: String,
    },
    UnknownMessage {
        message: String,
    },
    MissingMessageImplementation {
        locale: String,
        message: String,
    },
    MissingTextDirection {
        locale: String,
    },
    UnsupportedTextDirection {
        locale: String,
        direction: String,
    },
    InvalidIr {
        scope: String,
        errors: Vec<IrReferenceError>,
    },
    MissingPluralRules {
        locale: String,
    },
    MissingNumberFormatting {
        locale: String,
    },
    MissingCurrencyFormatting {
        locale: String,
    },
    MissingDateFormatting {
        locale: String,
    },
    MissingMessageSource {
        source_id: linguini_syntax::SourceId,
    },
    DuplicateMessageSource {
        source_id: linguini_syntax::SourceId,
    },
    InvalidMessageArtifactPath {
        locale: String,
        message: String,
        reason: &'static str,
    },
    InvalidSemanticArtifactPath {
        locale: String,
        kind: String,
        name: String,
        reason: &'static str,
    },
    SemanticDependencyCycle {
        locale: String,
        cycle: Vec<String>,
    },
    UnknownSemanticArtifact {
        locale: String,
        kind: String,
        name: String,
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
            Self::EmptyLocaleSet => {
                formatter.write_str("TypeScript project requires at least one locale")
            }
            Self::DuplicateLocale {
                locale,
                conflicts_with,
            } => write!(
                formatter,
                "locale `{locale}` conflicts with locale `{conflicts_with}` after case folding"
            ),
            Self::InvalidLocalePathComponent { locale, reason } => {
                write!(formatter, "locale `{locale}` is not a portable filename: {reason}")
            }
            Self::InvalidNamespacePathComponent { namespace, reason } => write!(
                formatter,
                "namespace `{namespace}` cannot be represented by a portable filename: {reason}"
            ),
            Self::OutputPathCollision {
                path,
                conflicts_with,
            } => write!(
                formatter,
                "generated output path `{path}` conflicts with `{conflicts_with}` on a case-insensitive filesystem"
            ),
            Self::MissingBaseLocale => {
                formatter.write_str("TypeScript project requires an explicit base locale")
            }
            Self::UnknownBaseLocale { base_locale } => write!(
                formatter,
                "configured base locale `{base_locale}` is not present in the project locale set"
            ),
            Self::UnknownIncludedMessage { message } => write!(
                formatter,
                "configured included message or namespace `{message}` is not present in the schema"
            ),
            Self::UnknownLocale { locale } => {
                write!(formatter, "configured locale `{locale}` is not present in the project")
            }
            Self::UnknownMessage { message } => {
                write!(formatter, "schema message `{message}` is not present in the project")
            }
            Self::MissingMessageImplementation { locale, message } => write!(
                formatter,
                "locale `{locale}` has no fallback implementation for schema message `{message}`"
            ),
            Self::MissingTextDirection { locale } => write!(
                formatter,
                "missing built-in CLDR text direction for configured locale `{locale}`"
            ),
            Self::UnsupportedTextDirection { locale, direction } => write!(
                formatter,
                "unsupported CLDR text direction `{direction}` for configured locale `{locale}`"
            ),
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
            Self::MissingNumberFormatting { locale } => write!(
                formatter,
                "missing required CLDR number formatting data for configured locale `{locale}`"
            ),
            Self::MissingCurrencyFormatting { locale } => write!(
                formatter,
                "missing required CLDR currency formatting data for configured locale `{locale}`"
            ),
            Self::MissingDateFormatting { locale } => write!(
                formatter,
                "missing required CLDR date formatting data for configured locale `{locale}`"
            ),
            Self::MissingMessageSource { source_id } => write!(
                formatter,
                "missing source record for message dependency source id `{}`",
                source_id.0
            ),
            Self::DuplicateMessageSource { source_id } => write!(
                formatter,
                "duplicate source record for message dependency source id `{}`",
                source_id.0
            ),
            Self::InvalidMessageArtifactPath {
                locale,
                message,
                reason,
            } => write!(
                formatter,
                "cannot derive portable bundler artifact path for message `{message}` and locale `{locale}`: {reason}"
            ),
            Self::InvalidSemanticArtifactPath { locale, kind, name, reason } => write!(
                formatter,
                "cannot derive portable semantic artifact path for {kind} `{name}` and locale `{locale}`: {reason}"
            ),
            Self::SemanticDependencyCycle { locale, cycle } => write!(
                formatter,
                "semantic ESM dependency cycle for locale `{locale}`: {}",
                cycle.join(" -> ")
            ),
            Self::UnknownSemanticArtifact { locale, kind, name } => write!(
                formatter,
                "semantic artifact {kind} `{name}` for locale `{locale}` is not selected by the project"
            ),
        }
    }
}

pub use artifacts::{TypeScriptLocaleRuntimeArtifact, TypeScriptMessageArtifact};
pub use message::{
    compile_typescript_bundler_message_artifact_module, compile_typescript_bundler_message_module,
    compile_typescript_message_module, CompiledTypeScriptMessageModule,
};
pub use semantic::{
    compile_typescript_bundler_semantic_module, CompiledTypeScriptSemanticModule,
    TypeScriptSemanticArtifact, TypeScriptSemanticImport, TypeScriptSemanticSymbolKind,
};

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
        validate_project_inputs(schema, locales, options)?;

        let empty_locale = IrModule::default();
        validate_codegen_ir(schema, &empty_locale, "schema")?;

        let locales = fallback_locale_modules(locales, options.base_locale.as_deref());
        for locale in &locales {
            validate_codegen_ir(
                schema,
                &locale.module,
                format!("locale `{}`", locale.locale),
            )?;
            validate_formatter_data(schema, &locale.module, &locale.locale)?;
        }

        Ok(Self {
            schema,
            locales,
            options: options.clone(),
        })
    }

    pub(crate) fn message_dependency_closure(
        &self,
        locale: &str,
        message: &str,
    ) -> Result<deps::MessageDependencyClosure, TypeScriptCodegenError> {
        deps::message_dependency_closure(self, locale, message)
    }

    /// Enumerates every effective message/locale leaf selected by this validated project.
    ///
    /// Returned artifact paths are deterministic, project-output-relative, forward-slash paths.
    /// Their encoding is injective under case folding and avoids platform-reserved names.
    pub fn message_artifacts(
        &self,
    ) -> Result<Vec<TypeScriptMessageArtifact>, TypeScriptCodegenError> {
        artifacts::message_artifacts(self)
    }

    /// Enumerates one shared generated-helper runtime for each effective locale.
    pub fn locale_runtime_artifacts(
        &self,
    ) -> Result<Vec<TypeScriptLocaleRuntimeArtifact>, TypeScriptCodegenError> {
        artifacts::locale_runtime_artifacts(self)
    }

    /// Enumerates deterministic one-binding semantic leaves required by selected messages.
    pub fn semantic_artifacts(
        &self,
    ) -> Result<Vec<TypeScriptSemanticArtifact>, TypeScriptCodegenError> {
        semantic::semantic_artifacts(self)
    }

    /// Locale identities whose fallback-composed modules passed project validation.
    pub fn effective_locales(&self) -> Vec<&str> {
        self.locales
            .iter()
            .map(|locale| locale.locale.as_str())
            .collect()
    }
}

fn validate_formatter_data(
    schema: &IrModule,
    locale: &IrModule,
    locale_name: &str,
) -> Result<(), TypeScriptCodegenError> {
    let requirements = formatter_requirements(schema, locale);
    if requirements.needs_number_data() && compiled_number_formatting(locale_name).is_none() {
        return Err(TypeScriptCodegenError::MissingNumberFormatting {
            locale: locale_name.to_owned(),
        });
    }
    if requirements.currency && compiled_currency_formatting(locale_name).is_none() {
        return Err(TypeScriptCodegenError::MissingCurrencyFormatting {
            locale: locale_name.to_owned(),
        });
    }
    if requirements.date && compiled_date_formatting(locale_name).is_none() {
        return Err(TypeScriptCodegenError::MissingDateFormatting {
            locale: locale_name.to_owned(),
        });
    }
    Ok(())
}

fn validate_project_inputs(
    schema: &IrModule,
    locales: &[TypeScriptLocaleModule],
    options: &TypeScriptProjectOptions,
) -> Result<(), TypeScriptCodegenError> {
    if locales.is_empty() {
        return Err(TypeScriptCodegenError::EmptyLocaleSet);
    }

    for locale in locales {
        if let Some(reason) = portable_path_component_error(&locale.locale) {
            return Err(TypeScriptCodegenError::InvalidLocalePathComponent {
                locale: locale.locale.clone(),
                reason,
            });
        }
    }

    for (index, locale) in locales.iter().enumerate() {
        if let Some(conflict) = locales[..index]
            .iter()
            .find(|candidate| candidate.locale.eq_ignore_ascii_case(&locale.locale))
        {
            return Err(TypeScriptCodegenError::DuplicateLocale {
                locale: locale.locale.clone(),
                conflicts_with: conflict.locale.clone(),
            });
        }
    }

    let base_locale = options
        .base_locale
        .as_deref()
        .ok_or(TypeScriptCodegenError::MissingBaseLocale)?;
    if !locales.iter().any(|locale| locale.locale == base_locale) {
        return Err(TypeScriptCodegenError::UnknownBaseLocale {
            base_locale: base_locale.to_owned(),
        });
    }

    for selected in &options.included_messages {
        let is_known = schema.messages.iter().any(|message| {
            message.name == *selected
                || message
                    .name
                    .strip_prefix(selected)
                    .is_some_and(|rest| rest.starts_with('.'))
        }) || schema.groups.iter().any(|group| group.name == *selected);
        if !is_known {
            return Err(TypeScriptCodegenError::UnknownIncludedMessage {
                message: selected.clone(),
            });
        }
    }

    for locale in locales {
        validate_text_direction(&locale.locale)?;
    }

    validate_namespace_output_paths(schema, locales, options)?;

    Ok(())
}

fn validate_namespace_output_paths(
    schema: &IrModule,
    locales: &[TypeScriptLocaleModule],
    options: &TypeScriptProjectOptions,
) -> Result<(), TypeScriptCodegenError> {
    let mut output_stems = BTreeMap::<String, (String, String)>::new();
    for namespace in top_level_namespaces(schema)
        .into_iter()
        .filter(|namespace| {
            !options.tree_shaking
                || options.included_messages.is_empty()
                || options.included_messages.iter().any(|selected| {
                    selected == namespace
                        || selected
                            .strip_prefix(namespace)
                            .is_some_and(|rest| rest.starts_with('.'))
                        || namespace
                            .strip_prefix(selected)
                            .is_some_and(|rest| rest.starts_with('.'))
                })
        })
    {
        let stem = safe_file_stem(&namespace);
        if let Some(reason) = portable_path_component_error(&stem) {
            return Err(TypeScriptCodegenError::InvalidNamespacePathComponent {
                namespace,
                reason,
            });
        }
        let folded = stem.to_ascii_lowercase();
        if folded == "_runtime" {
            let locale = &locales[0].locale;
            return Err(TypeScriptCodegenError::OutputPathCollision {
                path: format!("locales/{locale}/{stem}.ts"),
                conflicts_with: format!("locales/{locale}/_runtime.ts (locale runtime)"),
            });
        }
        if let Some((conflicting_namespace, conflicting_stem)) = output_stems.get(&folded) {
            let locale = &locales[0].locale;
            return Err(TypeScriptCodegenError::OutputPathCollision {
                path: format!("locales/{locale}/{stem}.ts"),
                conflicts_with: format!(
                    "locales/{locale}/{conflicting_stem}.ts (namespace `{conflicting_namespace}`)"
                ),
            });
        }
        output_stems.insert(folded, (namespace, stem));
    }
    Ok(())
}

fn validate_text_direction(locale: &str) -> Result<(), TypeScriptCodegenError> {
    match linguini_cldr::built_in_text_direction(locale) {
        Some("ltr" | "rtl") => Ok(()),
        Some(direction) => Err(TypeScriptCodegenError::UnsupportedTextDirection {
            locale: locale.to_owned(),
            direction: direction.to_owned(),
        }),
        None => Err(TypeScriptCodegenError::MissingTextDirection {
            locale: locale.to_owned(),
        }),
    }
}

pub fn generate_typescript_project_files(
    project: &ValidatedTypeScriptProject<'_>,
) -> Result<Vec<TypeScriptGeneratedFile>, TypeScriptCodegenError> {
    let schema = project.schema;
    let locales = &project.locales;
    let options = &project.options;
    let messages_schema = if options.tree_shaking && !options.included_messages.is_empty() {
        visible_schema(
            schema,
            &TypeScriptOptions {
                included_messages: options.included_messages.clone(),
                ..TypeScriptOptions::default()
            },
        )
    } else {
        schema.clone()
    };
    let mut files = vec![TypeScriptGeneratedFile {
        path: "shared.ts".to_owned(),
        contents: generate_shared_module(schema),
    }];

    // Keep the public namespace contract in its own schema-owned module. This is emitted even
    // when declaration output is disabled so source-only consumers and the generated runtime
    // always resolve the same recursive `LinguiniMessages` type.
    files.push(TypeScriptGeneratedFile {
        path: "messages.ts".to_owned(),
        contents: messages::generate_messages_module(&messages_schema),
    });

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
        files.push(TypeScriptGeneratedFile {
            path: "messages.d.ts".to_owned(),
            contents: messages::generate_messages_module(&messages_schema),
        });
    }

    for locale in locales {
        let locale_options = project_locale_options(&locale.locale, options)?;
        let visible_schema = visible_schema(schema, &locale_options);
        let visible_locale = locale_module_for_schema(&locale.module, &visible_schema);
        files.push(TypeScriptGeneratedFile {
            path: format!("locales/{}/_runtime.ts", locale.locale),
            contents: generate_locale_runtime(&visible_schema, &visible_locale, &locale_options),
        });
        let namespaces = top_level_namespaces(&visible_schema);
        for namespace in &namespaces {
            let namespace_file_stem = safe_file_stem(namespace);
            let namespace_schema = namespace_module(&visible_schema, namespace);
            let namespace_locale = namespace_module(&visible_locale, namespace);
            let validated = validate_codegen_ir(
                &namespace_schema,
                &namespace_locale,
                format!("locale `{}` namespace `{namespace}`", locale.locale),
            )?;
            files.push(TypeScriptGeneratedFile {
                path: format!("locales/{}/{namespace_file_stem}.ts", locale.locale),
                contents: generate_typescript_module_with_shared_import(
                    &validated,
                    &locale_options,
                    "../../shared",
                    "./_runtime",
                    Some(namespace),
                ),
            });
            if options.declaration {
                files.push(TypeScriptGeneratedFile {
                    path: format!("locales/{}/{namespace_file_stem}.d.ts", locale.locale),
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
        path: "locale.ts".to_owned(),
        contents: project::generate_project_locale(locales, options.base_locale.as_deref()),
    });
    if options.declaration {
        files.push(TypeScriptGeneratedFile {
            path: "locale.d.ts".to_owned(),
            contents: project::generate_project_locale_declaration(
                locales,
                options.base_locale.as_deref(),
            ),
        });
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
        let sveltekit = options.framework == Some(TypeScriptFramework::SvelteKit);
        if let Some(web) = options.web.as_ref() {
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
            files.push(TypeScriptGeneratedFile {
                path: "svelte-effects.svelte.ts".to_owned(),
                contents: project::generate_project_svelte_effects_module(web, sveltekit),
            });
            if options.declaration {
                files.push(TypeScriptGeneratedFile {
                    path: "svelte-effects.svelte.d.ts".to_owned(),
                    contents: project::generate_project_svelte_effects_declaration(),
                });
            }
            files.push(TypeScriptGeneratedFile {
                path: "svelte-control.ts".to_owned(),
                contents: project::generate_project_svelte_control_module(sveltekit),
            });
            if options.declaration {
                files.push(TypeScriptGeneratedFile {
                    path: "svelte-control.d.ts".to_owned(),
                    contents: project::generate_project_svelte_control_declaration(sveltekit),
                });
            }
        }
        files.push(TypeScriptGeneratedFile {
            path: "svelte-locale.svelte.ts".to_owned(),
            contents: project::generate_project_svelte_locale_module(
                options.web.is_some(),
                sveltekit,
            ),
        });
        if options.declaration {
            files.push(TypeScriptGeneratedFile {
                path: "svelte-locale.svelte.d.ts".to_owned(),
                contents: project::generate_project_svelte_locale_declaration(
                    options.web.is_some(),
                    sveltekit,
                ),
            });
        }
        files.push(TypeScriptGeneratedFile {
            path: "svelte.ts".to_owned(),
            contents: project::generate_project_svelte_module(options.web.as_ref(), sveltekit),
        });
        if options.declaration {
            files.push(TypeScriptGeneratedFile {
                path: "svelte.d.ts".to_owned(),
                contents: project::generate_project_svelte_declaration(
                    options.web.is_some(),
                    sveltekit,
                ),
            });
        }
    }

    if let Some(web) = options.web.as_ref().filter(|_| {
        options
            .framework
            .is_some_and(TypeScriptFramework::needs_sveltekit_module)
    }) {
        files.push(TypeScriptGeneratedFile {
            path: "sveltekit-control.ts".to_owned(),
            contents: project::generate_project_sveltekit_control_module(web),
        });
        if options.declaration {
            files.push(TypeScriptGeneratedFile {
                path: "sveltekit-control.d.ts".to_owned(),
                contents: project::generate_project_sveltekit_control_declaration(),
            });
        }
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

    validate_generated_output_collisions(&files)?;
    Ok(files)
}

fn validate_generated_output_collisions(
    files: &[TypeScriptGeneratedFile],
) -> Result<(), TypeScriptCodegenError> {
    let mut paths = BTreeMap::<String, &str>::new();
    for file in files {
        let folded = file.path.to_ascii_lowercase();
        if let Some(conflict) = paths.insert(folded, &file.path) {
            return Err(TypeScriptCodegenError::OutputPathCollision {
                path: file.path.clone(),
                conflicts_with: conflict.to_owned(),
            });
        }
    }
    Ok(())
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
        let identifier = safe_identifier(namespace);
        let file_stem = safe_file_stem(namespace);
        output.push_str(&format!(
            "import {{ {} }} from \"./{}/{}\";\n",
            identifier,
            escape_string(&options.locale),
            escape_string(&file_stem)
        ));
    }
    emit_imports(schema, locale, options, "../shared", &mut output);
    emit_locale_runtime_imports(
        schema,
        locale,
        options,
        &format!("./{}/_runtime", escape_string(&options.locale)),
        &mut output,
    );
    if !namespaces.is_empty() {
        output.push('\n');
    }
    emit_schema_type_reexports(schema, "../shared", &mut output);
    emit_locale_enum_types(schema, locale, &mut output);
    for namespace in namespaces {
        output.push_str(&format!("export {{ {} }};\n\n", safe_identifier(namespace)));
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
    runtime_import_path: &str,
    namespace_alias: Option<&str>,
) -> String {
    let schema = ir.schema();
    let locale = ir.locale();
    let mut output = String::new();
    emit_imports(schema, locale, options, shared_import_path, &mut output);
    emit_locale_runtime_imports(schema, locale, options, runtime_import_path, &mut output);
    emit_schema_type_reexports(schema, shared_import_path, &mut output);
    emit_locale_enum_types(schema, locale, &mut output);
    emit_variables(locale, options, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, options, &mut output);
    let exports = emit_messages(schema, locale, options, &mut output);
    emit_locale_default(&exports, &[], &mut output);
    if let Some(namespace_alias) = namespace_alias {
        let identifier = safe_identifier(namespace_alias);
        let alias_is_exported = exports
            .top_level
            .iter()
            .chain(exports.groups.iter())
            .any(|export| export == &identifier);
        if !alias_is_exported {
            output.push_str(&format!("\nexport const {identifier} = lgl;\n"));
        }
    }
    output
}

fn generate_locale_runtime(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
) -> String {
    let mut output = String::new();
    let requirements = formatter_requirements(schema, locale);
    if requirements.any() {
        output.push_str(&expr::formatter_data_declaration(
            &options.locale,
            requirements,
        ));
        for name in requirements.helper_names() {
            output = output.replacen(
                &format!("function {name}("),
                &format!("export function {name}("),
                1,
            );
        }
    }
    if plural_required(schema, locale) {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(
            options
                .plural_source
                .as_deref()
                .expect("project locale options always include plural source"),
        );
    }
    output
}

fn emit_locale_runtime_imports(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    runtime_import_path: &str,
    output: &mut String,
) {
    let helpers = runtime_helper_names(schema, locale, options);
    if !helpers.is_empty() {
        output.push_str(&format!(
            "import {{ {} }} from \"{}\";\n",
            helpers.join(", "),
            escape_string(runtime_import_path)
        ));
    }
}

fn runtime_helper_names(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
) -> Vec<String> {
    let mut helpers = formatter_requirements(schema, locale)
        .helper_names()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if plural_required(schema, locale) {
        helpers.push(options.plural_function.clone());
    }
    helpers
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
    let retained_message_names = visible
        .messages
        .iter()
        .map(|message| message.name.as_str())
        .collect::<Vec<_>>();
    visible.groups.retain(|group| {
        options.included_messages.iter().any(|selected| {
            is_path_or_descendant(&group.name, selected)
                || is_path_or_descendant(selected, &group.name)
        }) || retained_message_names
            .iter()
            .any(|message| is_path_or_descendant(message, &group.name))
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
    let schema_groups = schema
        .groups
        .iter()
        .map(|group| group.name.as_str())
        .collect::<BTreeSet<_>>();
    visible
        .groups
        .retain(|group| schema_groups.contains(group.name.as_str()));
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
    output.groups = module
        .groups
        .iter()
        .filter(|group| group.name == namespace || group.name.starts_with(&prefix))
        .cloned()
        .collect();
    output
}

fn root_module(module: &IrModule) -> IrModule {
    let mut output = module.clone();
    output
        .messages
        .retain(|message| !message.name.contains('.'));
    output.groups.retain(|group| !group.name.contains('.'));
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

pub(crate) fn locale_fallback_chain(
    locales: &[TypeScriptLocaleModule],
    locale: &str,
    base_locale: Option<&str>,
) -> Vec<String> {
    let mut chain = Vec::new();
    let tags = cldr_locale_fallback_chain(locale).unwrap_or_else(|_| vec![locale.to_owned()]);
    for (index, tag) in tags.into_iter().enumerate() {
        let raw_match = locales.iter().find(|entry| {
            if index == 0 {
                entry.locale.eq_ignore_ascii_case(locale)
            } else {
                entry.locale.eq_ignore_ascii_case(&tag)
            }
        });
        let canonical_match = || {
            locales.iter().find(|entry| {
                canonicalize_locale(&entry.locale).map_or_else(
                    |_| entry.locale.eq_ignore_ascii_case(&tag),
                    |canonical| canonical.eq_ignore_ascii_case(&tag),
                )
            })
        };
        if let Some(exact) = raw_match
            .or_else(canonical_match)
            .map(|entry| entry.locale.clone())
        {
            if !chain.contains(&exact) {
                chain.push(exact);
            }
        }
    }
    if let Some(base) = base_locale {
        if !chain.iter().any(|entry| entry == base) {
            chain.push(base.to_owned());
        }
    }
    chain
}

fn merge_locale_module(target: &mut IrModule, source: &IrModule) {
    merge_named_items(&mut target.enums, &source.enums, |item| &item.name);
    merge_named_items(&mut target.type_aliases, &source.type_aliases, |item| {
        &item.name
    });
    merge_named_items(&mut target.groups, &source.groups, |group| &group.name);
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
    safe_identifier(&format!("plural{}", pascal_identifier(locale)))
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
    for name in exports.top_level.iter().chain(exports.groups.iter()) {
        output.push_str(&format!("  {name},\n"));
    }
    for namespace in namespaces {
        output.push_str(&format!("  {},\n", safe_identifier(namespace)));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export default lgl;\n");
}

fn is_path_or_descendant(path: &str, parent: &str) -> bool {
    path == parent
        || path
            .strip_prefix(parent)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::{
        fallback_locale_module, locale_module_for_schema, namespace_module, root_module,
        top_level_namespaces, validate_project_inputs, visible_schema, TypeScriptCodegenError,
        TypeScriptLocaleModule, TypeScriptOptions, TypeScriptProjectOptions,
    };
    use linguini_ir::{lower_locale, lower_schema, IrModule};
    use linguini_syntax::{parse_locale, parse_schema};

    #[test]
    fn tree_shaking_keeps_selected_group_ancestors_only() {
        let schema = lower_schema(
            &parse_schema("top { keep { title } drop { hidden } }\nunrelated { other }\n")
                .expect("schema parses"),
        );
        let options = TypeScriptOptions {
            included_messages: vec!["top.keep.title".to_owned()],
            ..TypeScriptOptions::default()
        };

        let visible = visible_schema(&schema, &options);

        assert_eq!(
            visible
                .messages
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["top.keep.title"]
        );
        assert_eq!(
            visible
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["top", "top.keep"]
        );
    }

    #[test]
    fn ancestor_group_selection_keeps_declared_empty_descendants() {
        let schema = lower_schema(
            &parse_schema("top { empty {} nested { title } }\n").expect("schema parses"),
        );
        let options = TypeScriptOptions {
            included_messages: vec!["top".to_owned()],
            ..TypeScriptOptions::default()
        };

        let visible = visible_schema(&schema, &options);

        assert_eq!(
            visible
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["top", "top.empty", "top.nested"]
        );
    }

    #[test]
    fn direct_empty_group_selection_is_valid_and_unknown_path_stays_rejected() {
        let schema = lower_schema(&parse_schema("top { empty {} }\n").expect("schema parses"));
        let locales = [TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: IrModule::default(),
        }];
        let options = TypeScriptProjectOptions {
            tree_shaking: true,
            included_messages: vec!["top.empty".to_owned()],
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };

        validate_project_inputs(&schema, &locales, &options)
            .expect("declared empty group is valid selection");

        let unknown = TypeScriptProjectOptions {
            included_messages: vec!["top.unknown".to_owned()],
            ..options
        };
        assert!(matches!(
            validate_project_inputs(&schema, &locales, &unknown),
            Err(TypeScriptCodegenError::UnknownIncludedMessage { message })
                if message == "top.unknown"
        ));
    }

    #[test]
    fn locale_runtime_path_is_reserved_from_namespace_outputs() {
        let schema = lower_schema(&parse_schema("_runtime { title }\n").expect("schema parses"));
        let locales = [TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(
                &parse_locale("_runtime { title = Title }\n").expect("locale parses"),
            ),
        }];
        let error = validate_project_inputs(
            &schema,
            &locales,
            &TypeScriptProjectOptions {
                base_locale: Some("en".to_owned()),
                ..TypeScriptProjectOptions::default()
            },
        )
        .expect_err("runtime namespace collision");
        assert!(matches!(
            error,
            TypeScriptCodegenError::OutputPathCollision { path, conflicts_with }
                if path == "locales/en/_runtime.ts" && conflicts_with.contains("locale runtime")
        ));
    }

    #[test]
    fn namespace_and_root_projections_keep_only_relevant_groups() {
        let module = lower_schema(
            &parse_schema("alpha { nested { title } }\nbeta { title }\nroot\n")
                .expect("schema parses"),
        );

        assert_eq!(top_level_namespaces(&module), ["alpha", "beta"]);
        let alpha = namespace_module(&module, "alpha");
        assert_eq!(
            alpha
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "alpha.nested"]
        );
        assert_eq!(
            root_module(&module)
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "beta"]
        );
    }

    #[test]
    fn fallback_group_metadata_follows_locale_precedence() {
        let base = TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(
                &parse_locale(
                    "/// Base section\nsection {\n  title = Base\n  nested {\n    child = Child\n  }\n}\n",
                )
                .expect("base locale parses"),
            ),
        };
        let regional = TypeScriptLocaleModule {
            locale: "en-US".to_owned(),
            module: lower_locale(
                &parse_locale("/// Regional section\nsection { title = Regional }\n")
                    .expect("regional locale parses"),
            ),
        };

        let merged = fallback_locale_module(&[base, regional], "en-US", Some("en"));

        assert_eq!(merged.groups.len(), 2);
        assert_eq!(merged.groups[0].name, "section");
        assert_eq!(merged.groups[0].docs, ["Regional section"]);
        assert_eq!(merged.groups[1].name, "section.nested");
        assert_eq!(
            merged
                .messages
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["section.title", "section.nested.child"]
        );
        assert!(merged
            .origins
            .iter()
            .any(|origin| origin.name == "section" && origin.is_override));
    }

    #[test]
    fn locale_projection_discards_groups_absent_from_schema() {
        let schema = lower_schema(&parse_schema("keep { title }\n").expect("schema parses"));
        let locale = lower_locale(
            &parse_locale("keep { title = Keep }\ndrop { title = Drop }\n").expect("locale parses"),
        );

        let projected = locale_module_for_schema(&locale, &schema);

        assert_eq!(
            projected
                .groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["keep"]
        );
    }
}
