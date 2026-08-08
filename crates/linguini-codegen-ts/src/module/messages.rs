use std::collections::BTreeSet;

use linguini_ir::{IrMessage, IrModule};

use super::names::{emit_docs, property_key, safe_identifier, ts_type};
use super::tree::{nested_message_tree, MessageTree, MessageTreeMessage};

/// Generate the schema-owned namespace type used by the project runtime.
///
/// This module deliberately contains no locale imports. Locale modules are implementation
/// values and may contain a different inferred shape (for example, because a locale source
/// omits an implementation that is supplied by fallback composition), while `LinguiniMessages`
/// describes the complete schema contract exposed to application code.
pub fn generate_messages_module(schema: &IrModule) -> String {
    let mut output = String::new();
    let type_names = message_type_names(schema);
    if !type_names.is_empty() {
        output.push_str(&format!(
            "import type {{ {} }} from \"./shared\";\n\n",
            type_names.join(", ")
        ));
    }

    output.push_str("export type LinguiniMessages = ");
    emit_object_type(&schema_message_tree(schema), 0, &mut output);
    output.push_str(";\n");
    output
}

fn message_type_names(schema: &IrModule) -> Vec<String> {
    let referenced = schema
        .messages
        .iter()
        .flat_map(|message| {
            message
                .parameters
                .iter()
                .map(|parameter| parameter.ty.as_str())
        })
        .collect::<BTreeSet<_>>();
    schema
        .enums
        .iter()
        .map(|item| item.name.as_str())
        .chain(schema.type_aliases.iter().map(|item| item.name.as_str()))
        .filter(|name| referenced.contains(name))
        .map(safe_identifier)
        .collect()
}

fn schema_message_tree(schema: &IrModule) -> MessageTree {
    let mut tree = nested_message_tree(schema);
    for message in &schema.messages {
        if !message.name.contains('.') {
            tree.messages.push(MessageTreeMessage {
                property: message.name.clone(),
                signature: message.clone(),
            });
        }
    }
    tree
}

fn emit_object_type(tree: &MessageTree, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    let child_indent = "  ".repeat(depth + 1);
    output.push_str("{\n");
    for entry in &tree.messages {
        emit_docs(&entry.signature.docs, &child_indent, output);
        output.push_str(&format!(
            "{child_indent}readonly {}: {};\n",
            message_property_key(&entry.property, depth),
            message_type(&entry.signature)
        ));
    }
    for (name, child) in &tree.children {
        emit_docs(&child.docs, &child_indent, output);
        output.push_str(&format!(
            "{child_indent}readonly {}: ",
            message_property_key(name, depth)
        ));
        emit_object_type(child, depth + 1, output);
        output.push_str(";\n");
    }
    output.push_str(&indent);
    output.push('}');
}

fn message_property_key(name: &str, depth: usize) -> String {
    if depth == 0 {
        safe_identifier(name)
    } else {
        property_key(name)
    }
}

fn message_type(signature: &IrMessage) -> String {
    if signature.parameters.is_empty() {
        "string".to_owned()
    } else {
        format!("({}) => string", signature_params(signature))
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
