//! Borrow inherited global declarations without changing source roots or translation coverage.

use crate::NamedSpan;
use linguini_syntax::{LocaleDeclaration, LocaleFile};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn inherited_globals<'a>(
    locale: &'a LocaleFile,
    fallbacks: &[&'a LocaleFile],
) -> Vec<&'a LocaleDeclaration> {
    let mut owned = locale
        .declarations()
        .iter()
        .map(declaration_name)
        .collect::<BTreeSet<_>>();
    let mut inherited = Vec::new();
    for fallback in fallbacks {
        for declaration in fallback.declarations() {
            if !owned.contains(declaration_name(declaration)) && is_global(declaration) {
                inherited.push(declaration);
            }
        }
        // Declarations from one source can share a name in value and call positions. Claim
        // names after the whole source, matching whole-declaration fallback composition.
        owned.extend(fallback.declarations().iter().map(declaration_name));
    }
    inherited
}

fn declaration_name(declaration: &LocaleDeclaration) -> &str {
    match declaration {
        LocaleDeclaration::Enum(item) => &item.name.value,
        LocaleDeclaration::Variable(item) => &item.name.value,
        LocaleDeclaration::Form(item) => &item.name.value,
        LocaleDeclaration::Function(item) => &item.name.value,
        LocaleDeclaration::Message(item) => &item.name.value,
        LocaleDeclaration::Group(item) => &item.name.value,
        LocaleDeclaration::Override(inner) => declaration_name(inner),
    }
}

fn is_global(declaration: &LocaleDeclaration) -> bool {
    match declaration {
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Function(_) => true,
        LocaleDeclaration::Message(_) | LocaleDeclaration::Group(_) => false,
        LocaleDeclaration::Override(inner) => is_global(inner),
    }
}

pub(crate) fn extend_locale_enum_context(
    declaration: &LocaleDeclaration,
    enum_variants: &mut BTreeMap<String, Vec<NamedSpan>>,
) {
    match declaration {
        LocaleDeclaration::Enum(item) => {
            enum_variants
                .entry(item.name.value.clone())
                .or_insert_with(|| {
                    item.variants
                        .iter()
                        .map(|variant| NamedSpan::new(&variant.value, variant.span))
                        .collect()
                });
        }
        LocaleDeclaration::Override(inner) => {
            extend_locale_enum_context(inner, enum_variants);
        }
        LocaleDeclaration::Form(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Function(_)
        | LocaleDeclaration::Message(_)
        | LocaleDeclaration::Group(_) => {}
    }
}
