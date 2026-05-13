use std::collections::BTreeMap;

use linguini_ir::{IrBranch, IrExpression, IrFormEntry, IrText, IrTextPart, IrValue};

use super::names::{property_key, string_literal};
use super::TypeScriptOptions;

pub fn form_object(entries: &[IrFormEntry], options: &TypeScriptOptions) -> String {
    let fields = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute { name, value } => Some(format!(
                "{}: {}",
                property_key(name),
                value_expression(value, options)
            )),
            IrFormEntry::Branch(_) => None,
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {fields} }}")
}

pub fn value_expression(value: &IrValue, options: &TypeScriptOptions) -> String {
    match value {
        IrValue::Text(text) => text_expression(text),
        IrValue::Map(branches) => map_expression(branches, options),
        IrValue::Object(entries) => form_object(entries, options),
    }
}

pub fn map_expression(branches: &[IrBranch], options: &TypeScriptOptions) -> String {
    let items = branch_items(branches);
    format!(
        "(value: number | string) => selectBranch({}(value), {{ {items} }})",
        options.plural_function
    )
}

pub fn branch_switch(selector: &str, entries: &[IrFormEntry]) -> String {
    let branches = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Branch(branch) => Some(branch),
            IrFormEntry::Attribute { .. } => None,
        })
        .cloned()
        .collect::<Vec<_>>();
    format!(
        "selectBranch({selector}, {{ {} }})",
        branch_items(&branches)
    )
}

pub fn text_expression(text: &IrText) -> String {
    text_expression_with_context(text, &BTreeMap::new())
}

pub fn text_expression_with_context(text: &IrText, context: &BTreeMap<String, String>) -> String {
    let parts = text
        .parts
        .iter()
        .map(|part| match part {
            IrTextPart::Text(raw) => string_literal(raw),
            IrTextPart::Placeholder(expression) => expression_string(expression, context),
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "\"\"".to_owned()
    } else {
        parts.join(" + ")
    }
}

pub fn is_static_text(text: &IrText) -> bool {
    matches!(text.parts.as_slice(), [IrTextPart::Text(_)])
}

fn branch_items(branches: &[IrBranch]) -> String {
    branches
        .iter()
        .map(|branch| {
            let key = branch.keys.first().map(String::as_str).unwrap_or("other");
            format!("{}: {}", property_key(key), text_expression(&branch.value))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn expression_string(expression: &IrExpression, context: &BTreeMap<String, String>) -> String {
    format!("String({})", expression_value(expression, context))
}

fn expression_value(expression: &IrExpression, context: &BTreeMap<String, String>) -> String {
    if expression.path.is_empty() {
        return "\"\"".to_owned();
    }

    if !expression.arguments.is_empty() {
        if let [root, property] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{ty}Forms[{root}].{property}({})",
                    expression
                        .arguments
                        .iter()
                        .map(|argument| expression_value(argument, context))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        return format!(
            "{}({})",
            expression.path.join("."),
            expression
                .arguments
                .iter()
                .map(|argument| expression_value(argument, context))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    match expression.path.as_slice() {
        [root, property] => context.get(root).map_or_else(
            || expression.path.join("."),
            |ty| format!("{ty}Forms[{root}].{property}"),
        ),
        [root, property, rest @ ..] => {
            let suffix = rest
                .iter()
                .map(|part| format!(".{part}"))
                .collect::<String>();
            context.get(root).map_or_else(
                || expression.path.join("."),
                |ty| format!("{ty}Forms[{root}].{property}{suffix}"),
            )
        }
        _ => expression.path.join("."),
    }
}
