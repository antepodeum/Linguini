use std::collections::BTreeMap;

use linguini_ir::{IrBranch, IrExpression, IrFormEntry, IrFormatter, IrText, IrTextPart, IrValue};

use super::names::{escape_string, property_key, string_literal};
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
        IrValue::Text(text) => text_expression(text, options),
        IrValue::Map(branches) => map_expression(branches, options),
        IrValue::Object(entries) => form_object(entries, options),
    }
}

pub fn map_expression(branches: &[IrBranch], options: &TypeScriptOptions) -> String {
    let items = branch_items(branches, options);
    format!(
        "(value: number | string) => selectBranch({}(value), {{ {items} }})",
        options.plural_function
    )
}

pub fn text_expression(text: &IrText, options: &TypeScriptOptions) -> String {
    text_expression_with_context(text, &BTreeMap::new(), options)
}

pub fn text_expression_with_context(
    text: &IrText,
    context: &BTreeMap<String, String>,
    options: &TypeScriptOptions,
) -> String {
    let parts = text
        .parts
        .iter()
        .map(|part| match part {
            IrTextPart::Text(raw) => string_literal(raw),
            IrTextPart::Placeholder(expression) => expression_string(expression, context, options),
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

fn branch_items(branches: &[IrBranch], options: &TypeScriptOptions) -> String {
    branches
        .iter()
        .map(|branch| {
            let key = branch.keys.first().map(String::as_str).unwrap_or("_");
            format!(
                "{}: {}",
                property_key(key),
                text_expression(&branch.value, options)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn expression_string(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    options: &TypeScriptOptions,
) -> String {
    let value = expression_value(expression, context, options);
    let formatted = apply_formatters(value, &expression.formatters, options);
    format!("String({formatted})")
}

fn expression_value(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    options: &TypeScriptOptions,
) -> String {
    if expression.path.is_empty() {
        return "\"\"".to_owned();
    }

    if !expression.arguments.is_empty() {
        if let [root] = expression.path.as_slice() {
            if root == "plural" {
                return format!(
                    "{}({})",
                    options.plural_function,
                    expression
                        .arguments
                        .iter()
                        .map(|argument| expression_value(argument, context, options))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{ty}Forms[{root}]({})",
                    expression
                        .arguments
                        .iter()
                        .map(|argument| expression_value(argument, context, options))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root, property] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{ty}Forms[{root}].{property}({})",
                    expression
                        .arguments
                        .iter()
                        .map(|argument| expression_value(argument, context, options))
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
                .map(|argument| expression_value(argument, context, options))
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

fn apply_formatters(
    value: String,
    formatters: &[IrFormatter],
    options: &TypeScriptOptions,
) -> String {
    formatters.iter().fold(value, |current, formatter| {
        let formatter_options = formatter_options(formatter);
        let locale = string_literal(&options.locale);
        match formatter.name.as_str() {
            "currency" => format!("formatCurrency({current}, {locale}, {formatter_options})"),
            "date" => format!("formatDate({current}, {locale}, {formatter_options})"),
            _ => current,
        }
    })
}

fn formatter_options(formatter: &IrFormatter) -> String {
    let items = formatter
        .arguments
        .iter()
        .map(|argument| {
            format!(
                "{}: \"{}\"",
                property_key(&argument.name),
                escape_string(&argument.value)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {items} }}")
}
