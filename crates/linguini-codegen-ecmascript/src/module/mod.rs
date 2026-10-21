mod artifacts;
mod decl;
mod deps;
mod emit;
mod entry;
mod expr;
mod formatter_codegen;
mod formatter_helpers;
mod formatters;
mod globals;
mod locale;
mod message;
mod messages;
mod names;
mod project_output;
mod project_web;
mod provenance;
mod runtime;
mod semantic;
mod shared;
mod shared_runtime;
mod signature;
#[cfg(test)]
mod test_templates;
mod tree;
mod type_model;
mod type_tree;
mod web_accept_language;
mod web_cookie;
mod web_facade;
mod web_link_transform;
mod web_local_storage;
pub(crate) mod web_options;
mod web_path;
mod web_routes;
mod web_runtime_links;
mod web_server_cookie;
mod web_switch_route;
mod web_types;

pub use web_options::web_options_literal as render_web_options;
pub use web_server_cookie::generate_web_server_cookie_declaration;
pub use web_switch_route::generate_web_switch_route_declaration;
pub use web_types::generate_web_declaration;

pub(crate) use crate::ecmascript::EcmaModuleKind;
use formatters::formatter_helper_names;

pub use type_model::{render_jsdoc_type, render_typescript_type, TypeModel};
pub use web_server_cookie::compile_web_server_cookie_module;
pub use web_switch_route::compile_web_switch_route_module;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use linguini_cldr::{
    built_in_plural_rules, canonicalize_locale, compiled_currency_formatting,
    compiled_date_formatting, compiled_number_formatting,
    locale_fallback_chain as cldr_locale_fallback_chain,
};
use linguini_ir::{
    validate_typed_ir, IrEnum, IrForm, IrFunction, IrGroup, IrMessage, IrModule, IrModuleBuilder,
    IrOrigin, IrReferenceError, IrSymbolConflict, IrTypeAlias, IrVariable, LocaleIr, SchemaIr,
};

