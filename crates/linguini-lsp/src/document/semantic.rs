use std::collections::{BTreeMap, BTreeSet};

use linguini_format::SourceKind;
use linguini_schema::{locale_navigation, locale_occurrences, schema_occurrences};
pub(super) use linguini_schema::{SemanticKey, SemanticOccurrence};
use linguini_syntax::{lex_schema_with_recovery, Span, TokenKind};

use super::{contains, parsed_locale, parsed_schema, LinguiniDocument, LocaleSources};

#[derive(Debug, Clone)]
pub(super) struct ResolvedOccurrence {
    pub document: LinguiniDocument,
    pub occurrence: SemanticOccurrence,
}

/// Immutable semantic index for one workspace navigation operation.
///
/// Reference, definition, and rename resolution all consume this same snapshot. Rename conflict
/// checks therefore cannot rewalk a document after the target set has already been selected.
#[derive(Debug, Clone)]
struct NavigationDatabase {
    documents: BTreeMap<String, LinguiniDocument>,
    occurrences: BTreeMap<String, Vec<SemanticOccurrence>>,
    global_owners: BTreeMap<String, Vec<(SemanticKey, String)>>,
    inherited_scopes: BTreeMap<String, Vec<(String, Span)>>,
}

impl NavigationDatabase {
    fn build(
        documents: impl IntoIterator<Item = LinguiniDocument>,
        source: &LinguiniDocument,
    ) -> Self {
        let mut documents = documents
            .into_iter()
            .map(|document| (document.uri.clone(), document))
            .collect::<BTreeMap<_, _>>();
        documents.insert(source.uri.clone(), source.clone());
        let locale_sources = LocaleSources::new(documents.values());
        let mut occurrences_by_uri = BTreeMap::new();
        let mut global_owners = BTreeMap::new();
        let mut inherited_scopes = BTreeMap::new();
        for (uri, document) in &documents {
            let Some(locale) = parsed_locale(document).and_then(|parsed| parsed.ast.as_ref())
            else {
                occurrences_by_uri.insert(uri.clone(), occurrences(document));
                continue;
            };
            let sources = std::iter::once(document)
                .chain(locale_sources.fallbacks(document))
                .filter_map(|source| Some((source, parsed_locale(source)?.ast.as_ref()?)))
                .collect::<Vec<_>>();
            let fallbacks = sources
                .iter()
                .skip(1)
                .map(|(_, ast)| *ast)
                .collect::<Vec<_>>();
            let navigation = locale_navigation(locale, &fallbacks);
            let source_uri = |ast| {
                sources
                    .iter()
                    .find(|(_, candidate)| std::ptr::eq(*candidate, ast))
                    .map(|(document, _)| document.uri.clone())
            };
            let scopes = navigation
                .inherited
                .iter()
                .filter_map(|global| Some((source_uri(global.source)?, global.declaration.span())))
                .collect();
            let owners = navigation
                .globals
                .into_iter()
                .filter_map(|global| Some((global.occurrence.key, source_uri(global.source)?)))
                .collect();
            occurrences_by_uri.insert(uri.clone(), navigation.occurrences);
            global_owners.insert(uri.clone(), owners);
            inherited_scopes.insert(uri.clone(), scopes);
        }
        Self {
            documents,
            occurrences: occurrences_by_uri,
            global_owners,
            inherited_scopes,
        }
    }

