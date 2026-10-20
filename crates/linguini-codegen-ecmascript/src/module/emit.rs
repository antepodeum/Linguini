use std::collections::{BTreeMap, BTreeSet};

use linguini_codegen::parameter_formatters;
use linguini_ir::{IrMessage, IrModule};

use crate::ecmascript::{EcmaImport, EcmaImportBindings, EcmaNamedImport};
use crate::plural::generate_plural_function_with_visibility;

use super::expr::{
    form_object, function_dispatch_body, text_expression, text_expression_with_context,
};
use super::formatter_codegen::formatter_data_declaration;
use super::formatters::{formatter_requirements, module_uses_inline_functions};
use super::names::{
    emit_docs, emit_docs_with_tags, escape_string, form_binding_name, function_name, property_key,
    safe_identifier, ts_type, JsDocTag,
};
use super::signature::MessageCallSignature;
use super::tree::{nested_message_tree, MessageTree};
use super::type_model::{render_jsdoc_type, TypeModel};
use super::EcmaOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExports {
    pub top_level: Vec<String>,
    pub groups: Vec<String>,
}

pub fn module_imports(
    schema: &IrModule,
    locale: &IrModule,
    options: &EcmaOptions,
    shared_import_path: &str,
) -> Vec<EcmaImport> {
    let mut imports = Vec::new();
    let type_names = schema_type_names(schema);
    if !type_names.is_empty() {
        imports.push(EcmaImport {
            specifier: shared_import_path.to_owned(),
            bindings: EcmaImportBindings::TypeNamed(
                type_names
                    .into_iter()
                    .map(|name| EcmaNamedImport::new(&name, &name))
                    .collect(),
            ),
        });
    }

    let uses_forms = !locale.forms().is_empty();
    let uses_dispatch = !locale.functions().is_empty() || module_uses_inline_functions(locale);
    let uses_named_message_args = schema.messages().iter().any(|signature| {
        !signature.parameters.is_empty()
            && message_implementation(locale, &signature.name).is_some()
    });
    let mut shared_helpers = Vec::new();
    if uses_named_message_args {
        shared_helpers.push("normalizeMessageArgs");
    }
    if !shared_helpers.is_empty() {
        imports.push(EcmaImport::named(
            shared_import_path,
            shared_helpers
                .into_iter()
                .map(|name| EcmaNamedImport::new(name, name))
                .collect(),
        ));
    }
    if (uses_forms || uses_dispatch) && options.plural_rules.is_none() {
        if let Some(path) = &options.plural_import {
            imports.push(EcmaImport::named(
                path,
                vec![EcmaNamedImport::new(
                    &options.plural_function,
                    &options.plural_function,
                )],
            ));
        }
    }
    imports
}

#[cfg(test)]
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

pub(super) fn emit_schema_jsdoc_types(
    schema: &IrModule,
    shared_import_path: &str,

    output: &mut String,
) {
    let type_names = schema_type_names(schema);
    if !type_names.is_empty() {
        output.push_str(&crate::templates::render(
            "module/import_typedefs",
            minijinja::context! { names => type_names, module => shared_import_path },
        ));
        output.push_str("\n\n");
    }
}

pub fn schema_type_names(schema: &IrModule) -> Vec<String> {
    schema
        .enums()
        .iter()
        .map(|item| safe_identifier(&item.name))
        .chain(
            schema
                .type_aliases()
                .iter()
                .map(|item| safe_identifier(&item.name)),
        )
        .collect()
}

