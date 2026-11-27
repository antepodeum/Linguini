//! Locale composition and the stable codegen facade over shared CLDR project matching.

mod composition;

pub use composition::compose_locale_modules;

/// Stable codegen API for the shared CLDR project locale planner.
///
/// Matching and table planning are implemented by [`linguini_cldr::ProjectLocalePlan`].
/// This facade preserves the original Rust type identity and method signatures.
#[derive(Debug, Clone)]
pub struct ProjectLocalePlan<'a>(linguini_cldr::ProjectLocalePlan<'a>);

impl<'a> ProjectLocalePlan<'a> {
    /// Cache each declared tag's canonical form once for all project lookups.
    pub fn new(locales: impl IntoIterator<Item = &'a str>) -> Self {
        Self(linguini_cldr::ProjectLocalePlan::new(locales))
    }

    /// Most-specific declared locale first, followed by an optional project base.
    pub fn fallback_chain(&self, requested: &str, base_locale: Option<&str>) -> Vec<String> {
        self.0.fallback_chain(requested, base_locale)
    }

    /// Deterministic lowercase input tags mapped to declared project locale names.
    pub fn resolutions(&self) -> std::collections::BTreeMap<String, String> {
        self.0.resolutions()
    }
}

#[cfg(test)]
mod tests;