    fn global_owner(&self, document: &LinguiniDocument, key: &SemanticKey) -> Option<&str> {
        // Schema-owned types and members retain their cross-locale identity.
        if document.kind != SourceKind::Locale || self.schema_declares(document, key) {
            return None;
        }
        self.global_owners
            .get(&document.uri)?
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, uri)| uri.as_str())
    }

    fn schema_declares(&self, document: &LinguiniDocument, key: &SemanticKey) -> bool {
        matches!(key, SemanticKey::Type(_) | SemanticKey::EnumVariant { .. })
            && self.documents.values().any(|candidate| {
                candidate.kind == SourceKind::Schema
                    && same_namespace(document, candidate, None)
                    && self
                        .occurrences(&candidate.uri)
                        .iter()
                        .any(|occurrence| occurrence.declaration && occurrence.key == *key)
            })
    }

    fn rename_changes_shared_binding(&self, source: &LinguiniDocument, key: &SemanticKey) -> bool {
        let Some(target_owner) = self.global_owner(source, key) else {
            return false;
        };
        self.inherited_scopes.iter().any(|(context_uri, scopes)| {
            let context_owner = self.global_owner(&self.documents[context_uri], key);
            scopes.iter().any(|(source_uri, scope)| {
                let original_owner = self.global_owner(&self.documents[source_uri], key);
                original_owner != context_owner
                    && (original_owner == Some(target_owner) || context_owner == Some(target_owner))
                    && self.occurrences(source_uri).iter().any(|occurrence| {
                        !occurrence.declaration
                            && occurrence.key == *key
                            && occurrence.span.source == scope.source
                            && scope.start <= occurrence.span.start
                            && occurrence.span.end <= scope.end
                    })
            })
        })
    }

    fn occurrences(&self, uri: &str) -> &[SemanticOccurrence] {
        self.occurrences.get(uri).map(Vec::as_slice).unwrap_or(&[])
    }

    fn occurrence_at(
        &self,
        document: &LinguiniDocument,
        offset: usize,
    ) -> Option<SemanticOccurrence> {
        self.occurrences(&document.uri)
            .iter()
            .find(|occurrence| contains(occurrence.span, offset))
            .cloned()
    }

    fn resolve(&self, source: &LinguiniDocument, offset: usize) -> Option<Vec<ResolvedOccurrence>> {
        let source_occurrence = self.occurrence_at(source, offset)?;
        let global_owner = self.global_owner(source, &source_occurrence.key);
        let document_local = match &source_occurrence.key {
            SemanticKey::FormAttribute { .. }
            | SemanticKey::Variable(_)
            | SemanticKey::Function(_) => true,
            SemanticKey::Parameter { owner, .. } => {
                owner.starts_with("fn:") || owner.starts_with("inline:")
            }
            SemanticKey::Type(_) | SemanticKey::EnumVariant { .. } => {
                source.kind == SourceKind::Locale
                    && !self.schema_declares(source, &source_occurrence.key)
            }
            SemanticKey::Message(_) => false,
        };
        let schema_anchor =
            if global_owner.is_some() || document_local || source.namespace.is_some() {
                None
            } else {
                unique_schema_anchor(self, source, &source_occurrence)?
            };

        let mut resolved = Vec::new();
        for document in self.documents.values() {
            if let Some(owner) = global_owner {
                if !same_namespace(source, document, None)
                    || self.global_owner(document, &source_occurrence.key) != Some(owner)
                {
                    continue;
                }
            } else {
                if document_local && document.uri != source.uri {
                    continue;
                }
                if !document_local && !same_namespace(source, document, schema_anchor.as_deref()) {
                    continue;
                }
            }
            for occurrence in self.occurrences(&document.uri) {
                if occurrence.key == source_occurrence.key {
                    resolved.push(ResolvedOccurrence {
                        document: document.clone(),
                        occurrence: occurrence.clone(),
                    });
                }
            }
        }

        (!resolved.is_empty()).then_some(resolved)
    }
}

pub(super) fn occurrence_at(
    document: &LinguiniDocument,
    offset: usize,
) -> Option<SemanticOccurrence> {
    occurrences(document)
        .into_iter()
        .find(|occurrence| contains(occurrence.span, offset))
}

pub(super) fn resolved_occurrences(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
) -> Option<Vec<ResolvedOccurrence>> {
    NavigationDatabase::build(documents, source).resolve(source, offset)
}

pub(super) fn definition_occurrence(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
) -> Option<ResolvedOccurrence> {
    let database = NavigationDatabase::build(documents, source);
    let source_occurrence = database.occurrence_at(source, offset)?;
    let resolved = database.resolve(source, offset)?;
    let declarations = resolved
        .into_iter()
        .filter(|candidate| candidate.occurrence.declaration)
        .collect::<Vec<_>>();

    if source.kind == SourceKind::Schema && source_occurrence.declaration {
        return declarations.into_iter().find(|candidate| {
            candidate.document.uri == source.uri
                && candidate.occurrence.span == source_occurrence.span
        });
    }

    let schema_declarations = declarations
        .iter()
        .filter(|candidate| candidate.document.kind == SourceKind::Schema)
        .collect::<Vec<_>>();
    if schema_declarations.len() == 1 {
        return schema_declarations.into_iter().next().cloned();
    }
    if schema_declarations.len() > 1 {
        return None;
    }

    let owner = database
        .global_owner(source, &source_occurrence.key)
        .unwrap_or(&source.uri);
    let local = declarations
        .into_iter()
        .filter(|candidate| candidate.document.uri == owner)
        .collect::<Vec<_>>();
    (local.len() == 1).then(|| local[0].clone())
}

pub(super) fn rename_occurrences(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
    new_name: &str,
) -> Option<Vec<ResolvedOccurrence>> {
    if !valid_identifier(new_name) {
        return None;
    }
    let database = NavigationDatabase::build(documents, source);
    let resolved = database.resolve(source, offset)?;
    let source_occurrence = database.occurrence_at(source, offset)?;
    let renamed = renamed_key(&source_occurrence.key, new_name);
    if renamed == source_occurrence.key {
        return Some(Vec::new());
    }
    // An inherited body may bind the same spelling to different locale overrides. Editing
    // that shared source would alter another consumer's behavior, so refuse the rename.
    if database.rename_changes_shared_binding(source, &source_occurrence.key) {
        return None;
    }

    let affected_documents = resolved
        .iter()
        .map(|candidate| candidate.document.uri.as_str())
        .collect::<BTreeSet<_>>();
    for uri in affected_documents {
        if database.occurrences(uri).iter().any(|occurrence| {
            occurrence.declaration
                && occurrence.key != source_occurrence.key
                && declaration_keys_conflict(&renamed, &occurrence.key)
        }) || database.global_owners.get(uri).is_some_and(|globals| {
            globals.iter().any(|(key, _)| {
                *key != source_occurrence.key && declaration_keys_conflict(&renamed, key)
            })
        }) {
            return None;
        }
    }

    Some(resolved)
}