use self::formatters::{formatter_requirements, plural_required};
use self::names::{
    form_binding_name, portable_path_component_error, safe_file_stem, safe_identifier,
};
use self::shared::generate_shared_module;
use crate::ecmascript::{EcmaImport, EcmaModuleOutput, EcmaNamedImport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaOptions {
    pub locale: String,
    pub plural_function: String,
    pub plural_import: Option<String>,
    pub plural_rules: Option<linguini_cldr::PluralRules>,
    pub included_messages: Vec<String>,
}

impl Default for EcmaOptions {
    fn default() -> Self {
        Self {
            locale: "ru".to_owned(),
            plural_function: "plural".to_owned(),
            plural_import: Some("./plurals".to_owned()),
            plural_rules: None,
            included_messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaLocaleModule {
    pub locale: String,
    pub module: LocaleIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaProjectOptions {
    pub declaration: bool,
    pub gitignore: bool,
    pub tree_shaking: bool,
    pub included_messages: Vec<String>,
    pub base_locale: Option<String>,
    pub web: Option<EcmaWebOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaWebOptions {
    pub sources: Vec<EcmaLocaleSource>,
    pub locale_switch: EcmaLocaleSwitchPlan,
    pub cookie_name: String,
    /// Explicit cookie path, or `None` to derive it from the framework base path.
    pub cookie_path: Option<String>,
    pub cookie_domain: Option<String>,
    pub cookie_max_age: u64,
    pub cookie_same_site: String,
    /// Explicit secure policy, or `None` to derive it from the request/browser protocol.
    pub cookie_secure: Option<bool>,
    pub cookie_http_only: bool,
    pub local_storage_key: String,
    /// Controls whether generated URLs carry a locale path segment.
    pub locale_prefix: EcmaLocalePrefixMode,
    pub canonical_redirect: bool,
    pub exclude: Vec<String>,
    /// Link handling is a closed capability rather than an on/off flag.
    pub link_mode: EcmaLinkMode,
    pub switch_route: Option<EcmaWebSwitchRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaWebSwitchRoute {
    pub path: String,
    pub return_query: String,
    pub status: u16,
}

/// Closed locale-prefix policy lowered into generated ECMAScript projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EcmaLocalePrefixMode {
    Always,
    #[default]
    ExceptDefault,
    Never,
}

impl EcmaLocalePrefixMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::ExceptDefault => "except-default",
            Self::Never => "never",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EcmaLinkMode {
    Transform,
    #[default]
    Runtime,
    Manual,
}

impl EcmaLinkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transform => "transform",
            Self::Runtime => "runtime",
            Self::Manual => "manual",
        }
    }
}

/// Closed capability view used by the generated web modules.
///
/// `EcmaWebOptions` carries the lowered structured policy and serialization
/// details, while this view is what source and browser emitters inspect. Source
/// order is fixed at lowering time; generated runtimes never dispatch over an
/// open-ended strategy enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaWebFeatures {
    pub sources: Vec<EcmaLocaleSource>,
    pub has_cookie: bool,
    pub has_local_storage: bool,
    pub has_path: bool,
    pub has_accept_language: bool,
    pub locale_switch: EcmaLocaleSwitchPlan,
    pub locale_prefix: EcmaLocalePrefixMode,
    pub link_mode: EcmaLinkMode,
    pub switch_route: Option<EcmaWebSwitchRoute>,
}

impl EcmaWebOptions {
    pub fn features(&self) -> EcmaWebFeatures {
        EcmaWebFeatures {
            sources: self.sources.clone(),
            has_cookie: self.sources.contains(&EcmaLocaleSource::Cookie),
            has_local_storage: self.sources.contains(&EcmaLocaleSource::LocalStorage),
            has_path: self.sources.contains(&EcmaLocaleSource::Path),
            has_accept_language: self.sources.contains(&EcmaLocaleSource::AcceptLanguage),
            locale_switch: self.locale_switch,
            locale_prefix: self.locale_prefix,
            link_mode: self.link_mode,
            switch_route: self.switch_route.clone(),
        }
    }
}

/// Generated browser transition capabilities lowered from the validated web config.
///
/// `sources` remains the locale-resolution order. This plan is the only generated policy
/// surface used when a browser control changes locale and persists that choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EcmaLocaleSwitchPlan {
    pub writes_path: bool,
    pub writes_cookie: bool,
    pub writes_local_storage: bool,
}

impl Default for EcmaLocaleSwitchPlan {
    fn default() -> Self {
        Self {
            writes_path: true,
            writes_cookie: true,
            writes_local_storage: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmaLocaleSource {
    Path,
    Cookie,
    LocalStorage,
    AcceptLanguage,
}

impl EcmaLocaleSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Cookie => "cookie",
            Self::LocalStorage => "local-storage",
            Self::AcceptLanguage => "accept-language",
        }
    }
}

impl Default for EcmaProjectOptions {
    fn default() -> Self {
        Self {
            declaration: true,
            gitignore: true,
            tree_shaking: false,
            included_messages: Vec::new(),
            base_locale: None,
            web: None,
        }
    }
}

impl Default for EcmaWebOptions {
    fn default() -> Self {
        Self {
            sources: vec![
                EcmaLocaleSource::Path,
                EcmaLocaleSource::Cookie,
                EcmaLocaleSource::AcceptLanguage,
            ],
            locale_switch: EcmaLocaleSwitchPlan::default(),
            cookie_name: "LINGUINI_LOCALE".to_owned(),
            cookie_path: None,
            cookie_domain: None,
            cookie_max_age: 60 * 60 * 24 * 365,
            cookie_same_site: "lax".to_owned(),
            cookie_secure: None,
            cookie_http_only: false,
            local_storage_key: "LINGUINI_LOCALE".to_owned(),
            locale_prefix: EcmaLocalePrefixMode::ExceptDefault,
            canonical_redirect: true,
            exclude: Vec::new(),
            link_mode: EcmaLinkMode::Runtime,
            switch_route: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaGeneratedFile {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EcmaCodegenError {
    EmptyLocaleSet,
    DuplicateWebSource {
        source: String,
    },
    WebLocaleSwitchPlanMismatch {
        source: &'static str,
        expected: bool,
        actual: bool,
    },
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
    UnknownLocaleNamespace {
        locale: String,
        namespace: String,
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

impl EcmaCodegenError {
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

impl fmt::Display for EcmaCodegenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLocaleSet => {
                formatter.write_str("ECMAScript project requires at least one locale")
            }
            Self::DuplicateWebSource { source } => {
                write!(formatter, "generated web features contain duplicate locale source `{source}`")
            }
            Self::WebLocaleSwitchPlanMismatch {
                source,
                expected,
                actual,
            } => write!(
                formatter,
                "generated locale switch plan for source `{source}` is {actual}, expected {expected}"
            ),
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
                formatter.write_str("ECMAScript project requires an explicit base locale")
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
            Self::UnknownLocaleNamespace { locale, namespace } => write!(
                formatter,
                "locale namespace artifact `{namespace}` is not selected for locale `{locale}`"
            ),
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

pub use artifacts::{
    EcmaLocaleArtifact, EcmaLocaleArtifactKind, EcmaLocaleGlobalsArtifact,
    EcmaLocaleRuntimeArtifact, EcmaMessageArtifact, EcmaProjectArtifact, EcmaProjectArtifactKind,
};
pub use entry::{compile_project_artifact_module, CompiledEcmaProjectModule};
pub use globals::{compile_locale_globals_artifact_module, CompiledEcmaLocaleGlobalsModule};
pub use locale::{compile_locale_artifact_module, CompiledEcmaLocaleModule};
pub use message::{
    compile_bundler_message_artifact_module, compile_bundler_message_module,
    compile_message_module, CompiledEcmaMessageModule,
};
pub use project_output::compile_ecmascript_project_files;
pub use runtime::{compile_locale_runtime_artifact_module, CompiledEcmaLocaleRuntimeModule};
pub use semantic::{
    compile_bundler_semantic_module, CompiledEcmaSemanticModule, EcmaSemanticArtifact,
    EcmaSemanticImport, EcmaSemanticSymbolKind,
};
pub use web_accept_language::compile_web_accept_language_module;
pub use web_cookie::compile_web_cookie_module;
pub use web_facade::compile_web_module;
pub use web_link_transform::compile_web_link_transform_module;
pub use web_local_storage::compile_web_local_storage_module;
pub use web_path::compile_web_path_module;
pub use web_routes::compile_web_routes_module;
pub use web_runtime_links::compile_web_runtime_links_module;

impl std::error::Error for EcmaCodegenError {}

/// Validated input for project-level ECMAScript generation.
///
/// Construction validates the schema and every fallback-composed locale module. Its private
/// fields prevent production callers from bypassing the IR validation boundary.
#[derive(Debug)]
pub struct ValidatedEcmaProject<'a> {
    schema: &'a SchemaIr,
    locales: Vec<EcmaLocaleModule>,
    options: EcmaProjectOptions,
}

impl<'a> ValidatedEcmaProject<'a> {
    pub fn try_new(
        schema: &'a SchemaIr,
        locales: &[EcmaLocaleModule],
        options: &EcmaProjectOptions,
    ) -> Result<Self, EcmaCodegenError> {
        validate_project_inputs(schema, locales, options)?;

        let empty_locale = LocaleIr::default();
        validate_typed_codegen_ir(schema, &empty_locale, "schema")?;

        let locales = fallback_locale_modules(locales, options.base_locale.as_deref());
        for locale in &locales {
            validate_typed_codegen_ir(
                schema,
                &locale.module,
                format!("locale `{}`", locale.locale),
            )?;
            validate_formatter_data(schema, &locale.module, &locale.locale)?;
        }

        let visible_schema = if options.tree_shaking && !options.included_messages.is_empty() {
            visible_schema(
                schema,
                &EcmaOptions {
                    included_messages: options.included_messages.clone(),
                    ..EcmaOptions::default()
                },
            )
        } else {
            schema.as_module().clone()
        };
        for locale in &locales {
            for message in visible_schema.messages() {
                if !locale
                    .module
                    .messages()
                    .iter()
                    .any(|implementation| implementation.name == message.name)
                {
                    return Err(EcmaCodegenError::MissingMessageImplementation {
                        locale: locale.locale.clone(),
                        message: message.name.clone(),
                    });
                }
            }
        }

        let locales = locales
            .into_iter()
            .map(|mut locale| {
                locale.module = LocaleIr::try_from_module(
                    linguini_codegen::lower_inline_functions(schema, &locale.module),
                )
                .expect("inline lowering preserves locale IR ownership");
                locale
            })
            .collect();

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
    ) -> Result<deps::MessageDependencyClosure, EcmaCodegenError> {
        deps::message_dependency_closure(self, locale, message)
    }

    /// Enumerates every effective message/locale leaf selected by this validated project.
    ///
    /// Returned artifact paths are deterministic, project-output-relative, forward-slash paths.
    /// Their encoding is injective under case folding and avoids platform-reserved names.
    pub fn message_artifacts(&self) -> Result<Vec<EcmaMessageArtifact>, EcmaCodegenError> {
        artifacts::message_artifacts(self)
    }

    /// Enumerates one shared generated-helper runtime for each effective locale.
    pub fn locale_runtime_artifacts(
        &self,
    ) -> Result<Vec<EcmaLocaleRuntimeArtifact>, EcmaCodegenError> {
        artifacts::locale_runtime_artifacts(self)
    }

    /// Enumerates locale namespace modules followed by each locale barrel.
    pub fn locale_artifacts(&self) -> Result<Vec<EcmaLocaleArtifact>, EcmaCodegenError> {
        artifacts::locale_artifacts(self)
    }

    /// Enumerates one aggregate locale-global artifact for each locale that owns global symbols.
    pub fn locale_globals_artifacts(
        &self,
    ) -> Result<Vec<EcmaLocaleGlobalsArtifact>, EcmaCodegenError> {
        artifacts::locale_globals_artifacts(self)
    }

    /// Enumerates the project locale metadata module followed by its public index.
    pub fn project_artifacts(&self) -> Vec<EcmaProjectArtifact> {
        artifacts::project_artifacts()
    }

    /// Enumerates deterministic one-binding semantic leaves required by selected messages.
    pub fn semantic_artifacts(&self) -> Result<Vec<EcmaSemanticArtifact>, EcmaCodegenError> {
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
) -> Result<(), EcmaCodegenError> {
    let requirements = formatter_requirements(schema, locale);
    if requirements.needs_number_data() && compiled_number_formatting(locale_name).is_none() {
        return Err(EcmaCodegenError::MissingNumberFormatting {
            locale: locale_name.to_owned(),
        });
    }
    if requirements.currency && compiled_currency_formatting(locale_name).is_none() {
        return Err(EcmaCodegenError::MissingCurrencyFormatting {
            locale: locale_name.to_owned(),
        });
    }
    if requirements.date && compiled_date_formatting(locale_name).is_none() {
        return Err(EcmaCodegenError::MissingDateFormatting {
            locale: locale_name.to_owned(),
        });
    }
    Ok(())
}

fn validate_project_inputs(
    schema: &IrModule,
    locales: &[EcmaLocaleModule],
    options: &EcmaProjectOptions,
) -> Result<(), EcmaCodegenError> {
    if let Some(web) = options.web.as_ref() {
        validate_web_options(web)?;
    }
    if locales.is_empty() {
        return Err(EcmaCodegenError::EmptyLocaleSet);
    }

    for locale in locales {
        if let Some(reason) = portable_path_component_error(&locale.locale) {
            return Err(EcmaCodegenError::InvalidLocalePathComponent {
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
            return Err(EcmaCodegenError::DuplicateLocale {
                locale: locale.locale.clone(),
                conflicts_with: conflict.locale.clone(),
            });
        }
    }

    let base_locale = options
        .base_locale
        .as_deref()
        .ok_or(EcmaCodegenError::MissingBaseLocale)?;
    if !locales.iter().any(|locale| locale.locale == base_locale) {
        return Err(EcmaCodegenError::UnknownBaseLocale {
            base_locale: base_locale.to_owned(),
        });
    }

    for selected in &options.included_messages {
        let is_known = schema.messages().iter().any(|message| {
            message.name == *selected
                || message
                    .name
                    .strip_prefix(selected)
                    .is_some_and(|rest| rest.starts_with('.'))
        }) || schema.groups().iter().any(|group| group.name == *selected);
        if !is_known {
            return Err(EcmaCodegenError::UnknownIncludedMessage {
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

pub fn validate_web_options(options: &EcmaWebOptions) -> Result<(), EcmaCodegenError> {
    for (index, source) in options.sources.iter().enumerate() {
        if options.sources[..index].contains(source) {
            return Err(EcmaCodegenError::DuplicateWebSource {
                source: source.as_str().to_owned(),
            });
        }
    }

    let expected = |source: EcmaLocaleSource| options.sources.contains(&source);
    let checks = [
        (EcmaLocaleSource::Path, options.locale_switch.writes_path),
        (
            EcmaLocaleSource::Cookie,
            options.locale_switch.writes_cookie,
        ),
        (
            EcmaLocaleSource::LocalStorage,
            options.locale_switch.writes_local_storage,
        ),
    ];
    for (source, actual) in checks {
        let expected = expected(source);
        if actual != expected {
            return Err(EcmaCodegenError::WebLocaleSwitchPlanMismatch {
                source: source.as_str(),
                expected,
                actual,
            });
        }
    }
    Ok(())
}

fn validate_namespace_output_paths(
    schema: &IrModule,
    locales: &[EcmaLocaleModule],
    options: &EcmaProjectOptions,
) -> Result<(), EcmaCodegenError> {
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
            return Err(EcmaCodegenError::InvalidNamespacePathComponent { namespace, reason });
        }
        let folded = stem.to_ascii_lowercase();
        if folded == "_runtime" || folded == "_globals" {
            let locale = &locales[0].locale;
            let owner = if folded == "_runtime" {
                "locale runtime"
            } else {
                "locale globals"
            };
            return Err(EcmaCodegenError::OutputPathCollision {
                path: format!("locales/{locale}/{stem}.js"),
                conflicts_with: format!("locales/{locale}/{folded}.js ({owner})"),
            });
        }
        if let Some((conflicting_namespace, conflicting_stem)) = output_stems.get(&folded) {
            let locale = &locales[0].locale;
            return Err(EcmaCodegenError::OutputPathCollision {
                path: format!("locales/{locale}/{stem}.js"),
                conflicts_with: format!(
                    "locales/{locale}/{conflicting_stem}.js (namespace `{conflicting_namespace}`)"
                ),
            });
        }
        output_stems.insert(folded, (namespace, stem));
    }
    Ok(())
}

fn validate_text_direction(locale: &str) -> Result<(), EcmaCodegenError> {
    match linguini_cldr::built_in_text_direction(locale) {
        Some("ltr" | "rtl") => Ok(()),
        Some(direction) => Err(EcmaCodegenError::UnsupportedTextDirection {
            locale: locale.to_owned(),
            direction: direction.to_owned(),
        }),
        None => Err(EcmaCodegenError::MissingTextDirection {
            locale: locale.to_owned(),
        }),
    }
}

pub fn generate_project_files(
    project: &ValidatedEcmaProject<'_>,
) -> Result<Vec<EcmaGeneratedFile>, EcmaCodegenError> {
    project_output::generate_project_files(project)
}

/// Generates the schema-owned JavaScript/JSDoc root and its TypeScript declaration companions.
///
/// Locale and project runtime artifacts join this surface in their ordered migration stages.
pub fn generate_schema_files(schema: &SchemaIr) -> Vec<EcmaGeneratedFile> {
    let module = schema.as_module();
    vec![
        EcmaGeneratedFile {
            path: "shared.js".to_owned(),
            contents: generate_shared_module(
                module,
                EcmaModuleOutput::new(
                    EcmaModuleKind::JavaScript,
                    "shared.js",
                    Some("shared.d.ts".to_owned()),
                ),
                false,
            ),
        },
        EcmaGeneratedFile {
            path: "messages.js".to_owned(),
            contents: messages::generate_messages_module(
                module,
                EcmaModuleOutput::new(
                    EcmaModuleKind::JavaScript,
                    "messages.js",
                    Some("messages.d.ts".to_owned()),
                ),
                "./shared.js",
            ),
        },
        EcmaGeneratedFile {
            path: "shared.d.ts".to_owned(),
            contents: generate_shared_module(
                module,
                EcmaModuleOutput::new(EcmaModuleKind::Declaration, "shared.d.ts", None),
                true,
            ),
        },
        EcmaGeneratedFile {
            path: "messages.d.ts".to_owned(),
            contents: messages::generate_messages_module(
                module,
                EcmaModuleOutput::new(EcmaModuleKind::Declaration, "messages.d.ts", None),
                "./shared.js",
            ),
        },
    ]
}

fn validate_generated_output_collisions(
    files: &[EcmaGeneratedFile],
) -> Result<(), EcmaCodegenError> {
    let mut paths = BTreeMap::<String, &str>::new();
    for file in files {
        let folded = file.path.to_ascii_lowercase();
        if let Some(conflict) = paths.insert(folded, &file.path) {
            return Err(EcmaCodegenError::OutputPathCollision {
                path: file.path.clone(),
                conflicts_with: conflict.to_owned(),
            });
        }
    }
    Ok(())
}

pub(super) fn locale_global_import(
    schema: &IrModule,
    implementation: &IrModule,
    locale: &IrModule,
    import_path: &str,
) -> Option<EcmaImport> {
    let required = implementation
        .messages()
        .iter()
        .flat_map(|message| {
            schema
                .messages()
                .iter()
                .find(|signature| signature.name == message.name)
                .map(|signature| {
                    linguini_codegen::direct_message_dependencies(
                        schema, locale, signature, message,
                    )
                })
        })
        .flatten()
        .filter(|(kind, _)| *kind != EcmaSemanticSymbolKind::LocaleEnum)
        .map(|(kind, name)| semantic::semantic_binding(kind, &name))
        .collect::<BTreeSet<_>>();
    let value_names = locale_global_value_names(locale)
        .into_iter()
        .filter(|name| required.contains(name))
        .collect::<Vec<_>>();
    (!value_names.is_empty()).then(|| {
        EcmaImport::named(
            import_path,
            value_names
                .into_iter()
                .map(|name| EcmaNamedImport::new(&name, &name))
                .collect(),
        )
    })
}

pub(super) fn locale_global_value_names(locale: &IrModule) -> Vec<String> {
    locale
        .variables()
        .iter()
        .map(|item| safe_identifier(&item.name))
        .chain(
            locale
                .forms()
                .iter()
                .map(|item| form_binding_name(&item.name)),
        )
        .chain(
            locale
                .functions()
                .iter()
                .map(|item| safe_identifier(&item.name)),
        )
        .collect()
}

pub(super) fn locale_has_globals(locale: &IrModule) -> bool {
    !locale.enums().is_empty()
        || !locale.variables().is_empty()
        || !locale.forms().is_empty()
        || !locale.functions().is_empty()
}

pub(super) fn locale_globals(locale: &IrModule) -> IrModule {
    IrModuleBuilder::seeded(locale)
        .clear_messages()
        .clear_groups()
        .build()
        .expect("global-symbol projection preserves unique declaration names")
}

pub(super) fn locale_without_globals(locale: &IrModule) -> IrModule {
    IrModuleBuilder::seeded(locale)
        .clear_enums()
        .clear_variables()
        .clear_forms()
        .clear_functions()
        .build()
        .expect("message projection preserves unique declaration names")
}

pub(super) fn locale_runtime_import(
    schema: &IrModule,
    locale: &IrModule,
    options: &EcmaOptions,
    runtime_import_path: &str,
) -> Option<EcmaImport> {
    let helpers = runtime_helper_names(schema, locale, options);
    (!helpers.is_empty()).then(|| {
        EcmaImport::named(
            runtime_import_path,
            helpers
                .into_iter()
                .map(|name| EcmaNamedImport::new(&name, &name))
                .collect(),
        )
    })
}

fn runtime_helper_names(
    schema: &IrModule,
    locale: &IrModule,
    options: &EcmaOptions,
) -> Vec<String> {
    let mut helpers = formatter_helper_names(schema, locale);
    if plural_required(schema, locale) {
        helpers.push(options.plural_function.clone());
    }
    helpers
}

fn validate_typed_codegen_ir(
    schema: &SchemaIr,
    locale: &LocaleIr,
    scope: impl Into<String>,
) -> Result<(), EcmaCodegenError> {
    validate_typed_ir(schema, locale)
        .map(|_| ())
        .map_err(|errors| EcmaCodegenError::invalid_ir(scope, errors))
}

fn project_locale_options(
    locale: &str,
    project_options: &EcmaProjectOptions,
) -> Result<EcmaOptions, EcmaCodegenError> {
    let plural_function = plural_function_name(locale);
    let plural_rules = built_in_plural_rules(locale)
        .ok_or_else(|| EcmaCodegenError::missing_plural_rules(locale))?;
    Ok(EcmaOptions {
        locale: locale.to_owned(),
        plural_function: plural_function.clone(),
        plural_import: None,
        plural_rules: Some(plural_rules),
        included_messages: if project_options.tree_shaking {
            project_options.included_messages.clone()
        } else {
            Vec::new()
        },
    })
}

pub(super) fn visible_schema(schema: &IrModule, options: &EcmaOptions) -> IrModule {
    if options.included_messages.is_empty() {
        return schema.clone();
    }

    let is_selected = |name: &str| {
        options.included_messages.iter().any(|selected| {
            selected == name
                || name
                    .strip_prefix(selected.as_str())
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    };
    let retained_message_names = schema
        .messages()
        .iter()
        .map(|message| message.name.as_str())
        .filter(|name| is_selected(name))
        .collect::<Vec<_>>();
    IrModuleBuilder::seeded(schema)
        .retain_messages(|message| is_selected(&message.name))
        .retain_groups(|group| {
            options.included_messages.iter().any(|selected| {
                is_path_or_descendant(&group.name, selected)
                    || is_path_or_descendant(selected, &group.name)
            }) || retained_message_names
                .iter()
                .any(|message| is_path_or_descendant(message, &group.name))
        })
        .build()
        .expect("tree-shaken schema projection preserves unique declaration names")
}

pub(super) fn locale_module_for_schema(locale: &IrModule, schema: &IrModule) -> IrModule {
    let schema_groups = schema
        .groups()
        .iter()
        .map(|group| group.name.as_str())
        .collect::<BTreeSet<_>>();
    IrModuleBuilder::seeded(locale)
        .retain_messages(|message| {
            schema
                .messages()
                .iter()
                .any(|schema_message| schema_message.name == message.name)
        })
        .retain_groups(|group| schema_groups.contains(group.name.as_str()))
        .build()
        .expect("schema-projected locale preserves unique declaration names")
}

pub(super) fn top_level_namespaces(module: &IrModule) -> Vec<String> {
    let mut namespaces = module
        .messages()
        .iter()
        .filter_map(|message| message.name.split_once('.').map(|(namespace, _)| namespace))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    namespaces.sort();
    namespaces.dedup();
    namespaces
}

pub(super) fn namespace_module(module: &IrModule, namespace: &str) -> IrModule {
    let prefix = format!("{namespace}.");
    IrModuleBuilder::seeded(module)
        .retain_messages(|message| message.name.starts_with(&prefix))
        .retain_groups(|group| group.name == namespace || group.name.starts_with(&prefix))
        .build()
        .expect("namespace projection preserves unique declaration names")
}

pub(super) fn root_module(module: &IrModule) -> IrModule {
    IrModuleBuilder::seeded(module)
        .retain_messages(|message| !message.name.contains('.'))
        .retain_groups(|group| !group.name.contains('.'))
        .build()
        .expect("root projection preserves unique declaration names")
}

pub(super) fn root_module_with_locale_items(module: &IrModule) -> IrModule {
    root_module(module)
}

fn fallback_locale_modules(
    locales: &[EcmaLocaleModule],
    base_locale: Option<&str>,
) -> Vec<EcmaLocaleModule> {
    locales
        .iter()
        .map(|locale| EcmaLocaleModule {
            locale: locale.locale.clone(),
            module: fallback_locale_module(locales, &locale.locale, base_locale),
        })
        .collect()
}

fn fallback_locale_module(
    locales: &[EcmaLocaleModule],
    locale: &str,
    base_locale: Option<&str>,
) -> LocaleIr {
    let mut chain = locale_fallback_chain(locales, locale, base_locale);
    chain.reverse();

    let mut merged = FallbackDeclarations::default();
    for fallback_locale in chain {
        if let Some(source) = locales.iter().find(|entry| entry.locale == fallback_locale) {
            merged.absorb(&source.module);
        }
    }
    LocaleIr::try_from_module(
        merged
            .into_module()
            .expect("fallback chain composition preserves unique declaration names"),
    )
    .expect("fallback composition emits only locale declaration kinds")
}

/// Accumulates locale-fallback precedence: the first chain entry supplying a
/// name wins its vector position, and later duplicates replace that entry in
/// place instead of appending.
#[derive(Default)]
struct FallbackDeclarations {
    enums: Vec<IrEnum>,
    type_aliases: Vec<IrTypeAlias>,
    groups: Vec<IrGroup>,
    messages: Vec<IrMessage>,
    variables: Vec<IrVariable>,
    forms: Vec<IrForm>,
    functions: Vec<IrFunction>,
    origins: Vec<IrOrigin>,
}

impl FallbackDeclarations {
    fn absorb(&mut self, source: &IrModule) {
        macro_rules! upsert_kind {
            ($field:ident, $accessor:ident) => {
                for item in source.$accessor() {
                    match self
                        .$field
                        .iter_mut()
                        .find(|existing| existing.name == item.name)
                    {
                        Some(existing) => *existing = item.clone(),
                        None => self.$field.push(item.clone()),
                    }
                }
            };
        }

        upsert_kind!(enums, enums);
        upsert_kind!(type_aliases, type_aliases);
        upsert_kind!(groups, groups);
        upsert_kind!(messages, messages);
        upsert_kind!(variables, variables);
        upsert_kind!(forms, forms);
        upsert_kind!(functions, functions);

        for source_origin in source.origins() {
            let mut origin = source_origin.clone();
            if self
                .origins
                .iter()
                .any(|existing| existing.kind == origin.kind && existing.name == origin.name)
            {
                // Locale fallback precedence is an explicit semantic replacement. Retain both
                // provenance records while making that replacement visible to IR validation.
                origin.is_override = true;
            }
            self.origins.push(origin);
        }
    }

    fn into_module(self) -> Result<IrModule, IrSymbolConflict> {
        let mut builder = IrModuleBuilder::new();
        for item in self.enums {
            builder = builder.push_enum(item);
        }
        for item in self.type_aliases {
            builder = builder.push_type_alias(item);
        }
        for item in self.groups {
            builder = builder.push_group(item);
        }
        for item in self.messages {
            builder = builder.push_message(item);
        }
        for item in self.variables {
            builder = builder.push_variable(item);
        }
        for item in self.forms {
            builder = builder.push_form(item);
        }
        for item in self.functions {
            builder = builder.push_function(item);
        }
        for origin in self.origins {
            builder = builder.push_origin(origin);
        }
        builder.build()
    }
}

pub(crate) fn locale_fallback_chain(
    locales: &[EcmaLocaleModule],
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
        top_level_namespaces, validate_project_inputs, visible_schema, EcmaCodegenError,
        EcmaLocaleModule, EcmaOptions, EcmaProjectOptions,
    };
    use linguini_ir::{lower_locale_typed as lower_locale, lower_schema_typed as lower_schema};
    use linguini_syntax::{parse_locale, parse_schema};

    #[test]
    fn tree_shaking_keeps_selected_group_ancestors_only() {
        let schema = lower_schema(
            &parse_schema("top { keep { title } drop { hidden } }\nunrelated { other }\n")
                .expect("schema parses"),
        );
        let options = EcmaOptions {
            included_messages: vec!["top.keep.title".to_owned()],
            ..EcmaOptions::default()
        };

        let visible = visible_schema(&schema, &options);

        assert_eq!(
            visible
                .messages()
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["top.keep.title"]
        );
        assert_eq!(
            visible
                .groups()
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
        let options = EcmaOptions {
            included_messages: vec!["top".to_owned()],
            ..EcmaOptions::default()
        };

        let visible = visible_schema(&schema, &options);

        assert_eq!(
            visible
                .groups()
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["top", "top.empty", "top.nested"]
        );
    }

    #[test]
    fn direct_empty_group_selection_is_valid_and_unknown_path_stays_rejected() {
        let schema = lower_schema(&parse_schema("top { empty {} }\n").expect("schema parses"));
        let locales = [EcmaLocaleModule {
            locale: "en".to_owned(),
            module: Default::default(),
        }];
        let options = EcmaProjectOptions {
            tree_shaking: true,
            included_messages: vec!["top.empty".to_owned()],
            base_locale: Some("en".to_owned()),
            ..EcmaProjectOptions::default()
        };

        validate_project_inputs(&schema, &locales, &options)
            .expect("declared empty group is valid selection");

        let unknown = EcmaProjectOptions {
            included_messages: vec!["top.unknown".to_owned()],
            ..options
        };
        assert!(matches!(
            validate_project_inputs(&schema, &locales, &unknown),
            Err(EcmaCodegenError::UnknownIncludedMessage { message })
                if message == "top.unknown"
        ));
    }

    #[test]
    fn locale_runtime_path_is_reserved_from_namespace_outputs() {
        let schema = lower_schema(&parse_schema("_runtime { title }\n").expect("schema parses"));
        let locales = [EcmaLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(
                &parse_locale("_runtime { title = Title }\n").expect("locale parses"),
            ),
        }];
        let error = validate_project_inputs(
            &schema,
            &locales,
            &EcmaProjectOptions {
                base_locale: Some("en".to_owned()),
                ..EcmaProjectOptions::default()
            },
        )
        .expect_err("runtime namespace collision");
        assert!(matches!(
            error,
            EcmaCodegenError::OutputPathCollision { path, conflicts_with }
                if path == "locales/en/_runtime.js" && conflicts_with.contains("locale runtime")
        ));
    }

    #[test]
    fn locale_globals_path_is_reserved_from_namespace_outputs() {
        let schema = lower_schema(&parse_schema("_globals { title }\n").expect("schema parses"));
        let locales = [EcmaLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(
                &parse_locale("_globals { title = Title }\n").expect("locale parses"),
            ),
        }];
        let error = validate_project_inputs(
            &schema,
            &locales,
            &EcmaProjectOptions {
                base_locale: Some("en".to_owned()),
                ..EcmaProjectOptions::default()
            },
        )
        .expect_err("globals namespace collision");
        assert!(matches!(
            error,
            EcmaCodegenError::OutputPathCollision { path, conflicts_with }
                if path == "locales/en/_globals.js" && conflicts_with.contains("locale globals")
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
                .groups()
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "alpha.nested"]
        );
        assert_eq!(
            root_module(&module)
                .groups()
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "beta"]
        );
    }

    #[test]
    fn fallback_group_metadata_follows_locale_precedence() {
        let base = EcmaLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(
                &parse_locale(
                    "/// Base section\nsection {\n  title = Base\n  nested {\n    child = Child\n  }\n}\n",
                )
                .expect("base locale parses"),
            ),
        };
        let regional = EcmaLocaleModule {
            locale: "en-US".to_owned(),
            module: lower_locale(
                &parse_locale("/// Regional section\nsection { title = Regional }\n")
                    .expect("regional locale parses"),
            ),
        };

        let merged = fallback_locale_module(&[base, regional], "en-US", Some("en"));

        assert_eq!(merged.groups().len(), 2);
        assert_eq!(merged.groups()[0].name, "section");
        assert_eq!(merged.groups()[0].docs, ["Regional section"]);
        assert_eq!(merged.groups()[1].name, "section.nested");
        assert_eq!(
            merged
                .messages()
                .iter()
                .map(|message| message.name.as_str())
                .collect::<Vec<_>>(),
            ["section.title", "section.nested.child"]
        );
        assert!(merged
            .origins()
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
                .groups()
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["keep"]
        );
    }
}
