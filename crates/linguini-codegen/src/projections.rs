//! Backend-independent message selection and namespace projections.

use linguini_ir::{IrModule, IrModuleBuilder};
use std::collections::BTreeSet;

/// Select already validated dotted message/group paths; an empty list selects all.
/// Retains declaration order, group ancestors/descendants, and global declarations.
pub fn select_schema_paths(schema: &IrModule, selected_paths: &[String]) -> IrModule {
    if selected_paths.is_empty() {
        return schema.clone();
    }

    let is_selected = |name: &str| {
        selected_paths.iter().any(|selected| {
            selected == name
                || name
                    .strip_prefix(selected.as_str())
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    };
    let retained_message_names = schema
        .messages()
        .iter()
        .map(|message| message.name.as_str())
        .filter(|name| is_selected(name))
        .collect::<Vec<_>>();
    IrModuleBuilder::seeded(schema)
        .retain_messages(|message| is_selected(&message.name))
        .retain_groups(|group| {
            selected_paths.iter().any(|selected| {
                is_path_or_descendant(&group.name, selected)
                    || is_path_or_descendant(selected, &group.name)
            }) || retained_message_names
                .iter()
                .any(|message| is_path_or_descendant(message, &group.name))
        })
        .build()
        .expect("tree-shaken schema projection preserves unique declaration names")
}

/// Retain locale globals and only messages/groups belonging to the projected schema.
pub fn locale_module_for_schema(locale: &IrModule, schema: &IrModule) -> IrModule {
    let schema_groups = schema
        .groups()
        .iter()
        .map(|group| group.name.as_str())
        .collect::<BTreeSet<_>>();
    IrModuleBuilder::seeded(locale)
        .retain_messages(|message| {
            schema
                .messages()
                .iter()
                .any(|schema_message| schema_message.name == message.name)
        })
        .retain_groups(|group| schema_groups.contains(group.name.as_str()))
        .build()
        .expect("schema-projected locale preserves unique declaration names")
}

/// Top-level namespaces containing messages, sorted and deduplicated.
/// Empty declared groups reserve namespaces without creating runtime artifacts.
pub fn top_level_message_namespaces(module: &IrModule) -> Vec<String> {
    let mut namespaces = module
        .messages()
        .iter()
        .filter_map(|message| message.name.split_once('.').map(|(namespace, _)| namespace))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    namespaces.sort();
    namespaces.dedup();
    namespaces
}

/// Retain messages and group metadata below one namespace, preserving globals.
pub fn namespace_module(module: &IrModule, namespace: &str) -> IrModule {
    let prefix = format!("{namespace}.");
    IrModuleBuilder::seeded(module)
        .retain_messages(|message| message.name.starts_with(&prefix))
        .retain_groups(|group| group.name == namespace || group.name.starts_with(&prefix))
        .build()
        .expect("namespace projection preserves unique declaration names")
}

/// Retain root messages and root group metadata, preserving globals.
pub fn root_module(module: &IrModule) -> IrModule {
    IrModuleBuilder::seeded(module)
        .retain_messages(|message| !message.name.contains('.'))
        .retain_groups(|group| !group.name.contains('.'))
        .build()
        .expect("root projection preserves unique declaration names")
}

fn is_path_or_descendant(path: &str, parent: &str) -> bool {
    path == parent
        || path
            .strip_prefix(parent)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

#[cfg(test)]
mod tests;
