use std::collections::BTreeMap;

use linguini_ir::{IrBranchPattern, IrMessage, IrModule};

use super::expr::{
    branch_switch, form_object, is_static_text, text_expression, text_expression_with_context,
};
use super::names::{
    escape_comment, escape_string, function_name, property_key, safe_identifier, string_literal,
    ts_type,
};
use super::TypeScriptOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExports {
    pub top_level: Vec<String>,
    pub groups: Vec<String>,
}

pub fn emit_imports(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    if !module.forms.is_empty() {
        if let Some(path) = &options.plural_import {
            output.push_str(&format!(
                "import {{ {} }} from \"{}\";\n\n",
                options.plural_function,
                escape_string(path)
            ));
        }
    }
}

pub fn emit_enums(module: &IrModule, output: &mut String) {
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

pub fn emit_type_aliases(module: &IrModule, output: &mut String) {
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

pub fn emit_forms(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    for form in &module.forms {
        output.push_str(&format!("const {}Forms = {{\n", form.name));
        for variant in &form.variants {
            if let Some(selector) = &variant.selector {
                output.push_str(&format!(
                    "  {}: ({}: string) => {},\n",
                    property_key(&variant.name),
                    safe_identifier(selector),
                    branch_switch(&safe_identifier(selector), &variant.entries)
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

pub fn emit_local_functions(module: &IrModule, output: &mut String) {
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

pub fn emit_messages(schema: &IrModule, locale: &IrModule, output: &mut String) -> ModuleExports {
    let grouped = group_messages(schema);
    let mut exports = ModuleExports {
        top_level: Vec::new(),
        groups: Vec::new(),
    };

    for signature in &schema.messages {
        if signature.name.contains('.') {
            continue;
        }
        if emit_message_function(signature, locale, output) {
            exports.top_level.push(function_name(&signature.name));
        }
    }

    for (group, messages) in grouped {
        emit_group_object(&group, &messages, locale, output);
        exports.groups.push(group);
    }

    exports
}

pub fn emit_locale_facade(
    exports: &ModuleExports,
    options: &TypeScriptOptions,
    output: &mut String,
) {
    output.push_str(&format!("export const {} = {{\n", options.locale));
    for name in exports.top_level.iter().chain(exports.groups.iter()) {
        output.push_str(&format!("  {name},\n"));
    }
    output.push_str("} as const;\n\n");
    output.push_str(&format!(
        "export const locales = {{ {} }} as const;\n\n",
        options.locale
    ));
    output.push_str("export type Locale = keyof typeof locales;\n\n");
    output.push_str("export function createLinguini(locale: Locale): (typeof locales)[Locale] {\n");
    output.push_str("  return locales[locale];\n");
    output.push_str("}\n");
}

pub fn emit_branch_helper(module: &IrModule, output: &mut String) {
    if module.forms.is_empty() {
        return;
    }

    output.push_str(
        "\nfunction selectBranch(key: string, branches: Record<string, string>): string {\n",
    );
    output.push_str("  return branches[key] ?? branches.other ?? \"\";\n");
    output.push_str("}\n");
}

fn emit_message_function(signature: &IrMessage, locale: &IrModule, output: &mut String) -> bool {
    let Some(implementation) = message_implementation(locale, &signature.name) else {
        return false;
    };
    for doc in &signature.docs {
        output.push_str(&format!("/** {} */\n", escape_comment(doc)));
    }
    let params = signature_params(signature);
    let body = message_body(signature, implementation);
    output.push_str(&format!(
        "export function {}({params}): string {{\n  return {body};\n}}\n\n",
        function_name(&signature.name)
    ));
    true
}

fn emit_group_object(
    group: &str,
    signatures: &[IrMessage],
    locale: &IrModule,
    output: &mut String,
) {
    output.push_str(&format!("export const {group} = {{\n"));
    for signature in signatures {
        let property = signature.name.split('.').nth(1).unwrap_or(&signature.name);
        if let Some(implementation) = message_implementation(locale, &signature.name) {
            output.push_str(&format!(
                "  {}: {},\n",
                property_key(property),
                group_property_value(signature, implementation)
            ));
        }
    }
    output.push_str("} as const;\n\n");
}

fn group_property_value(signature: &IrMessage, implementation: &IrMessage) -> String {
    let Some(body) = &implementation.body else {
        return "\"\"".to_owned();
    };
    if signature.parameters.is_empty() && is_static_text(body) {
        text_expression(body)
    } else {
        format!(
            "({}) => {}",
            signature_params(signature),
            message_body(signature, implementation)
        )
    }
}

fn message_body(signature: &IrMessage, implementation: &IrMessage) -> String {
    let context = signature
        .parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    implementation
        .body
        .as_ref()
        .map(|body| text_expression_with_context(body, &context))
        .unwrap_or_else(|| "\"\"".to_owned())
}

fn signature_params(signature: &IrMessage) -> String {
    signature
        .parameters
        .iter()
        .map(|parameter| format!("{}: {}", parameter.name, ts_type(&parameter.ty)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn group_messages(module: &IrModule) -> BTreeMap<String, Vec<IrMessage>> {
    let mut grouped = BTreeMap::new();
    for message in &module.messages {
        if let Some((group, _)) = message.name.split_once('.') {
            grouped
                .entry(group.to_owned())
                .or_insert_with(Vec::new)
                .push(message.clone());
        }
    }
    grouped
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
