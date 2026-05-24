use std::collections::BTreeMap;

use linguini_core::{FormatterKind, TypeKind};
use linguini_ir::{
    IrFormatter, IrFormatterArgument, IrFunction, IrFunctionBranch, IrFunctionBranchValue,
    IrMessage, IrModule,
};

use super::expr::{
    form_object, formatter_data_declaration, is_static_text, text_expression,
    text_expression_with_context,
};
use super::formatters::module_uses_formatters;
use super::names::{
    escape_comment, escape_string, function_name, property_key, safe_identifier, string_literal,
    ts_type,
};
use super::templates::{render_template, SINGLE_INDEX_RUNTIME};
use super::tree::{nested_message_tree, MessageTree};
use super::TypeScriptOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExports {
    pub top_level: Vec<String>,
    pub groups: Vec<String>,
}

pub fn emit_imports(
    _schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: &mut String,
) {
    let uses_forms = !locale.forms.is_empty();
    let uses_dispatch = !locale.functions.is_empty();
    if uses_forms || uses_dispatch {
        let mut imports = Vec::new();
        if uses_forms || uses_dispatch {
            imports.push("selectBranch");
        }
        output.push_str(&format!(
            "import {{ {} }} from \"../shared\";\n",
            imports.join(", ")
        ));
        if options.plural_source.is_none() {
            if let Some(path) = &options.plural_import {
                output.push_str(&format!(
                    "import {{ {} }} from \"{}\";\n\n",
                    options.plural_function,
                    escape_string(path)
                ));
            } else {
                output.push('\n');
            }
        } else {
            output.push('\n');
        }
    }
}

pub fn emit_plural_helpers(options: &TypeScriptOptions, output: &mut String) {
    if let Some(source) = &options.plural_source {
        let source = source.replacen(
            &format!("export function {}", options.plural_function),
            &format!("function {}", options.plural_function),
            1,
        );
        output.push_str(source.trim_end());
        output.push_str("\n\n");
    }
}

pub fn emit_formatter_data(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: &mut String,
) {
    if module_uses_formatters(schema)
        || module_uses_formatters(locale)
        || schema_uses_auto_formatters(schema)
    {
        output.push_str(&formatter_data_declaration(&options.locale));
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
            output.push_str(&format!(
                "  {}: {},\n",
                property_key(&variant.name),
                form_object(&variant.entries, options)
            ));
        }
        output.push_str("} as const;\n\n");
    }
}

pub fn emit_local_functions(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    for function in &module.functions {
        let params = function_parameters(function)
            .iter()
            .map(|name| format!("{name}: string | number"))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "function {}({params}): string {{\n",
            function.name
        ));
        output.push_str(&format!(
            "  return {};\n",
            dispatch_expression(function, &function.branches, 0, options)
        ));
        output.push_str("}\n\n");
    }
}

pub fn emit_messages(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: &mut String,
) -> ModuleExports {
    let nested = nested_message_tree(schema);
    let mut exports = ModuleExports {
        top_level: Vec::new(),
        groups: Vec::new(),
    };

    for signature in &schema.messages {
        if signature.name.contains('.') {
            continue;
        }
        if emit_message_function(schema, signature, locale, options, output) {
            exports.top_level.push(function_name(&signature.name));
        }
    }

    for (group, messages) in nested.children {
        emit_message_object(schema, &group, &messages, locale, options, output);
        exports.groups.push(group);
    }

    exports
}

pub fn emit_index(options: &TypeScriptOptions, output: &mut String) {
    let locale = safe_identifier(&options.locale);
    let locale_path = escape_string(&options.locale);
    let locale_literal = string_literal(&options.locale);

    output.push_str(&render_template(
        SINGLE_INDEX_RUNTIME,
        &[
            ("LOCALE_IDENTIFIER", locale),
            ("LOCALE_PATH", locale_path),
            ("LOCALE_LITERAL", locale_literal),
        ],
    ));
}

fn emit_message_function(
    schema: &IrModule,
    signature: &IrMessage,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: &mut String,
) -> bool {
    let Some(implementation) = message_implementation(locale, &signature.name) else {
        return false;
    };
    for doc in &signature.docs {
        output.push_str(&format!("/** {} */\n", escape_comment(doc)));
    }
    let params = signature_params(signature);
    let body = message_body(schema, signature, implementation, options);
    output.push_str(&format!(
        "export function {}({params}): string {{\n  return {body};\n}}\n\n",
        function_name(&signature.name)
    ));
    true
}

fn emit_message_object(
    schema: &IrModule,
    name: &str,
    tree: &MessageTree,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: &mut String,
) {
    output.push_str(&format!("export const {name} = "));
    emit_object_literal(schema, tree, locale, options, 0, output);
    output.push_str(" as const;\n\n");
}

