//! Build artifacts for the category unions that a project's message closures can request.

use std::collections::{BTreeMap, BTreeSet};

use linguini_codegen::{plural_category_demand, PluralCategoryDemand};

use super::{
    project_locale_options, EcmaCodegenError, EcmaGeneratedFile, EcmaLocaleRuntimeArtifact,
    ValidatedEcmaProject,
};

/// A private classifier exported under the locale runtime's existing binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaPluralRuntimeVariant {
    pub categories: Vec<String>,
    pub file: EcmaGeneratedFile,
}

/// Compiler-owned selector metadata. `None` demand preserves the complete classifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcmaPluralRuntimePlan {
    pub binding: String,
    pub categories: Vec<String>,
    pub messages: BTreeMap<String, Option<Vec<String>>>,
    pub variants: Vec<EcmaPluralRuntimeVariant>,
}

/// Compile only category unions reachable by selecting the supplied message closures.
/// These are build artifacts; a bundler selects one variant per runtime and payload.
/// The complete runtime continues to own public category results.
pub fn compile_plural_runtime_variants(
    project: &ValidatedEcmaProject<'_>,
    artifact: &EcmaLocaleRuntimeArtifact,
    messages: &[String],
) -> Result<Option<EcmaPluralRuntimePlan>, EcmaCodegenError> {
    if !project
        .locales
        .iter()
        .any(|locale| locale.locale == artifact.locale)
    {
        return Err(EcmaCodegenError::UnknownLocale {
            locale: artifact.locale.clone(),
        });
    }
    let options = project_locale_options(&artifact.locale, &project.options);
    let Some(rules) = &options.plural_rules else {
        return Ok(None);
    };
    let categories = rules
        .categories
        .iter()
        .filter(|category| category.category != "other")
        .map(|category| category.category.clone())
        .collect::<BTreeSet<_>>();
    if categories.is_empty() {
        return Ok(None);
    }
    let mut demands = BTreeMap::new();
    let mut unions = BTreeSet::<BTreeSet<String>>::new();
    let mut uses_plural = false;
    for message in messages {
        let closure = project.message_dependency_closure(&artifact.locale, message)?;
        uses_plural |= linguini_codegen::plural_required(closure.schema(), closure.locale_module());
        let demand = match plural_category_demand(closure.schema(), closure.locale_module()) {
            PluralCategoryDemand::Full => None,
            PluralCategoryDemand::Categories(demand) => {
                let demand = demand
                    .intersection(&categories)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if demand == categories {
                    None
                } else {
                    Some(demand)
                }
            }
        };
        if let Some(demand) = &demand {
            let additions = unions
                .iter()
                .map(|prior| prior.union(demand).cloned().collect())
                .chain(std::iter::once(demand.clone()))
                .collect::<Vec<_>>();
            unions.extend(additions);
        }
        demands.insert(
            message.clone(),
            demand.map(|categories| categories.into_iter().collect()),
        );
    }
    if !uses_plural {
        return Ok(None);
    }
    let variants = unions
        .into_iter()
        .filter(|demand| *demand != categories)
        .map(|demand| {
            let selected = demand.iter().cloned().collect::<Vec<_>>();
            let stem = if selected.is_empty() {
                "fallback".to_owned()
            } else {
                selected.join("-")
            };
            let export = super::ValueExport::plural(&options.plural_function);
            let mut contents = crate::plural::generate_plural_function_for_demand(
                &export.binding,
                rules,
                &PluralCategoryDemand::Categories(demand),
            );
            contents.push_str(&export.statement(false));
            EcmaPluralRuntimeVariant {
                categories: selected,
                file: EcmaGeneratedFile {
                    path: format!("bundler/plurals/{}/{stem}.js", artifact.locale),
                    contents,
                },
            }
        })
        .collect();
    Ok(Some(EcmaPluralRuntimePlan {
        binding: options.plural_function,
        categories: categories.into_iter().collect(),
        messages: demands,
        variants,
    }))
}
