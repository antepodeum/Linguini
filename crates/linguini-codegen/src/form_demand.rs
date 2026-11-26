//! Conservative projection of form attributes without changing variant lookup shape.

use std::collections::{BTreeMap, BTreeSet};

use linguini_ir::{IrForm, IrFormEntry, IrModule, IrModuleBuilder, IrTextPart, IrValue};

#[derive(Default)]
pub(crate) struct FormDemand {
    paths: BTreeMap<String, BTreeSet<Vec<String>>>,
}

impl FormDemand {
    /// Returns a newly requested projection. An empty path retains the whole form.
    pub(crate) fn request(&mut self, form: &IrForm, path: &[String]) -> Option<IrForm> {
        let path = if form
            .variants
            .iter()
            .all(|variant| contains_path(&variant.entries, path))
        {
            path.to_vec()
        } else {
            Vec::new()
        };
        let paths = self.paths.entry(form.name.clone()).or_default();
        if paths.contains(&Vec::new()) || !paths.insert(path.clone()) {
            return None;
        }
        Some(project_form(form, &BTreeSet::from([path])))
    }

    pub(crate) fn project(&self, module: &IrModule) -> IrModule {
        IrModuleBuilder::seeded(module)
            .update_forms(|form| {
                if let Some(paths) = self.paths.get(&form.name) {
                    *form = project_form(form, paths);
                }
            })
            .build()
            .expect("form attribute projection preserves declaration identities")
    }
}

fn contains_path(entries: &[IrFormEntry], path: &[String]) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return true;
    };
    entries.iter().any(|entry| match entry {
        IrFormEntry::Attribute { name, value, .. } if name == first => {
            rest.is_empty()
                || matches!(value, IrValue::Object(children) if contains_path(children, rest))
        }
        _ => false,
    })
}

fn project_form(form: &IrForm, paths: &BTreeSet<Vec<String>>) -> IrForm {
    let mut form = form.clone();
    for variant in &mut form.variants {
        variant.entries = project_entries(&variant.entries, paths);
    }
    form
}

fn project_entries(entries: &[IrFormEntry], paths: &BTreeSet<Vec<String>>) -> Vec<IrFormEntry> {
    if paths.contains(&Vec::new()) || has_callable_attributes(entries) {
        return entries.to_vec();
    }
    entries
        .iter()
        .filter_map(|entry| {
            let IrFormEntry::Attribute {
                name,
                parameters,
                value,
            } = entry
            else {
                // Preserve callable rows and their function prototype.
                return Some(entry.clone());
            };
            if has_eager_expression(value) {
                return Some(entry.clone());
            }
            let children = paths
                .iter()
                .filter_map(|path| {
                    let (first, rest) = path.split_first()?;
                    (first == name).then(|| rest.to_vec())
                })
                .collect::<BTreeSet<_>>();
            if children.is_empty() {
                return None;
            }
            let value = match value {
                IrValue::Object(entries) => IrValue::Object(project_entries(entries, &children)),
                _ => value.clone(),
            };
            Some(IrFormEntry::Attribute {
                name: name.clone(),
                parameters: parameters.clone(),
                value,
            })
        })
        .collect()
}

fn has_eager_expression(value: &IrValue) -> bool {
    match value {
        IrValue::Text(text) => text
            .parts
            .iter()
            .any(|part| !matches!(part, IrTextPart::Text(_))),
        // Map bodies execute only when their generated function is called.
        IrValue::Map(_) => false,
        IrValue::Object(entries) => entries_have_observable_initialization(entries),
    }
}

// Attaching attributes to a callable row can require observable target operations.
fn has_callable_attributes(entries: &[IrFormEntry]) -> bool {
    entries
        .iter()
        .any(|entry| matches!(entry, IrFormEntry::Branch(_)))
        && entries
            .iter()
            .any(|entry| matches!(entry, IrFormEntry::Attribute { .. }))
}

fn entries_have_observable_initialization(entries: &[IrFormEntry]) -> bool {
    has_callable_attributes(entries)
        || entries.iter().any(|entry| match entry {
            IrFormEntry::Attribute { value, .. } => has_eager_expression(value),
            IrFormEntry::Branch(_) => false,
        })
}

pub(crate) fn form_has_observable_initialization(form: &IrForm) -> bool {
    form.variants
        .iter()
        .any(|variant| entries_have_observable_initialization(&variant.entries))
}

/// Pure named attributes that can be emitted independently while preserving row lookup.
/// Callable rows and eager initializers retain a single owning form instead.
pub fn independent_form_attribute_names(form: &IrForm) -> Option<BTreeSet<&str>> {
    if form_has_observable_initialization(form)
        || form.variants.iter().any(|variant| {
            variant
                .entries
                .iter()
                .any(|entry| matches!(entry, IrFormEntry::Branch(_)))
        })
    {
        return None;
    }
    let names = form
        .variants
        .iter()
        .flat_map(|variant| &variant.entries)
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute { name, .. } => Some(name.as_str()),
            IrFormEntry::Branch(_) => None,
        })
        .collect::<BTreeSet<_>>();
    (!names.is_empty()).then_some(names)
}