fn emit_object_literal(
    schema: &IrModule,
    tree: &MessageTree,
    locale: &IrModule,
    options: &TypeScriptOptions,
    depth: usize,
    output: &mut String,
) {
    let indent = "  ".repeat(depth);
    let child_indent = "  ".repeat(depth + 1);
    output.push_str("{\n");
    for entry in &tree.messages {
        if let Some(implementation) = message_implementation(locale, &entry.signature.name) {
            output.push_str(&format!(
                "{child_indent}{}: {},\n",
                property_key(&entry.property),
                group_property_value(schema, &entry.signature, implementation, options)
            ));
        }
    }
    for (name, child) in &tree.children {
        output.push_str(&format!("{child_indent}{}: ", property_key(name)));
        emit_object_literal(schema, child, locale, options, depth + 1, output);
        output.push_str(",\n");
    }
    output.push_str(&indent);
    output.push('}');
}

fn group_property_value(
    schema: &IrModule,
    signature: &IrMessage,
    implementation: &IrMessage,
    options: &TypeScriptOptions,
) -> String {
    let Some(body) = &implementation.body else {
        return "\"\"".to_owned();
    };
    if signature.parameters.is_empty() && is_static_text(body) {
        text_expression(body, options)
    } else {
        format!(
            "({}) => {}",
            signature_params(signature),
            message_body(schema, signature, implementation, options)
        )
    }
}

fn message_body(
    schema: &IrModule,
    signature: &IrMessage,
    implementation: &IrMessage,
    options: &TypeScriptOptions,
) -> String {
    let context = signature
        .parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    let default_formatters = parameter_formatters(schema, signature);
    implementation
        .body
        .as_ref()
        .map(|body| text_expression_with_context(body, &context, &default_formatters, options))
        .unwrap_or_else(|| "\"\"".to_owned())
}

fn parameter_formatters(
    schema: &IrModule,
    signature: &IrMessage,
) -> BTreeMap<String, Vec<IrFormatter>> {
    signature
        .parameters
        .iter()
        .filter_map(|parameter| {
            default_type_formatters(schema, &parameter.ty)
                .map(|formatters| (parameter.name.clone(), formatters))
        })
        .collect()
}

fn schema_uses_auto_formatters(schema: &IrModule) -> bool {
    schema.messages.iter().any(|message| {
        message
            .parameters
            .iter()
            .any(|parameter| default_type_formatters(schema, &parameter.ty).is_some())
    })
}

fn default_type_formatters(schema: &IrModule, ty: &str) -> Option<Vec<IrFormatter>> {
    if let Some(alias) = schema.type_aliases.iter().find(|alias| alias.name == ty) {
        if !alias.formatters.is_empty() {
            return Some(alias.formatters.clone());
        }
        return default_type_formatters(schema, &alias.target);
    }

    let kind = match TypeKind::from_name(ty)? {
        TypeKind::Number | TypeKind::Decimal => FormatterKind::Number,
        TypeKind::Date => FormatterKind::Date,
        TypeKind::String | TypeKind::Boolean => return None,
    };
    Some(vec![IrFormatter {
        kind,
        arguments: Vec::<IrFormatterArgument>::new(),
    }])
}

fn signature_params(signature: &IrMessage) -> String {
    signature
        .parameters
        .iter()
        .map(|parameter| format!("{}: {}", parameter.name, ts_type(&parameter.ty)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn message_implementation<'a>(module: &'a IrModule, name: &str) -> Option<&'a IrMessage> {
    module.messages.iter().find(|message| message.name == name)
}

fn function_parameters(function: &IrFunction) -> Vec<String> {
    function
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            parameter
                .name
                .as_deref()
                .map(safe_identifier)
                .unwrap_or_else(|| format!("p{index}"))
        })
        .collect()
}

fn dispatch_expression(
    function: &IrFunction,
    branches: &[IrFunctionBranch],
    depth: usize,
    options: &TypeScriptOptions,
) -> String {
    let parameter_index = dispatch_parameter_indices(function)
        .get(depth)
        .copied()
        .unwrap_or(depth);
    let parameter = function_parameters(function)
        .get(parameter_index)
        .cloned()
        .unwrap_or_else(|| "undefined".to_owned());
    let selector = function
        .parameters
        .get(parameter_index)
        .filter(|parameter| parameter.ty == "Plural")
        .map(|_| format!("{}({parameter})", options.plural_function))
        .unwrap_or_else(|| format!("String({parameter})"));
    let items = branches
        .iter()
        .map(|branch| {
            let value = match &branch.value {
                IrFunctionBranchValue::Text(text) => text_expression(text, options),
                IrFunctionBranchValue::Dispatch(branches) => {
                    dispatch_expression(function, branches, depth + 1, options)
                }
            };
            format!("{}: {value}", property_key(&branch.key))
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("selectBranch({selector}, {{ {items} }})")
}

fn dispatch_parameter_indices(function: &IrFunction) -> Vec<usize> {
    function
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| (parameter.ty != "String").then_some(index))
        .collect()
}
