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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Cookie => "cookie",
            Self::LocalStorage => "local-storage",
            Self::AcceptLanguage => "accept-language",
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
