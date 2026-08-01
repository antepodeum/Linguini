use std::collections::{BTreeMap, BTreeSet};

use linguini_core::TypeKind;
use linguini_ir::{IrFormatter, IrFormatterArgument, IrFunction, IrMessage, IrModule};

use super::expr::{
    form_object, formatter_data_declaration, function_dispatch_expression, text_expression,
    text_expression_with_context,
};
use super::formatters::{formatter_requirements, module_uses_inline_functions};
use super::names::{
    escape_comment, escape_string, form_binding_name, function_name, property_key, safe_identifier,
    ts_type,
};
use super::tree::{nested_message_tree, MessageTree};
use super::TypeScriptOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExports {
    pub top_level: Vec<String>,
    pub groups: Vec<String>,
}

pub fn emit_imports(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    shared_import_path: &str,
    output: &mut String,
) {
    let type_names = schema_type_names(schema);
    if !type_names.is_empty() {
        output.push_str(&format!(
            "import type {{ {} }} from \"{}\";\n",
            type_names.join(", "),
            shared_import_path
        ));
    }

    let uses_forms = !locale.forms.is_empty();
    let uses_dispatch = !locale.functions.is_empty() || module_uses_inline_functions(locale);
    if uses_forms || uses_dispatch {
        output.push_str(&format!(
            "import {{ selectBranch }} from \"{shared_import_path}\";\n"
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
    if !type_names.is_empty() && !uses_forms && !uses_dispatch {
        output.push('\n');
    }
}

pub fn emit_schema_type_reexports(
    schema: &IrModule,
    shared_import_path: &str,
    output: &mut String,
) {
    let type_names = schema_type_names(schema);
    if !type_names.is_empty() {
        output.push_str(&format!(
            "export type {{ {} }} from \"{}\";\n\n",
            type_names.join(", "),
            shared_import_path
        ));
    }
    for (public_name, generated_name) in schema_type_aliases(schema) {
        output.push_str(&format!(
            "export type {public_name} = {generated_name};\n\n"
        ));
    }
}

pub fn schema_type_names(schema: &IrModule) -> Vec<String> {
    schema
        .enums
        .iter()
        .map(|item| safe_identifier(&item.name))
        .chain(
            schema
                .type_aliases
                .iter()
                .map(|item| safe_identifier(&item.name)),
        )
        .collect()
}

pub fn schema_type_aliases(schema: &IrModule) -> Vec<(String, String)> {
    let names = schema
        .enums
        .iter()
        .map(|item| item.name.as_str())
        .chain(schema.type_aliases.iter().map(|item| item.name.as_str()))
        .collect::<Vec<_>>();
    let mut public_name_counts = BTreeMap::new();
    for name in &names {
        *public_name_counts
            .entry(name.rsplit('.').next().unwrap_or(name))
            .or_insert(0_usize) += 1;
    }
    let generated_names = names
        .iter()
        .map(|name| safe_identifier(name))
        .collect::<BTreeSet<_>>();

    names
        .into_iter()
        .filter_map(|name| {
            let public_name = safe_identifier(name.rsplit('.').next().unwrap_or(name));
            let generated_name = safe_identifier(name);
            (public_name != generated_name
                && public_name_counts.get(name.rsplit('.').next().unwrap_or(name)) == Some(&1)
                && !generated_names.contains(&public_name))
            .then_some((public_name, generated_name))
        })
        .collect()
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
    let requirements = formatter_requirements(schema, locale);
    if requirements.any() {
        output.push_str(&formatter_data_declaration(&options.locale, requirements));
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
        output.push_str(&format!(
            "export type {} = {variants};\n\n",
            safe_identifier(&item.name)
        ));
    }
}

pub fn emit_locale_enum_types(schema: &IrModule, locale: &IrModule, output: &mut String) {
    for item in &locale.enums {
        let supplied_by_schema = schema.enums.iter().any(|schema| schema.name == item.name)
            || schema
                .type_aliases
                .iter()
                .any(|schema| schema.name == item.name);
        if supplied_by_schema {
            continue;
        }
        let variants = item
            .variants
            .iter()
            .map(|variant| format!("\"{}\"", escape_string(variant)))
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!(
            "type {} = {variants};\n\n",
            safe_identifier(&item.name)
        ));
    }
}

pub fn emit_type_aliases(module: &IrModule, output: &mut String) {
    for item in &module.type_aliases {
        for doc in &item.docs {
            output.push_str(&format!("/** {} */\n", escape_comment(doc)));
        }
        output.push_str(&format!(
            "export type {} = {};\n\n",
            safe_identifier(&item.name),
            ts_type(&item.target)
        ));
    }
}

pub fn emit_forms(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    for form in &module.forms {
        output.push_str(&format!("const {} = {{\n", form_binding_name(&form.name)));
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

pub fn emit_variables(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    for variable in &module.variables {
        for doc in &variable.docs {
            output.push_str(&format!("/** {} */\n", escape_comment(doc)));
        }
        output.push_str(&format!(
            "const {} = {};\n\n",
            safe_identifier(&variable.name),
            text_expression(&variable.value, options)
        ));
    }
}

pub fn emit_local_functions(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    for function in &module.functions {
        let parameter_names = function_parameters(function);
        let params = parameter_names
            .iter()
            .zip(&function.parameters)
            .map(|(name, parameter)| {
                let ty = if parameter.ty == "Plural" {
                    "number | bigint | string".to_owned()
                } else {
                    ts_type(&parameter.ty)
                };
                format!("{name}: {ty}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "function {}({params}): string {{\n",
            safe_identifier(&function.name)
        ));
        let context = function
            .parameters
            .iter()
            .filter_map(|parameter| {
                parameter
                    .name
                    .as_ref()
                    .map(|name| (name.clone(), parameter.ty.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        output.push_str(&format!(
            "  return {};\n",
            function_dispatch_expression(
                &function.parameters,
                &function.branches,
                &context,
                &BTreeMap::new(),
                options,
            )
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
        exports.groups.push(safe_identifier(&group));
    }

    exports
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
    let name = function_name(&signature.name);
    if signature.parameters.is_empty() {
        output.push_str(&format!("export const {name} = {body};\n\n"));
    } else {
        output.push_str(&format!(
            "export function {name}({params}): string {{\n  return {body};\n}}\n\n"
        ));
    }
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
    output.push_str(&format!("export const {} = ", safe_identifier(name)));
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
    if signature.parameters.is_empty() {
        message_body(schema, signature, implementation, options)
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

fn default_type_formatters(schema: &IrModule, ty: &str) -> Option<Vec<IrFormatter>> {
    let mut current = ty;
    let mut visited = BTreeSet::new();

    loop {
        if !visited.insert(current) {
            return None;
        }

        if let Some(alias) = schema
            .type_aliases
            .iter()
            .find(|alias| alias.name == current)
        {
            if !alias.formatters.is_empty() {
                return Some(alias.formatters.clone());
            }
            current = &alias.target;
            continue;
        }

        let kind = TypeKind::from_name(current)?.default_formatter()?;
        return Some(vec![IrFormatter {
            kind,
            arguments: Vec::<IrFormatterArgument>::new(),
        }]);
    }
}

fn signature_params(signature: &IrMessage) -> String {
    signature
        .parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}: {}",
                safe_identifier(&parameter.name),
                ts_type(&parameter.ty)
            )
        })
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
                .unwrap_or_else(|| format!("__lgl_p{index}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::emit_schema_type_reexports;
    use linguini_ir::{lower_schema, qualify_module};
    use linguini_syntax::parse_schema;

    #[test]
    fn namespaced_types_keep_unique_public_aliases() {
        let mut schema = lower_schema(
            &parse_schema("enum Fruit { apple }\ntype Money = Decimal\n").expect("schema"),
        );
        qualify_module(&mut schema, "shop");
        let mut output = String::new();

        emit_schema_type_reexports(&schema, "./shared", &mut output);

        assert!(output.contains("export type Fruit = __lgl_name_73686F702E4672756974;"));
        assert!(output.contains("export type Money = __lgl_name_73686F702E4D6F6E6579;"));
    }
}
