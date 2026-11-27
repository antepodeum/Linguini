//! Namespace-scoped fallback source selection shared by diagnostics and navigation.

use super::LinguiniDocument;
use linguini_format::SourceKind;
use std::collections::BTreeMap;

pub(super) struct LocaleSources<'a> {
    documents: BTreeMap<(Option<&'a str>, &'a str), Vec<&'a LinguiniDocument>>,
}

impl<'a> LocaleSources<'a> {
    pub(super) fn new(documents: impl IntoIterator<Item = &'a LinguiniDocument>) -> Self {
        let mut indexed = BTreeMap::<_, Vec<_>>::new();
        for document in documents {
            if document.kind == SourceKind::Locale && document.is_within_safety_limits() {
                if let Some(locale) = document.locale.as_deref() {
                    indexed
                        .entry((document.namespace.as_deref(), locale))
                        .or_default()
                        .push(document);
                }
            }
        }
        Self { documents: indexed }
    }

    pub(super) fn fallbacks(&self, document: &LinguiniDocument) -> Vec<&'a LinguiniDocument> {
        document
            .locale_fallbacks
            .iter()
            .filter_map(|locale| {
                let sources = self
                    .documents
                    .get(&(document.namespace.as_deref(), locale.as_str()))?;
                let mut candidates = sources
                    .iter()
                    .copied()
                    .filter(|source| source.uri != document.uri);
                let source = candidates.next()?;
                candidates.next().is_none().then_some(source)
            })
            .collect()
    }
}
