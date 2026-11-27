//! Project-scoped matching over the shared pinned CLDR locale graph.

use std::collections::BTreeMap;

use crate::{
    canonicalize_locale, locale_fallback_chain, locale_resolution_candidates, maximize_locale,
};

#[derive(Debug, Clone)]
struct ProjectLocale<'a> {
    name: &'a str,
    canonical: Option<String>,
}

/// Resolves pinned CLDR inheritance against a project's declared locale spellings.
///
/// Project validation belongs to the caller. Matching preserves declared names and
/// their input order, with an exact requested spelling preferred over canonical aliases.
/// The plan contains no target syntax or web routing/persistence policy.
#[derive(Debug, Clone)]
pub struct ProjectLocalePlan<'a> {
    locales: Vec<ProjectLocale<'a>>,
}

impl<'a> ProjectLocalePlan<'a> {
    /// Cache each declared tag's canonical form once for all project lookups.
    pub fn new(locales: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            locales: locales
                .into_iter()
                .map(|name| ProjectLocale {
                    name,
                    canonical: canonicalize_locale(name).ok(),
                })
                .collect(),
        }
    }

    /// Most-specific declared locale first, followed by an optional project base.
    /// The base is appended only once; its declaration is validated by the caller.
    pub fn fallback_chain(&self, requested: &str, base_locale: Option<&str>) -> Vec<String> {
        let mut chain = Vec::new();
        let tags = locale_fallback_chain(requested).unwrap_or_else(|_| vec![requested.to_owned()]);
        for (index, tag) in tags.into_iter().enumerate() {
            let raw_match = self.locales.iter().find(|entry| {
                entry
                    .name
                    .eq_ignore_ascii_case(if index == 0 { requested } else { &tag })
            });
            let canonical_match = || {
                self.locales.iter().find(|entry| {
                    entry
                        .canonical
                        .as_deref()
                        .unwrap_or(entry.name)
                        .eq_ignore_ascii_case(&tag)
                })
            };
            if let Some(exact) = raw_match.or_else(canonical_match).map(|entry| entry.name) {
                if !chain.iter().any(|entry| entry == exact) {
                    chain.push(exact.to_owned());
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

    /// Deterministic lowercase input tags mapped to declared project locale names.
    /// Includes pinned CLDR candidates and the declared languages' likely scripts.
    /// Unresolvable candidates are omitted rather than assigned an implicit base.
    pub fn resolutions(&self) -> BTreeMap<String, String> {
        let mut candidates = locale_resolution_candidates()
            .iter()
            .map(|locale| (*locale).to_owned())
            .collect::<Vec<_>>();
        for locale in &self.locales {
            let Some(canonical) = &locale.canonical else {
                continue;
            };
            let Some(language) = canonical.split('-').next() else {
                continue;
            };
            let Ok(maximized) = maximize_locale(language) else {
                continue;
            };
            if let Some(script) = maximized
                .split('-')
                .nth(1)
                .filter(|subtag| subtag.len() == 4)
            {
                candidates.push(format!("{language}-{script}"));
            }
        }
        candidates.sort();
        candidates.dedup();

        let mut resolution = self
            .locales
            .iter()
            .map(|locale| (locale.name.to_ascii_lowercase(), locale.name.to_owned()))
            .collect::<BTreeMap<_, _>>();
        for candidate in candidates {
            if let Some(resolved) = self.fallback_chain(&candidate, None).into_iter().next() {
                resolution.insert(candidate.to_ascii_lowercase(), resolved);
            }
        }
        resolution
    }
}

#[cfg(test)]
mod tests;
