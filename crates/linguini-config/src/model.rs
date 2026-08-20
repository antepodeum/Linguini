use crate::error::{ConfigError, ConfigResult};
use std::collections::BTreeSet;
use std::path::{Component, Path};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LinguiniConfig {
    pub project: ProjectConfig,
    pub paths: PathsConfig,
    pub targets: TargetsConfig,
    pub analysis: AnalysisConfig,
    pub web: WebConfig,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectConfig {
    pub name: String,
    pub default_locale: String,
    pub locales: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PathsConfig {
    pub schema: String,
    pub locale: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct AnalysisConfig {
    pub unused_messages: Option<UnusedMessagesConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnusedMessagesConfig {
    pub sources: Vec<String>,
    pub exclude: Vec<String>,
    /// Canonical message paths or namespace/group prefixes whose use is resolved dynamically.
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct TargetsConfig {
    pub ts: Option<TypeScriptTargetConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeScriptTargetConfig {
    pub out: String,
    pub declaration: bool,
    pub gitignore: bool,
    pub tree_shaking: bool,
    pub messages: Vec<String>,
    pub framework: Option<String>,
    pub bundler: Option<TypeScriptBundlerConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeScriptBundlerConfig {
    pub sources: Vec<String>,
    pub exclude: Vec<String>,
    /// Controls how locale modules are selected by the bundler runtime.
    ///
    /// The default is [`TypeScriptBundlerLocaleLoading::Eager`].
    pub locale_loading: TypeScriptBundlerLocaleLoading,
    /// Controls how the bundler handles computed or otherwise dynamic message access.
    ///
    /// The default is [`TypeScriptBundlerDynamicMode::Error`] with no escapes. In
    /// [`TypeScriptBundlerDynamicMode::Bundle`] mode, `allow` is a finite list of
    /// canonical dotted message paths that may be resolved dynamically.
    pub dynamic: TypeScriptBundlerDynamicConfig,
}

/// Policy for loading locale modules in the TypeScript bundler integration.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum TypeScriptBundlerLocaleLoading {
    /// Include all generated locale modules in the bundle.
    #[default]
    Eager,
    /// Load locale modules on demand at runtime.
    Dynamic,
}

impl TypeScriptBundlerLocaleLoading {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eager => "eager",
            Self::Dynamic => "dynamic",
        }
    }
}

/// Policy for dynamic message access in the TypeScript bundler integration.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum TypeScriptBundlerDynamicMode {
    /// Reject dynamic access during a strict bundler build.
    #[default]
    Error,
    /// Permit only the explicitly listed finite message paths.
    Bundle,
}

impl TypeScriptBundlerDynamicMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Bundle => "bundle",
        }
    }
}

/// Configuration for dynamic message access under `targets.ts.bundler.dynamic`.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct TypeScriptBundlerDynamicConfig {
    pub mode: TypeScriptBundlerDynamicMode,
    /// Exact canonical dotted message paths admitted in bundle mode.
    pub allow: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebConfig {
    pub configured: bool,
    pub routing: WebRoutingConfig,
    pub locale: WebLocaleConfig,
    pub cookie: Option<WebCookieConfig>,
    pub local_storage: Option<WebLocalStorageConfig>,
    pub links: WebLinksConfig,
    pub routes: WebRoutesConfig,
    pub switch_route: Option<WebSwitchRouteConfig>,
    pub locale_switch: LocaleSwitchPlan,
}

