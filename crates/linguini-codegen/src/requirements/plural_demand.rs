//! Category distinctions observed by a closed set of message implementations.

use super::*;
use crate::{form_dispatch_plan, function_dispatch_plan, Decision};

/// Public category values and unmatched dispatch errors require the full classifier.
/// Otherwise, unlisted categories all take a consumer's fallback branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluralCategoryDemand {
    Full,
    Categories(BTreeSet<String>),
}

impl Default for PluralCategoryDemand {
    fn default() -> Self {
        Self::Categories(BTreeSet::new())
    }
}

impl PluralCategoryDemand {
    /// Combine consumers before selecting predicates, so they share one classifier.
    pub fn merge(&mut self, other: Self) {
        match (&mut *self, other) {
            (Self::Full, _) => {}
            (_, Self::Full) => *self = Self::Full,
            (Self::Categories(categories), Self::Categories(other)) => categories.extend(other),
        }
    }

    fn record<S>(&mut self, decision: &Decision<S, IrText>, is_plural: &impl Fn(&S) -> bool) {
        let Decision::Branch {
            selector,
            arms,
            fallback,
        } = decision
        else {
            return;
        };
        if is_plural(selector) {
            if fallback.is_none()
                || arms
                    .iter()
                    .any(|arm| arm.keys.iter().any(|key| key == "other"))
            {
                *self = Self::Full;
            } else {
                self.merge(Self::Categories(
                    arms.iter()
                        .flat_map(|arm| arm.keys.iter().cloned())
                        .collect(),
                ));
            }
        }
        for arm in arms {
            self.record(&arm.result, is_plural);
        }
        if let Some(fallback) = fallback {
            self.record(fallback, is_plural);
        }
    }

    fn entries(&mut self, entries: &[IrFormEntry]) {
        let branches = entries
            .iter()
            .filter_map(|entry| match entry {
                IrFormEntry::Branch(branch) => Some(branch.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !branches.is_empty() {
            self.record(&form_dispatch_plan(&branches, (), Clone::clone), &|_| true);
        }
        for entry in entries {
            match entry {
                IrFormEntry::Branch(branch) => {
                    if text_requires_plural(&branch.value, &BTreeMap::new()) {
                        *self = Self::Full;
                    }
                }
                IrFormEntry::Attribute {
                    parameters, value, ..
                } => {
                    let context = parameter_context(parameters);
                    match value {
                        IrValue::Object(entries) => self.entries(entries),
                        IrValue::Map(branches) => {
                            if parameters
                                .first()
                                .is_some_and(|parameter| parameter.ty == "Plural")
                            {
                                self.record(
                                    &form_dispatch_plan(branches, (), Clone::clone),
                                    &|_| true,
                                );
                            }
                            if branches
                                .iter()
                                .any(|branch| text_requires_plural(&branch.value, &context))
                            {
                                *self = Self::Full;
                            }
                        }
                        IrValue::Text(text) => {
                            if text_requires_plural(text, &context) {
                                *self = Self::Full;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Analyze the exact schema/locale closure supplied by the caller. Explicit `Plural()`
/// results and unlowered inline selectors conservatively retain public category semantics.
pub fn plural_category_demand(schema: &IrModule, locale: &IrModule) -> PluralCategoryDemand {
    let mut demand = PluralCategoryDemand::default();
    if locale
        .variables()
        .iter()
        .any(|variable| text_requires_plural(&variable.value, &BTreeMap::new()))
    {
        return PluralCategoryDemand::Full;
    }
    for message in locale.messages() {
        let context = schema
            .messages()
            .iter()
            .find(|signature| signature.name == message.name)
            .map(|signature| {
                signature
                    .parameters
                    .iter()
                    .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                    .collect()
            })
            .unwrap_or_default();
        if message
            .body
            .as_ref()
            .is_some_and(|body| text_requires_plural(body, &context))
        {
            return PluralCategoryDemand::Full;
        }
    }
    for function in locale.functions() {
        let context = parameter_context(&function.parameters);
        if function
            .branches
            .iter()
            .any(|branch| function_branch_requires_plural(branch, &context))
        {
            return PluralCategoryDemand::Full;
        }
        let plan = function_dispatch_plan(
            &function.parameters,
            &function.branches,
            |index| index,
            Clone::clone,
        );
        demand.record(&plan, &|index| {
            index.is_some_and(|index| function.parameters[index].ty == "Plural")
        });
    }
    for form in locale.forms() {
        for variant in &form.variants {
            demand.entries(&variant.entries);
        }
    }
    demand
}

#[cfg(test)]
mod tests;