pub(super) fn valid_identifier(value: &str) -> bool {
    if value == "_"
        || matches!(
            value,
            "enum" | "type" | "impl" | "form" | "fn" | "let" | "override"
        )
    {
        return false;
    }
    let lexed = lex_schema_with_recovery(value);
    lexed.errors.is_empty()
        && matches!(
            lexed.tokens.as_slice(),
            [token]
                if token.span == Span::new(0, value.len())
                    && matches!(&token.kind, TokenKind::Ident(name) if name == value)
        )
}

pub(super) fn occurrences(document: &LinguiniDocument) -> Vec<SemanticOccurrence> {
    document
        .semantic_occurrences
        .get_or_init(|| match document.kind {
            SourceKind::Schema => parsed_schema(document)
                .and_then(|parsed| parsed.ast.as_ref())
                .map(schema_occurrences)
                .unwrap_or_default(),
            SourceKind::Locale => parsed_locale(document)
                .and_then(|parsed| parsed.ast.as_ref())
                .map(locale_occurrences)
                .unwrap_or_default(),
        })
        .clone()
}

fn declaration_keys_conflict(left: &SemanticKey, right: &SemanticKey) -> bool {
    match (declaration_slot(left), declaration_slot(right)) {
        (Some(left), Some(right)) => left == right,
        _ => left == right,
    }
}

fn declaration_slot(key: &SemanticKey) -> Option<(String, String)> {
    match key {
        SemanticKey::Message(path) => {
            let (scope, name) = path.rsplit_once('.').unwrap_or(("", path.as_str()));
            Some((format!("symbol:{scope}"), name.to_owned()))
        }
        SemanticKey::Type(name) | SemanticKey::Variable(name) | SemanticKey::Function(name) => {
            Some(("symbol:".to_owned(), name.clone()))
        }
        SemanticKey::EnumVariant {
            enumeration,
            variant,
        } => Some((format!("enum:{enumeration}"), variant.clone())),
        SemanticKey::FormAttribute { form, path } => {
            let (scope, name) = path.rsplit_once('.').unwrap_or(("", path.as_str()));
            Some((format!("form:{form}:{scope}"), name.to_owned()))
        }
        SemanticKey::Parameter { owner, name } => {
            Some((format!("parameter:{owner}"), name.clone()))
        }
    }
}

fn unique_schema_anchor(
    database: &NavigationDatabase,
    source: &LinguiniDocument,
    source_occurrence: &SemanticOccurrence,
) -> Option<Option<String>> {
    let schema_uris = database
        .documents
        .values()
        .filter(|document| document.kind == SourceKind::Schema)
        .filter(|document| {
            database
                .occurrences(&document.uri)
                .iter()
                .any(|occurrence| occurrence.declaration && occurrence.key == source_occurrence.key)
        })
        .map(|document| document.uri.clone())
        .collect::<BTreeSet<_>>();

    if source.kind == SourceKind::Schema && source_occurrence.declaration {
        return Some(Some(source.uri.clone()));
    }
    match schema_uris.len() {
        0 => Some(None),
        1 => schema_uris.into_iter().next().map(Some),
        _ => None,
    }
}

fn same_namespace(
    source: &LinguiniDocument,
    candidate: &LinguiniDocument,
    schema_anchor: Option<&str>,
) -> bool {
    match (&source.namespace, &candidate.namespace) {
        (Some(left), Some(right)) => left == right,
        (Some(_), None) | (None, Some(_)) => false,
        (None, None) => {
            candidate.kind != SourceKind::Schema
                || schema_anchor
                    .map(|anchor| candidate.uri == anchor)
                    .unwrap_or(true)
        }
    }
}

fn renamed_key(key: &SemanticKey, new_name: &str) -> SemanticKey {
    match key {
        SemanticKey::Message(path) => {
            let prefix = path.rsplit_once('.').map(|(prefix, _)| prefix);
            SemanticKey::Message(
                prefix
                    .map(|prefix| format!("{prefix}.{new_name}"))
                    .unwrap_or_else(|| new_name.to_owned()),
            )
        }
        SemanticKey::Type(_) => SemanticKey::Type(new_name.to_owned()),
        SemanticKey::EnumVariant { enumeration, .. } => SemanticKey::EnumVariant {
            enumeration: enumeration.clone(),
            variant: new_name.to_owned(),
        },
        SemanticKey::FormAttribute { form, path } => {
            let prefix = path.rsplit_once('.').map(|(prefix, _)| prefix);
            SemanticKey::FormAttribute {
                form: form.clone(),
                path: prefix
                    .map(|prefix| format!("{prefix}.{new_name}"))
                    .unwrap_or_else(|| new_name.to_owned()),
            }
        }
        SemanticKey::Variable(_) => SemanticKey::Variable(new_name.to_owned()),
        SemanticKey::Function(_) => SemanticKey::Function(new_name.to_owned()),
        SemanticKey::Parameter { owner, .. } => SemanticKey::Parameter {
            owner: owner.clone(),
            name: new_name.to_owned(),
        },
    }
}