/// Closed, generated web policy lowered from a validated [`WebConfig`].
///
/// Code generators should consume this capability set instead of reaching into the
/// individual configuration namespaces.  The source order is retained exactly as
/// configured; capability fields are present only when their source was selected by
/// validation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebFeatures {
    pub locale_prefix: LocalePrefixMode,
    pub canonical: CanonicalMode,
    pub source_order: Vec<LocaleSource>,
    pub cookie: Option<WebCookieConfig>,
    pub local_storage: Option<WebLocalStorageConfig>,
    pub links: LinkMode,
    pub route_exclusions: Vec<String>,
    pub switch_route: Option<WebSwitchRouteConfig>,
    pub locale_switch: LocaleSwitchPlan,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WebRoutingConfig {
    pub locale_prefix: LocalePrefixMode,
    pub canonical: CanonicalMode,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LocalePrefixMode {
    Always,
    ExceptDefault,
    Never,
}

impl LocalePrefixMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::ExceptDefault => "except-default",
            Self::Never => "never",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CanonicalMode {
    Redirect,
    Preserve,
}

impl CanonicalMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Redirect => "redirect",
            Self::Preserve => "preserve",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebLocaleConfig {
    pub sources: Vec<LocaleSource>,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum LocaleSource {
    Path,
    Cookie,
    LocalStorage,
    AcceptLanguage,
}

impl LocaleSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Cookie => "cookie",
            Self::LocalStorage => "local-storage",
            Self::AcceptLanguage => "accept-language",
        }
    }

    pub(crate) fn parse(value: &str) -> ConfigResult<Self> {
        match value {
            "path" => Ok(Self::Path),
            "cookie" => Ok(Self::Cookie),
            "local-storage" => Ok(Self::LocalStorage),
            "accept-language" => Ok(Self::AcceptLanguage),
            _ => Err(ConfigError::InvalidString(value.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebCookieConfig {
    pub name: String,
    pub path: CookiePath,
    pub domain: Option<String>,
    pub max_age_seconds: u64,
    pub same_site: SameSite,
    pub secure: SecurePolicy,
    pub http_only: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CookiePath {
    Auto,
    Explicit(String),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SameSite {
    Lax,
    Strict,
    None,
}

impl SameSite {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lax => "lax",
            Self::Strict => "strict",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SecurePolicy {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebLocalStorageConfig {
    pub key: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WebLinksConfig {
    pub mode: LinkMode,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LinkMode {
    Transform,
    Runtime,
    Manual,
}

impl LinkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transform => "transform",
            Self::Runtime => "runtime",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct WebRoutesConfig {
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebSwitchRouteConfig {
    pub path: String,
    pub return_query: String,
    pub status: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LocaleSwitchPlan {
    pub writes_path: bool,
    pub writes_cookie: bool,
    pub writes_local_storage: bool,
}

impl LocaleSwitchPlan {
    pub(crate) fn compile(sources: &[LocaleSource]) -> Self {
        Self {
            writes_path: sources.contains(&LocaleSource::Path),
            writes_cookie: sources.contains(&LocaleSource::Cookie),
            writes_local_storage: sources.contains(&LocaleSource::LocalStorage),
        }
    }

    pub fn has_server_transport(self) -> bool {
        self.writes_path || self.writes_cookie
    }
}

impl WebConfig {
    /// Lower the validated configuration into one closed capability set.
    ///
    /// Callers should perform [`LinguiniConfig::validate`] before consuming this
    /// value.  The parser already does so, while this method remains deliberately
    /// side-effect free for callers that construct configurations directly.
    pub fn features(&self) -> WebFeatures {
        WebFeatures {
            locale_prefix: self.routing.locale_prefix,
            canonical: self.routing.canonical,
            source_order: self.locale.sources.clone(),
            cookie: self.cookie.clone(),
            local_storage: self.local_storage.clone(),
            links: self.links.mode,
            route_exclusions: self.routes.exclude.clone(),
            switch_route: self.switch_route.clone(),
            locale_switch: self.locale_switch,
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        let sources = vec![
            LocaleSource::Path,
            LocaleSource::Cookie,
            LocaleSource::AcceptLanguage,
        ];
        Self {
            configured: false,
            routing: WebRoutingConfig {
                locale_prefix: LocalePrefixMode::ExceptDefault,
                canonical: CanonicalMode::Redirect,
            },
            locale: WebLocaleConfig {
                sources: sources.clone(),
            },
            cookie: Some(WebCookieConfig {
                name: "LINGUINI_LOCALE".to_owned(),
                path: CookiePath::Auto,
                domain: None,
                max_age_seconds: 365 * 24 * 60 * 60,
                same_site: SameSite::Lax,
                secure: SecurePolicy::Auto,
                http_only: false,
            }),
            local_storage: None,
            links: WebLinksConfig {
                mode: LinkMode::Transform,
            },
            routes: WebRoutesConfig::default(),
            switch_route: None,
            locale_switch: LocaleSwitchPlan::compile(&sources),
        }
    }
}

impl LinguiniConfig {
    pub fn validate(&self) -> ConfigResult<()> {
        validate_project(&self.project)?;
        validate_relative_path("paths.schema", &self.paths.schema)?;
        validate_relative_path("paths.locale", &self.paths.locale)?;
        reject_path_overlap(
            "paths.schema",
            &self.paths.schema,
            "paths.locale",
            &self.paths.locale,
        )?;
        validate_analysis(&self.analysis)?;

        if let Some(ts) = &self.targets.ts {
            validate_relative_path("targets.ts.out", &ts.out)?;
            reject_path_overlap(
                "targets.ts.out",
                &ts.out,
                "paths.schema",
                &self.paths.schema,
            )?;
            reject_path_overlap(
                "targets.ts.out",
                &ts.out,
                "paths.locale",
                &self.paths.locale,
            )?;
            if !ts.tree_shaking && !ts.messages.is_empty() {
                return Err(ConfigError::InvalidString(
                    "targets.ts.messages requires tree_shaking = true".to_owned(),
                ));
            }
            if let Some(framework) = &ts.framework {
                match framework.as_str() {
                    "svelte" | "sveltekit" => {}
                    value => return Err(ConfigError::InvalidString(value.to_owned())),
                }
            }
            if let Some(bundler) = &ts.bundler {
                if !matches!(ts.framework.as_deref(), Some("svelte" | "sveltekit")) {
                    return Err(ConfigError::InvalidString(
                        "targets.ts.bundler requires targets.ts.framework = \"svelte\" or \"sveltekit\""
                            .to_owned(),
                    ));
                }
                validate_application_paths(
                    "targets.ts.bundler.sources",
                    &bundler.sources,
                    "targets.ts.bundler.exclude",
                    &bundler.exclude,
                )?;
                validate_bundler_dynamic(&bundler.dynamic)?;
            }
        }

        validate_web(&self.web)?;
        if self
            .targets
            .ts
            .as_ref()
            .is_some_and(|target| target.framework.as_deref() == Some("sveltekit"))
            && self.web.locale.sources == [LocaleSource::LocalStorage]
        {
            return Err(ConfigError::InvalidString(
                "local-storage-only locale resolution cannot determine the locale during SvelteKit SSR; add `cookie` or `path`, or use the client-only `svelte` framework"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

fn validate_bundler_dynamic(dynamic: &TypeScriptBundlerDynamicConfig) -> ConfigResult<()> {
    if dynamic.mode == TypeScriptBundlerDynamicMode::Error && !dynamic.allow.is_empty() {
        return Err(ConfigError::InvalidString(
            "targets.ts.bundler.dynamic.allow requires mode = \"bundle\"".to_owned(),
        ));
    }
    if dynamic.mode == TypeScriptBundlerDynamicMode::Bundle && dynamic.allow.is_empty() {
        return Err(ConfigError::InvalidArray(
            "targets.ts.bundler.dynamic.allow".to_owned(),
        ));
    }

    let mut seen = BTreeSet::new();
    for path in &dynamic.allow {
        validate_dynamic_message_path(path)?;
        let folded = path.to_lowercase();
        if !seen.insert(folded) {
            return Err(ConfigError::DuplicateKey(format!(
                "targets.ts.bundler.dynamic.allow path `{path}`"
            )));
        }
    }
    Ok(())
}

fn validate_dynamic_message_path(path: &str) -> ConfigResult<()> {
    if path.is_empty() || path.trim() != path {
        return Err(ConfigError::InvalidString(format!(
            "targets.ts.bundler.dynamic.allow = {path}"
        )));
    }
    if path.contains('/')
        || path.contains('\\')
        || path.contains('*')
        || path.contains('?')
        || path.contains('[')
        || path.contains(']')
    {
        return Err(ConfigError::InvalidString(format!(
            "targets.ts.bundler.dynamic.allow = {path}"
        )));
    }
    for segment in path.split('.') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(ConfigError::InvalidString(format!(
                "targets.ts.bundler.dynamic.allow = {path}"
            )));
        }
        if segment
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(ConfigError::InvalidString(format!(
                "targets.ts.bundler.dynamic.allow = {path}"
            )));
        }
    }
    Ok(())
}

fn validate_analysis(analysis: &AnalysisConfig) -> ConfigResult<()> {
    let Some(unused) = &analysis.unused_messages else {
        return Ok(());
    };
    validate_application_paths(
        "analysis.unused_messages.sources",
        &unused.sources,
        "analysis.unused_messages.exclude",
        &unused.exclude,
    )?;

    let mut ignored = BTreeSet::new();
    for prefix in &unused.ignore {
        if prefix.is_empty()
            || prefix.starts_with('.')
            || prefix.ends_with('.')
            || prefix.split('.').any(|segment| {
                segment.is_empty()
                    || segment
                        .chars()
                        .any(|character| character.is_whitespace() || character.is_control())
            })
        {
            return Err(ConfigError::InvalidString(format!(
                "analysis.unused_messages.ignore = {prefix}"
            )));
        }
        if !ignored.insert(prefix) {
            return Err(ConfigError::DuplicateKey(format!(
                "analysis.unused_messages.ignore prefix `{prefix}`"
            )));
        }
    }
    Ok(())
}

fn validate_application_paths(
    sources_field: &'static str,
    sources: &[String],
    exclude_field: &'static str,
    exclude: &[String],
) -> ConfigResult<()> {
    if sources.is_empty() {
        return Err(ConfigError::InvalidArray(sources_field.to_owned()));
    }
    validate_distinct_paths(sources_field, sources)?;
    validate_distinct_paths(exclude_field, exclude)?;
    for source in sources {
        let source_components = portable_folded_components(source);
        if exclude
            .iter()
            .any(|excluded| source_components.starts_with(&portable_folded_components(excluded)))
        {
            return Err(ConfigError::InvalidPath {
                field: sources_field,
                value: source.clone(),
                reason: if exclude_field == "targets.ts.bundler.exclude" {
                    "source is fully covered by targets.ts.bundler.exclude"
                } else {
                    "source is fully covered by analysis.unused_messages.exclude"
                },
            });
        }
    }
    Ok(())
}

fn validate_distinct_paths(field: &'static str, paths: &[String]) -> ConfigResult<()> {
    let mut seen = BTreeSet::new();
    for path in paths {
        validate_relative_path(field, path)?;
        let normalized = portable_components(path).join("/");
        if !seen.insert(normalized.clone()) {
            return Err(ConfigError::DuplicateKey(format!(
                "{field} path `{normalized}`"
            )));
        }
    }
    Ok(())
}

fn validate_project(project: &ProjectConfig) -> ConfigResult<()> {
    if project.name.trim().is_empty() {
        return Err(ConfigError::MissingField("project.name"));
    }
    if project.name.chars().any(|character| character.is_control()) {
        return Err(ConfigError::InvalidString(project.name.clone()));
    }
    if project.locales.is_empty() {
        return Err(ConfigError::InvalidArray("project.locales".to_owned()));
    }

    validate_locale_tag(&project.default_locale)?;
    let mut locales = BTreeSet::new();
    for locale in &project.locales {
        validate_locale_tag(locale)?;
        let folded = locale.to_ascii_lowercase();
        if !locales.insert(folded) {
            return Err(ConfigError::DuplicateKey(format!(
                "project.locales locale `{locale}`"
            )));
        }
    }
    if !project
        .locales
        .iter()
        .any(|locale| locale.eq_ignore_ascii_case(&project.default_locale))
    {
        return Err(ConfigError::MissingField("project.locales default_locale"));
    }
    Ok(())
}

fn validate_web(web: &WebConfig) -> ConfigResult<()> {
    let mut sources = BTreeSet::new();
    for source in &web.locale.sources {
        if !sources.insert(*source) {
            return Err(ConfigError::DuplicateKey(format!(
                "web.locale.sources source `{}`",
                source.as_str()
            )));
        }
    }
    if web.routing.locale_prefix == LocalePrefixMode::Never && sources.contains(&LocaleSource::Path)
    {
        return Err(ConfigError::InvalidString(
            "`path` locale source requires a URL locale prefix".to_owned(),
        ));
    }

    let wants_cookie = sources.contains(&LocaleSource::Cookie);
    if wants_cookie != web.cookie.is_some() {
        return Err(ConfigError::InvalidString(
            "web.cookie capability must match `cookie` in web.locale.sources".to_owned(),
        ));
    }
    let wants_local_storage = sources.contains(&LocaleSource::LocalStorage);
    if wants_local_storage != web.local_storage.is_some() {
        return Err(ConfigError::InvalidString(
            "web.local_storage capability must match `local-storage` in web.locale.sources"
                .to_owned(),
        ));
    }

    if let Some(cookie) = &web.cookie {
        validate_cookie(cookie)?;
    }
    if let Some(storage) = &web.local_storage {
        if storage.key.is_empty() || storage.key.chars().any(|character| character.is_control()) {
            return Err(ConfigError::InvalidString(storage.key.clone()));
        }
    }
    for pattern in &web.routes.exclude {
        if !pattern.starts_with('/') || pattern.starts_with("//") || pattern.contains(['?', '#']) {
            return Err(ConfigError::InvalidString(pattern.clone()));
        }
    }

    let compiled_plan = LocaleSwitchPlan::compile(&web.locale.sources);
    if web.locale_switch != compiled_plan {
        return Err(ConfigError::InvalidString(
            "web.locale_switch does not match configured locale sources".to_owned(),
        ));
    }
    if let Some(route) = &web.switch_route {
        validate_switch_route(route)?;
        if !compiled_plan.has_server_transport() {
            return Err(ConfigError::InvalidString(
                "web.switch_route requires `path` or `cookie` in web.locale.sources".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_cookie(cookie: &WebCookieConfig) -> ConfigResult<()> {
    if cookie.name.is_empty()
        || cookie.name.bytes().any(|byte| {
            byte <= 0x20
                || byte >= 0x7f
                || matches!(
                    byte,
                    b'(' | b')'
                        | b'<'
                        | b'>'
                        | b'@'
                        | b','
                        | b';'
                        | b':'
                        | b'\\'
                        | b'"'
                        | b'/'
                        | b'['
                        | b']'
                        | b'?'
                        | b'='
                        | b'{'
                        | b'}'
                )
        })
    {
        return Err(ConfigError::InvalidString(cookie.name.clone()));
    }
    if let CookiePath::Explicit(path) = &cookie.path {
        if !path.starts_with('/')
            || path
                .bytes()
                .any(|byte| byte < 0x20 || byte == 0x7f || byte == b';')
        {
            return Err(ConfigError::InvalidString(path.clone()));
        }
    }
    if let Some(domain) = &cookie.domain {
        if domain.is_empty()
            || domain.starts_with('.')
            || domain.ends_with('.')
            || domain
                .bytes()
                .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-')))
        {
            return Err(ConfigError::InvalidString(domain.clone()));
        }
    }
    if cookie.max_age_seconds == 0 {
        return Err(ConfigError::InvalidString(
            "web.cookie.max_age must be positive".to_owned(),
        ));
    }
    if cookie.same_site == SameSite::None && cookie.secure != SecurePolicy::Always {
        return Err(ConfigError::InvalidString(
            "SameSite=None requires web.cookie.secure = true".to_owned(),
        ));
    }
    if cookie.http_only {
        return Err(ConfigError::InvalidString(
            "web.cookie.http_only must be false while browser locale switching is enabled"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_switch_route(route: &WebSwitchRouteConfig) -> ConfigResult<()> {
    if !route.path.starts_with('/')
        || route.path.starts_with("//")
        || route.path.matches("{locale}").count() != 1
        || route.path.contains(['?', '#'])
        || route.path.split('/').any(|segment| segment == "..")
    {
        return Err(ConfigError::InvalidString(route.path.clone()));
    }
    if route.return_query.is_empty()
        || !route
            .return_query
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(ConfigError::InvalidString(route.return_query.clone()));
    }
    if !matches!(route.status, 301 | 302 | 303 | 307 | 308) {
        return Err(ConfigError::InvalidString(route.status.to_string()));
    }
    Ok(())
}

pub(crate) fn validate_relative_path(field: &'static str, value: &str) -> ConfigResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "path must not be empty",
        });
    }

    let portable = trimmed.replace('\\', "/");
    if portable != trimmed {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "use `/` as the portable path separator",
        });
    }
    let has_windows_prefix = portable
        .as_bytes()
        .get(1)
        .is_some_and(|character| *character == b':');
    if portable.starts_with('/') || has_windows_prefix || Path::new(trimmed).is_absolute() {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "path must be project-relative",
        });
    }

    if portable.split('/').any(|segment| segment == "..")
        || Path::new(trimmed)
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "parent traversal is not allowed",
        });
    }

    if portable
        .split('/')
        .all(|segment| segment.is_empty() || segment == ".")
    {
        return Err(ConfigError::InvalidPath {
            field,
            value: value.to_owned(),
            reason: "path must not resolve to the project root",
        });
    }
    Ok(())
}

fn reject_path_overlap(
    left_field: &'static str,
    left: &str,
    right_field: &'static str,
    right: &str,
) -> ConfigResult<()> {
    let left = portable_components(left);
    let right = portable_components(right);
    if left.starts_with(&right) || right.starts_with(&left) {
        return Err(ConfigError::InvalidPath {
            field: left_field,
            value: left.join("/"),
            reason: match right_field {
                "paths.schema" => "path overlaps the schema source root",
                "paths.locale" => "path overlaps the locale source root",
                _ => "path overlaps another project source root",
            },
        });
    }
    Ok(())
}

fn portable_components(value: &str) -> Vec<String> {
    value
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .map(str::to_owned)
        .collect()
}

fn portable_folded_components(value: &str) -> Vec<String> {
    portable_components(value)
        .into_iter()
        .map(|component| component.to_ascii_lowercase())
        .collect()
}

pub fn validate_locale_tag(tag: &str) -> ConfigResult<()> {
    canonicalize_locale_tag(tag).map(|_| ())
}

pub(crate) fn canonicalize_locale_tag(tag: &str) -> ConfigResult<String> {
    if tag.is_empty() || !tag.is_ascii() || tag.contains('_') {
        return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
    }

    if is_grandfathered(tag) {
        return Ok(tag.to_ascii_lowercase());
    }

    let subtags = tag.split('-').collect::<Vec<_>>();
    if subtags.iter().any(|part| part.is_empty()) {
        return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
    }
    if subtags[0].eq_ignore_ascii_case("x") {
        if subtags.len() < 2
            || !subtags[1..]
                .iter()
                .all(|part| valid_alphanumeric(part, 1, 8))
        {
            return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
        }
        return Ok(subtags
            .iter()
            .map(|part| part.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("-"));
    }

    let language = subtags[0];
    if !valid_alpha(language, 2, 8) {
        return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
    }

    let mut canonical = vec![language.to_ascii_lowercase()];
    let mut index = 1;
    if language.len() <= 3 {
        let mut extlang_count = 0;
        while index < subtags.len() && extlang_count < 3 && valid_alpha(subtags[index], 3, 3) {
            canonical.push(subtags[index].to_ascii_lowercase());
            index += 1;
            extlang_count += 1;
        }
    }
    if index < subtags.len() && valid_alpha(subtags[index], 4, 4) {
        let script = subtags[index].to_ascii_lowercase();
        canonical.push(format!(
            "{}{}",
            script[..1].to_ascii_uppercase(),
            &script[1..]
        ));
        index += 1;
    }
    if index < subtags.len()
        && (valid_alpha(subtags[index], 2, 2) || valid_numeric(subtags[index], 3, 3))
    {
        canonical.push(subtags[index].to_ascii_uppercase());
        index += 1;
    }

    let mut variants = BTreeSet::new();
    while index < subtags.len() && valid_variant(subtags[index]) {
        let variant = subtags[index].to_ascii_lowercase();
        if !variants.insert(variant.clone()) {
            return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
        }
        canonical.push(variant);
        index += 1;
    }

    let mut extensions = BTreeSet::new();
    while index < subtags.len()
        && subtags[index].len() == 1
        && subtags[index]
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        && !subtags[index].eq_ignore_ascii_case("x")
    {
        let singleton = subtags[index].to_ascii_lowercase();
        if !extensions.insert(singleton.clone()) {
            return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
        }
        canonical.push(singleton);
        index += 1;
        let start = index;
        while index < subtags.len() && valid_alphanumeric(subtags[index], 2, 8) {
            canonical.push(subtags[index].to_ascii_lowercase());
            index += 1;
        }
        if start == index {
            return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
        }
    }

    if index < subtags.len() && subtags[index].eq_ignore_ascii_case("x") {
        canonical.push("x".to_owned());
        index += 1;
        let start = index;
        while index < subtags.len() && valid_alphanumeric(subtags[index], 1, 8) {
            canonical.push(subtags[index].to_ascii_lowercase());
            index += 1;
        }
        if start == index {
            return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
        }
    }

    if index != subtags.len() {
        return Err(ConfigError::InvalidLocaleTag(tag.to_owned()));
    }
    Ok(canonical.join("-"))
}

fn valid_alpha(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len())
        && value
            .chars()
            .all(|character| character.is_ascii_alphabetic())
}

fn valid_numeric(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len()) && value.chars().all(|character| character.is_ascii_digit())
}

fn valid_alphanumeric(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

fn valid_variant(value: &str) -> bool {
    valid_alphanumeric(value, 5, 8)
        || (value.len() == 4
            && value
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric()))
}

fn is_grandfathered(tag: &str) -> bool {
    const TAGS: &[&str] = &[
        "art-lojban",
        "cel-gaulish",
        "en-gb-oed",
        "i-ami",
        "i-bnn",
        "i-default",
        "i-enochian",
        "i-hak",
        "i-klingon",
        "i-lux",
        "i-mingo",
        "i-navajo",
        "i-pwn",
        "i-tao",
        "i-tay",
        "i-tsu",
        "no-bok",
        "no-nyn",
        "sgn-be-fr",
        "sgn-be-nl",
        "sgn-ch-de",
        "zh-guoyu",
        "zh-hakka",
        "zh-min",
        "zh-min-nan",
        "zh-xiang",
    ];
    TAGS.iter().any(|known| tag.eq_ignore_ascii_case(known))
}

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_locale_tag, validate_locale_tag, AnalysisConfig, LinguiniConfig, PathsConfig,
        ProjectConfig, TargetsConfig, TypeScriptBundlerConfig, TypeScriptBundlerDynamicConfig,
        TypeScriptBundlerDynamicMode, TypeScriptBundlerLocaleLoading, TypeScriptTargetConfig,
        WebConfig,
    };
    use crate::ConfigError;

    #[test]
    fn accepts_and_canonicalizes_bcp47_tags() {
        for (input, expected) in [
            ("ru", "ru"),
            ("en-us", "en-US"),
            ("es-419", "es-419"),
            ("zh-hant-tw", "zh-Hant-TW"),
            ("de-CH-1901", "de-CH-1901"),
            ("sl-rozaj-biske", "sl-rozaj-biske"),
            ("en-u-ca-gregory", "en-u-ca-gregory"),
            ("x-company-test", "x-company-test"),
        ] {
            assert_eq!(canonicalize_locale_tag(input).expect("valid tag"), expected);
        }
    }

    #[test]
    fn rejects_invalid_or_duplicate_bcp47_subtags() {
        for tag in [
            "",
            "e",
            "en_US",
            "en--US",
            "en-abc1",
            "en-u",
            "en-u-ca-u-nu-latn",
            "de-1901-1901",
            "x",
        ] {
            assert!(validate_locale_tag(tag).is_err(), "{tag}");
        }
    }

    #[test]
    fn rejects_unsafe_or_overlapping_codegen_paths() {
        for out in [
            "/tmp/generated",
            "../generated",
            r"..\generated",
            r"C:\generated",
            ".",
            "schema/generated",
            "locales",
        ] {
            let config = LinguiniConfig {
                project: ProjectConfig {
                    name: "shop".to_owned(),
                    default_locale: "en".to_owned(),
                    locales: vec!["en".to_owned()],
                },
                paths: PathsConfig {
                    schema: "schema".to_owned(),
                    locale: "locales".to_owned(),
                },
                targets: TargetsConfig {
                    ts: Some(TypeScriptTargetConfig {
                        out: out.to_owned(),
                        declaration: true,
                        gitignore: true,
                        tree_shaking: false,
                        messages: Vec::new(),
                        framework: None,
                        bundler: None,
                    }),
                },
                analysis: super::AnalysisConfig::default(),
                web: super::WebConfig::default(),
            };

            assert!(config.validate().is_err(), "{out}");
        }
    }

    #[test]
    fn validates_dynamic_bundler_policy_without_schema_knowledge() {
        let base = || LinguiniConfig {
            project: ProjectConfig {
                name: "shop".to_owned(),
                default_locale: "en".to_owned(),
                locales: vec!["en".to_owned()],
            },
            paths: PathsConfig {
                schema: "schema".to_owned(),
                locale: "locales".to_owned(),
            },
            targets: TargetsConfig {
                ts: Some(TypeScriptTargetConfig {
                    out: "generated".to_owned(),
                    declaration: true,
                    gitignore: true,
                    tree_shaking: false,
                    messages: Vec::new(),
                    framework: Some("svelte".to_owned()),
                    bundler: Some(TypeScriptBundlerConfig {
                        sources: vec!["src".to_owned()],
                        exclude: Vec::new(),
                        locale_loading: TypeScriptBundlerLocaleLoading::Eager,
                        dynamic: TypeScriptBundlerDynamicConfig {
                            mode: TypeScriptBundlerDynamicMode::Bundle,
                            allow: vec!["unresolved.namespace".to_owned()],
                        },
                    }),
                }),
            },
            analysis: AnalysisConfig::default(),
            web: WebConfig::default(),
        };

        assert!(base().validate().is_ok());

        let mut invalid = base();
        invalid
            .targets
            .ts
            .as_mut()
            .unwrap()
            .bundler
            .as_mut()
            .unwrap()
            .dynamic
            .allow = vec!["main.title".to_owned(), "MAIN.TITLE".to_owned()];
        assert!(matches!(
            invalid.validate(),
            Err(ConfigError::DuplicateKey(_))
        ));

        let mut strict_with_allow = base();
        let dynamic = &mut strict_with_allow
            .targets
            .ts
            .as_mut()
            .unwrap()
            .bundler
            .as_mut()
            .unwrap()
            .dynamic;
        dynamic.mode = TypeScriptBundlerDynamicMode::Error;
        assert!(strict_with_allow.validate().is_err());

        let mut bundle_without_allow = base();
        let dynamic = &mut bundle_without_allow
            .targets
            .ts
            .as_mut()
            .unwrap()
            .bundler
            .as_mut()
            .unwrap()
            .dynamic;
        dynamic.allow.clear();
        assert!(bundle_without_allow.validate().is_err());
    }
}
