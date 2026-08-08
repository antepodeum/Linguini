use std::collections::BTreeSet;

use crate::{
    IrExpression, IrFormEntry, IrFunctionBranch, IrFunctionBranchValue, IrModule, IrText,
    IrTextPart, IrValue,
};
use linguini_core::TypeKind;

/// Qualifies every declaration and every reference to a module-local declaration.
///
/// Filesystem namespaces are a project concern, so syntax lowering intentionally produces names
/// relative to one source file. Project loaders must call this function before merging modules.
/// The operation qualifies provenance and type references as well as runtime declarations, keeping
/// validation, diagnostics, and code generation on the same canonical paths.
pub fn qualify_module(module: &mut IrModule, namespace: &str) {
    if namespace.is_empty() {
        return;
    }

    let namespace_parts = namespace
        .split('.')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if namespace_parts.is_empty() {
        return;
    }

    let declaration_roots = declaration_roots(module);
    for item in &mut module.enums {
        item.name = qualified_name(namespace, &item.name);
    }
    for item in &mut module.type_aliases {
        item.name = qualified_name(namespace, &item.name);
        qualify_type(&mut item.target, namespace);
    }
    for item in &mut module.variables {
        item.name = qualified_name(namespace, &item.name);
        qualify_text(&mut item.value, &namespace_parts, &declaration_roots);
    }
    for item in &mut module.messages {
        item.name = qualified_name(namespace, &item.name);
        for parameter in &mut item.parameters {
            qualify_type(&mut parameter.ty, namespace);
        }
        if let Some(body) = &mut item.body {
            qualify_text(body, &namespace_parts, &declaration_roots);
        }
    }
    for item in &mut module.groups {
        item.name = qualified_name(namespace, &item.name);
    }
    for item in &mut module.forms {
        item.name = qualified_name(namespace, &item.name);
        for variant in &mut item.variants {
            qualify_entries(&mut variant.entries, &namespace_parts, &declaration_roots);
        }
    }
    for item in &mut module.functions {
        item.name = qualified_name(namespace, &item.name);
        for parameter in &mut item.parameters {
            qualify_type(&mut parameter.ty, namespace);
        }
        let local_names = item
            .parameters
            .iter()
            .filter_map(|parameter| parameter.name.clone())
            .collect::<BTreeSet<_>>();
        qualify_function_branches_scoped(
            &mut item.branches,
            &namespace_parts,
            &declaration_roots,
            &local_names,
        );
    }
    for origin in &mut module.origins {
        origin.name = qualified_name(namespace, &origin.name);
    }
}

fn declaration_roots(module: &IrModule) -> BTreeSet<String> {
    module
        .enums
        .iter()
        .map(|item| item.name.as_str())
        .chain(module.type_aliases.iter().map(|item| item.name.as_str()))
        .chain(module.variables.iter().map(|item| item.name.as_str()))
        .chain(module.messages.iter().map(|item| item.name.as_str()))
        .chain(module.groups.iter().map(|item| item.name.as_str()))
        .chain(module.forms.iter().map(|item| item.name.as_str()))
        .chain(module.functions.iter().map(|item| item.name.as_str()))
        .filter_map(|name| name.split('.').next())
        .map(str::to_owned)
        .collect()
}

fn qualify_type(ty: &mut String, namespace: &str) {
    if TypeKind::from_name(ty).is_none() && ty != "Plural" {
        *ty = qualified_name(namespace, ty);
    }
}

fn qualify_entries(
    entries: &mut [IrFormEntry],
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
) {
    qualify_entries_scoped(entries, namespace, declaration_roots, &BTreeSet::new());
}

fn qualify_entries_scoped(
    entries: &mut [IrFormEntry],
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) {
    for entry in entries {
        match entry {
            IrFormEntry::Attribute {
                parameters, value, ..
            } => {
                let mut attribute_names = local_names.clone();
                for parameter in parameters {
                    qualify_type(&mut parameter.ty, &namespace.join("."));
                    if let Some(name) = &parameter.name {
                        attribute_names.insert(name.clone());
                    }
                }
                qualify_value_scoped(value, namespace, declaration_roots, &attribute_names);
            }
            IrFormEntry::Branch(branch) => {
                qualify_text_scoped(&mut branch.value, namespace, declaration_roots, local_names);
            }
        }
    }
}

fn qualify_value_scoped(
    value: &mut IrValue,
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) {
    match value {
        IrValue::Text(text) => {
            qualify_text_scoped(text, namespace, declaration_roots, local_names);
        }
        IrValue::Map(branches) => {
            for branch in branches {
                qualify_text_scoped(&mut branch.value, namespace, declaration_roots, local_names);
            }
        }
        IrValue::Object(entries) => {
            qualify_entries_scoped(entries, namespace, declaration_roots, local_names);
        }
    }
}

fn qualify_function_branches_scoped(
    branches: &mut [IrFunctionBranch],
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) {
    for branch in branches {
        match &mut branch.value {
            IrFunctionBranchValue::Text(text) => {
                qualify_text_scoped(text, namespace, declaration_roots, local_names);
            }
            IrFunctionBranchValue::Dispatch(children) => {
                qualify_function_branches_scoped(
                    children,
                    namespace,
                    declaration_roots,
                    local_names,
                );
            }
        }
    }
}

fn qualify_text(text: &mut IrText, namespace: &[String], declaration_roots: &BTreeSet<String>) {
    qualify_text_scoped(text, namespace, declaration_roots, &BTreeSet::new());
}

fn qualify_text_scoped(
    text: &mut IrText,
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) {
    for part in &mut text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            qualify_expression(expression, namespace, declaration_roots, local_names);
        }
    }
}

fn qualify_expression(
    expression: &mut IrExpression,
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) {
    if let Some(root) = expression.path.first_mut().filter(|root| {
        declaration_roots.contains(root.as_str()) && !local_names.contains(root.as_str())
    }) {
        *root = qualified_name(&namespace.join("."), root);
    }

    for argument in &mut expression.arguments {
        qualify_expression(argument, namespace, declaration_roots, local_names);
    }
    if let crate::IrExpressionKind::InlineFunction { inputs, branches } = &mut expression.kind {
        let mut inline_names = local_names.clone();
        for input in inputs {
            let (value, binding_name) = match input {
                crate::IrInlineFunctionInput::Binding { name, value, .. } => {
                    (value, Some(name.clone()))
                }
                crate::IrInlineFunctionInput::Selector { value, .. } => (value, None),
            };
            qualify_expression(value, namespace, declaration_roots, local_names);
            if let Some(name) = binding_name {
                inline_names.insert(name);
            }
        }
        qualify_function_branches_scoped(branches, namespace, declaration_roots, &inline_names);
    }
}

fn qualified_name(namespace: &str, name: &str) -> String {
    // Lowering emits names relative to one source file.  Always prepend the
    // filesystem namespace here: a local declaration may itself be named
    // `shop` (or `shop.checkout`) and must remain distinguishable from the
    // project namespace `shop` (or `shop.checkout`).  Qualification is only
    // performed on freshly lowered modules, so an idempotence shortcut would
    // conflate those two identities.
    format!("{namespace}.{name}")
}
