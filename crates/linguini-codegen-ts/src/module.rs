use std::collections::BTreeMap;

use linguini_ir::{
    IrBranch, IrBranchPattern, IrExpression, IrFormEntry, IrMessage, IrModule, IrText, IrTextPart,
    IrValue,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptOptions {
    pub plural_function: String,
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            plural_function: "plural".to_owned(),
        }
    }
}

pub fn generate_typescript_module(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
) -> String {
    let mut output = String::new();
    emit_enums(schema, &mut output);
    emit_type_aliases(schema, &mut output);
    emit_forms(locale, options, &mut output);
    emit_local_functions(locale, &mut output);
    emit_messages(schema, locale, &mut output);
    emit_branch_helper(locale, &mut output);
    output
}

fn emit_enums(module: &IrModule, output: &mut String) {
    for item in &module.enums {
        for doc in &item.docs {
            output.push_str(&format!("/** {} */\n", escape_comment(doc)));
        }
        let variants = item
            .variants
            .iter()
            .map(|variant| format!("\"{}\"", escape_string(variant)))
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!("export type {} = {variants};\n\n", item.name));
    }
}

fn emit_type_aliases(module: &IrModule, output: &mut String) {
    for item in &module.type_aliases {
        for doc in &item.docs {
            output.push_str(&format!("/** {} */\n", escape_comment(doc)));
        }
        output.push_str(&format!(
            "export type {} = {};\n\n",
            item.name,
            ts_type(&item.target)
        ));
    }
}

fn emit_forms(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    for form in &module.forms {
        output.push_str(&format!("const {}Forms = {{\n", form.name));
        for variant in &form.variants {
            if let Some(selector) = &variant.selector {
                output.push_str(&format!(
                    "  {}: ({}: string) => {},\n",
                    property_key(&variant.name),
                    safe_identifier(selector),
                    branch_switch(selector, &variant.entries, options)
                ));
            } else {
                output.push_str(&format!(
                    "  {}: {},\n",
                    property_key(&variant.name),
                    form_object(&variant.entries, options)
                ));
            }
        }
        output.push_str("} as const;\n\n");
    }
}

fn emit_local_functions(module: &IrModule, output: &mut String) {
    for function in &module.functions {
        let params = function
            .parameters
            .iter()
            .map(|name| format!("{}: string", safe_identifier(name)))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "function {}({params}): string {{\n",
            function.name
        ));
        for branch in &function.branches {
            match &branch.pattern {
                IrBranchPattern::Names(names) => output.push_str(&format!(
                    "  if ({}) return {};\n",
                    tuple_condition(&function.parameters, names),
                    text_expression(&branch.value)
                )),
                IrBranchPattern::Else => {
                    output.push_str(&format!("  return {};\n", text_expression(&branch.value)));
                }
            }
        }
        if !function
            .branches
            .iter()
            .any(|branch| branch.pattern == IrBranchPattern::Else)
        {
            output.push_str("  return \"\";\n");
        }
        output.push_str("}\n\n");
    }
}

fn emit_messages(schema: &IrModule, locale: &IrModule, output: &mut String) {
    for signature in &schema.messages {
        if let Some(implementation) = message_implementation(locale, &signature.name) {
            for doc in &signature.docs {
                output.push_str(&format!("/** {} */\n", escape_comment(doc)));
            }
            let params = signature
                .parameters
                .iter()
                .map(|parameter| format!("{}: {}", parameter.name, ts_type(&parameter.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let context = signature
                .parameters
                .iter()
                .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                .collect::<BTreeMap<_, _>>();
            let body = implementation
                .body
                .as_ref()
                .map(|body| text_expression_with_context(body, &context))
                .unwrap_or_else(|| "\"\"".to_owned());
            output.push_str(&format!(
                "export function {}({params}): string {{\n  return {body};\n}}\n\n",
                function_name(&signature.name)
            ));
        }
    }
}

fn form_object(entries: &[IrFormEntry], options: &TypeScriptOptions) -> String {
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

fn value_expression(value: &IrValue, options: &TypeScriptOptions) -> String {
    match value {
        IrValue::Text(text) => text_expression(text),
        IrValue::Map(branches) => map_expression(branches, options),
        IrValue::Object(entries) => form_object(entries, options),
    }
}

fn map_expression(branches: &[IrBranch], options: &TypeScriptOptions) -> String {
    let items = branches
        .iter()
        .map(|branch| {
            let key = branch.keys.first().map(String::as_str).unwrap_or("other");
            format!("{}: {}", property_key(key), text_expression(&branch.value))
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "(value: number | string) => selectBranch({}(value), {{ {items} }})",
        options.plural_function
    )
}

fn branch_switch(selector: &str, entries: &[IrFormEntry], _options: &TypeScriptOptions) -> String {
    let branches = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Branch(branch) => Some(branch),
            IrFormEntry::Attribute { .. } => None,
        })
        .map(|branch| {
            let key = branch.keys.first().map(String::as_str).unwrap_or("other");
            format!("{}: {}", property_key(key), text_expression(&branch.value))
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "selectBranch({}, {{ {branches} }})",
        safe_identifier(selector)
    )
}

fn emit_branch_helper(module: &IrModule, output: &mut String) {
    if module.forms.is_empty() {
        return;
    }

    output.push_str(
        "function selectBranch(key: string, branches: Record<string, string>): string {\n",
    );
    output.push_str("  return branches[key] ?? branches.other ?? \"\";\n");
    output.push_str("}\n");
}

fn text_expression(text: &IrText) -> String {
    text_expression_with_context(text, &BTreeMap::new())
}

fn text_expression_with_context(text: &IrText, context: &BTreeMap<String, String>) -> String {
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

fn message_implementation<'a>(module: &'a IrModule, name: &str) -> Option<&'a IrMessage> {
    module.messages.iter().find(|message| message.name == name)
}

fn tuple_condition(parameters: &[String], names: &[String]) -> String {
    parameters
        .iter()
        .zip(names)
        .map(|(parameter, name)| format!("{parameter} === {}", string_literal(name)))
        .collect::<Vec<_>>()
        .join(" && ")
}

fn ts_type(name: &str) -> String {
    match name {
        "String" | "Date" => "string".to_owned(),
        "Number" | "Decimal" => "number".to_owned(),
        "Boolean" => "boolean".to_owned(),
        other => other.to_owned(),
    }
}

fn function_name(name: &str) -> String {
    name.replace('.', "_")
}

fn safe_identifier(name: &str) -> String {
    name.replace('-', "_")
}

fn property_key(name: &str) -> String {
    if name
        .bytes()
        .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    {
        name.to_owned()
    } else {
        string_literal(name)
    }
}

fn string_literal(value: &str) -> String {
    format!("\"{}\"", escape_string(value))
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn escape_comment(value: &str) -> String {
    value.replace("*/", "* /")
}