pub fn schema_type_aliases(schema: &IrModule) -> Vec<(String, String)> {
    let names = schema
        .enums()
        .iter()
        .map(|item| item.name.as_str())
        .chain(schema.type_aliases().iter().map(|item| item.name.as_str()))
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

pub fn emit_plural_helpers(options: &EcmaOptions, output: &mut String) {
    if let Some(rules) = &options.plural_rules {
        let source =
            generate_plural_function_with_visibility(&options.plural_function, rules, false);
        output.push_str(source.trim_end());
        output.push_str("\n\n");
    }
}

pub fn emit_formatter_data(
    schema: &IrModule,
    locale: &IrModule,
    options: &EcmaOptions,
    output: &mut String,
) {
    let requirements = formatter_requirements(schema, locale);
    if requirements.any() {
        output.push_str(&formatter_data_declaration(
            &options.locale,
            &linguini_codegen::formatter_requests(schema, locale),
            false,
        ));
    }
}

pub(super) fn emit_enum(item: &linguini_ir::IrEnum, output: &mut String) {
    emit_docs(&item.docs, "", output);
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

pub(super) fn emit_locale_enum_jsdoc_types(
    schema: &IrModule,
    locale: &IrModule,
    output: &mut String,
) {
    for item in locale.enums() {
        let supplied_by_schema = schema.enums().iter().any(|schema| schema.name == item.name)
            || schema
                .type_aliases()
                .iter()
                .any(|schema| schema.name == item.name);
        if supplied_by_schema {
            continue;
        }
        emit_docs(&item.docs, "", output);
        let variants = item
            .variants
            .iter()
            .map(|variant| format!("\"{}\"", escape_string(variant)))
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!(
            "/** @typedef {{{variants}}} {} */\n",
            safe_identifier(&item.name)
        ));
    }
}

pub(super) fn emit_type_alias(item: &linguini_ir::IrTypeAlias, output: &mut String) {
    emit_docs(&item.docs, "", output);
    output.push_str(&format!(
        "export type {} = {};\n\n",
        safe_identifier(&item.name),
        ts_type(&item.target)
    ));
}

pub(super) fn emit_forms(
    module: &IrModule,
    options: &EcmaOptions,
    exported: bool,
    output: &mut String,
) {
    for form in module.forms() {
        emit_docs(&form.docs, "", output);
        let variants = form.variants.iter().map(|variant| minijinja::context! {
            key => property_key(&variant.name), value => form_object(&variant.entries, options),
        }).collect::<Vec<_>>();
        output.push_str(&crate::templates::render(
            "module/form",
            minijinja::context! { exported, name => form_binding_name(&form.name), variants },
        ));
    }
}

pub(super) fn emit_variables(
    module: &IrModule,
    options: &EcmaOptions,
    exported: bool,
    output: &mut String,
) {
    for variable in module.variables() {
        emit_docs(&variable.docs, "", output);
        output.push_str(&crate::templates::render("module/constant", minijinja::context! {
            exported, name => safe_identifier(&variable.name), value => text_expression(&variable.value, options),
        }));
    }
}

pub(super) fn emit_local_functions(
    module: &IrModule,
    options: &EcmaOptions,
    exported: bool,
    output: &mut String,
) {
    for function in module.functions() {
        let parameter_names = super::names::function_parameter_names(
            &function.parameters,
            &function.branches,
            &options.plural_function,
        );
        {
            let tags = parameter_names
                .iter()
                .zip(&function.parameters)
                .map(|(name, parameter)| JsDocTag::Param {
                    name: name.clone(),
                    ty: if parameter.ty == "Plural" {
                        "number | bigint | string".to_owned()
                    } else {
                        render_jsdoc_type(&TypeModel::from_source_name(&parameter.ty))
                    },
                })
                .chain(std::iter::once(JsDocTag::Returns {
                    ty: "string".to_owned(),
                }))
                .collect::<Vec<_>>();
            emit_docs_with_tags(&function.docs, &tags, "", output);
        }
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
        let body = function_dispatch_body(
            &function.parameters,
            &function.branches,
            &context,
            &BTreeMap::new(),
            options,
        );
        output.push_str(&crate::templates::render("local_function", minijinja::context! {
            docs => "", name => safe_identifier(&function.name), parameters => parameter_names, body, exported,
        }));
    }
}

pub(super) fn emit_messages(
    schema: &IrModule,
    locale: &IrModule,
    options: &EcmaOptions,
    output: &mut String,
) -> ModuleExports {
    let nested = nested_message_tree(schema);
    let mut exports = ModuleExports {
        top_level: Vec::new(),
        groups: Vec::new(),
    };

    for signature in schema.messages() {
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
    options: &EcmaOptions,
    output: &mut String,
) -> bool {
    let Some(implementation) = message_implementation(locale, &signature.name) else {
        return false;
    };
    let call_signature = MessageCallSignature::from_message(signature);
    let body = message_body(schema, signature, implementation, options);
    let name = function_name(&signature.name);
    if !call_signature.is_parameterized() {
        emit_docs(&signature.docs, "", output);
        output.push_str(&crate::templates::render(
            "module/constant",
            minijinja::context! { exported => true, name, value => body },
        ));
    } else {
        {
            output.push_str(&call_signature.javascript_implementation_docs(&signature.docs));
        }
        output.push_str(&crate::templates::render(
            "module/message",
            minijinja::context! {
                name, parameters => call_signature.implementation_rest_params(),
                normalization => call_signature.normalized_bindings_statement(), body,
            },
        ));
    }
    true
}

fn emit_message_object(
    schema: &IrModule,
    name: &str,
    tree: &MessageTree,
    locale: &IrModule,
    options: &EcmaOptions,
    output: &mut String,
) {
    emit_docs(&tree.docs, "", output);
    let value = object_literal(schema, tree, locale, options, 0);
    output.push_str(&crate::templates::render(
        "module/constant",
        minijinja::context! {
            exported => true, name => safe_identifier(name), value,
        },
    ));
}

fn object_literal(
    schema: &IrModule,
    tree: &MessageTree,
    locale: &IrModule,
    options: &EcmaOptions,
    depth: usize,
) -> String {
    let indent = "  ".repeat(depth);
    let child_indent = "  ".repeat(depth + 1);
    let mut fields = Vec::new();
    for entry in &tree.messages {
        if let Some(implementation) = message_implementation(locale, &entry.signature.name) {
            let mut docs = String::new();
            emit_docs(&entry.signature.docs, &child_indent, &mut docs);
            fields.push(minijinja::context! { docs, key => property_key(&entry.property), value => group_property_value(schema, &entry.signature, implementation, options) });
        }
    }
    for (name, child) in &tree.children {
        let mut docs = String::new();
        emit_docs(&child.docs, &child_indent, &mut docs);
        fields.push(minijinja::context! { docs, key => property_key(name), value => object_literal(schema, child, locale, options, depth + 1) });
    }
    crate::templates::render(
        "module/object",
        minijinja::context! { indent, child_indent, fields },
    )
    .trim_end()
    .to_owned()
}

fn group_property_value(
    schema: &IrModule,
    signature: &IrMessage,
    implementation: &IrMessage,
    options: &EcmaOptions,
) -> String {
    let call_signature = MessageCallSignature::from_message(signature);
    if !call_signature.is_parameterized() {
        message_body(schema, signature, implementation, options)
    } else {
        crate::templates::render(
            "module/message_property",
            minijinja::context! {
                callable_type => call_signature.javascript_callable_type(),
                parameters => call_signature.implementation_rest_params(),
                normalization => call_signature.normalized_bindings_statement(),
                body => message_body(schema, signature, implementation, options),
            },
        )
        .trim_end()
        .to_owned()
    }
}

pub(crate) fn message_body(
    schema: &IrModule,
    signature: &IrMessage,
    implementation: &IrMessage,
    options: &EcmaOptions,
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

fn message_implementation<'a>(module: &'a IrModule, name: &str) -> Option<&'a IrMessage> {
    module
        .messages()
        .iter()
        .find(|message| message.name == name)
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
