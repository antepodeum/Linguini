//! External capabilities supplied by an application or framework integration.

use linguini_codegen_ecmascript::{EcmaImport, EcmaImportBindings, EcmaNamedImport};

/// One named ESM binding; executable host behavior stays in its owning module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvelteHostImport {
    pub module: String,
    pub name: String,
}

impl SvelteHostImport {
    pub fn new(module: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
        }
    }

    pub(crate) fn value_import(&self, local: &str) -> EcmaImport {
        EcmaImport::named(&self.module, vec![EcmaNamedImport::new(&self.name, local)])
    }

    pub(crate) fn type_import(&self, local: &str) -> EcmaImport {
        EcmaImport {
            specifier: self.module.clone(),
            bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(&self.name, local)]),
        }
    }
}

/// Optional browser, deployment-path, request-locale, and navigation bindings.
///
/// `request_locale` is a getter called during rendering, never at module initialization.
/// `navigate` accepts an href and locale-switch options and resolves after navigation.
/// Without a navigation binding, controls use ordinary browser location navigation.
#[derive(Debug, Clone, Default)]
pub struct SvelteHost {
    pub browser: Option<SvelteHostImport>,
    pub base_path: Option<SvelteHostImport>,
    pub request_locale: Option<SvelteHostImport>,
    pub navigate: Option<SvelteHostImport>,
    pub navigation_state: Option<SvelteHostImport>,
}

#[cfg(test)]
pub(crate) fn test_host() -> SvelteHost {
    let binding = |name| Some(SvelteHostImport::new("../host.js", name));
    SvelteHost {
        browser: binding("browser"),
        base_path: binding("base"),
        request_locale: binding("getRequestLocale"),
        navigate: binding("navigate"),
        navigation_state: binding("NavigationState"),
    }
}
