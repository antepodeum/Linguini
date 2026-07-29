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
        qualify_function_branches(&mut item.branches, &namespace_parts, &declaration_roots);
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
    for entry in entries {
        match entry {
            IrFormEntry::Attribute {
                parameters, value, ..
            } => {
                for parameter in parameters {
                    qualify_type(&mut parameter.ty, &namespace.join("."));
                }
                qualify_value(value, namespace, declaration_roots);
            }
            IrFormEntry::Branch(branch) => {
                qualify_text(&mut branch.value, namespace, declaration_roots);
            }
        }
    }
}

fn qualify_value(value: &mut IrValue, namespace: &[String], declaration_roots: &BTreeSet<String>) {
    match value {
        IrValue::Text(text) => qualify_text(text, namespace, declaration_roots),
        IrValue::Map(branches) => {
            for branch in branches {
                qualify_text(&mut branch.value, namespace, declaration_roots);
            }
        }
        IrValue::Object(entries) => qualify_entries(entries, namespace, declaration_roots),
    }
}

fn qualify_function_branches(
    branches: &mut [IrFunctionBranch],
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
) {
    for branch in branches {
        match &mut branch.value {
            IrFunctionBranchValue::Text(text) => {
                qualify_text(text, namespace, declaration_roots);
            }
            IrFunctionBranchValue::Dispatch(children) => {
                qualify_function_branches(children, namespace, declaration_roots);
            }
        }
    }
}

fn qualify_text(text: &mut IrText, namespace: &[String], declaration_roots: &BTreeSet<String>) {
    for part in &mut text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            qualify_expression(expression, namespace, declaration_roots);
        }
    }
}

fn qualify_expression(
    expression: &mut IrExpression,
    namespace: &[String],
    declaration_roots: &BTreeSet<String>,
) {
    if let Some(root) = expression
        .path
        .first_mut()
        .filter(|root| declaration_roots.contains(root.as_str()))
    {
        *root = qualified_name(&namespace.join("."), root);
    }

    for argument in &mut expression.arguments {
        qualify_expression(argument, namespace, declaration_roots);
    }
}

fn qualified_name(namespace: &str, name: &str) -> String {
    if name == namespace || name.starts_with(&format!("{namespace}.")) {
        name.to_owned()
    } else {
        format!("{namespace}.{name}")
    }
}
