use std::collections::BTreeSet;

use linguini_ir::IrModule;

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaStatement,
};

use super::names::{emit_docs, property_key, safe_identifier};
use super::signature::MessageCallSignature;
use super::tree::{nested_message_tree, MessageTree, MessageTreeMessage};

/// Generate the schema-owned namespace type used by the project runtime.
///
/// This module deliberately contains no locale imports. Locale modules are implementation
/// values and may contain a different inferred shape (for example, because a locale source
/// omits an implementation that is supplied by fallback composition), while `LinguiniMessages`
/// describes the complete schema contract exposed to application code.
pub fn generate_messages_module(
    schema: &IrModule,
    output: EcmaModuleOutput,
    shared_import_path: &str,
) -> String {
    messages_module(schema, output, shared_import_path).render_code()
}

pub(super) fn messages_module(
    schema: &IrModule,
    output: EcmaModuleOutput,
    shared_import_path: &str,
) -> EcmaModule {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    let type_names = message_type_names(schema);
    if !target.is_typescript() {
        for name in &type_names {
            module.push_statement(EcmaStatement::documentation(
                format!("/** @typedef {{import(\"{shared_import_path}\").{name}}} {name} */"),
                None,
            ));
        }
    }
    if !type_names.is_empty() {
        module.push_import(EcmaImport {
            specifier: shared_import_path.to_owned(),
            bindings: EcmaImportBindings::TypeNamed(
                type_names
                    .into_iter()
                    .map(|name| EcmaNamedImport::new(&name, &name))
                    .collect(),
            ),
        });
    }

    let mut output = String::new();
    output.push_str("export type LinguiniMessages = ");
    emit_object_type(
        &schema_message_tree(schema),
        0,
        crate::ecmascript::EcmaScriptTarget::TypeScript,
        &mut output,
    );
    output.push_str(";\n");
    module.push_statement(EcmaStatement::type_declaration(output, None));
    if !target.is_typescript() {
        let mut ty = String::new();
        emit_object_type(&schema_message_tree(schema), 0, target, &mut ty);
        let mut jsdoc = String::new();
        super::names::emit_docs_with_tags(
            &[],
            &[super::names::JsDocTag::Typedef {
                name: "LinguiniMessages".to_owned(),
                ty: ty.replace("*/", "*\\u002f"),
            }],
            "",
            &mut jsdoc,
        );
        module.push_statement(EcmaStatement::documentation(jsdoc, None));
        module.push_statement(EcmaStatement::generated("export {};"));
    }
    module
}

fn message_type_names(schema: &IrModule) -> Vec<String> {
    let referenced = schema
        .messages()
        .iter()
        .flat_map(|message| {
            message
                .parameters
                .iter()
                .map(|parameter| parameter.ty.as_str())
        })
        .collect::<BTreeSet<_>>();
    schema
        .enums()
        .iter()
        .map(|item| item.name.as_str())
        .chain(schema.type_aliases().iter().map(|item| item.name.as_str()))
        .filter(|name| referenced.contains(name))
        .map(safe_identifier)
        .collect()
}

fn schema_message_tree(schema: &IrModule) -> MessageTree {
    let mut tree = nested_message_tree(schema);
    for message in schema.messages() {
        if !message.name.contains('.') {
            tree.messages.push(MessageTreeMessage {
                property: message.name.clone(),
                signature: message.clone(),
            });
        }
    }
    tree
}

fn emit_object_type(
    tree: &MessageTree,
    depth: usize,
    target: crate::ecmascript::EcmaScriptTarget,
    output: &mut String,
) {
    let indent = "  ".repeat(depth);
    let child_indent = "  ".repeat(depth + 1);
    output.push_str("{\n");
    for entry in &tree.messages {
        let signature = MessageCallSignature::from_message(&entry.signature);
        if target.is_typescript() && !signature.is_parameterized() {
            emit_docs(&entry.signature.docs, &child_indent, output);
        }
        output.push_str(&format!(
            "{child_indent}readonly {}: {};\n",
            message_property_key(&entry.property, depth),
            if target.is_typescript() {
                signature.callable_type_with_docs(&entry.signature.docs, &child_indent)
            } else {
                signature.javascript_callable_type()
            }
        ));
    }
    for (name, child) in &tree.children {
        if target.is_typescript() {
            emit_docs(&child.docs, &child_indent, output);
        }
        output.push_str(&format!(
            "{child_indent}readonly {}: ",
            message_property_key(name, depth)
        ));
        emit_object_type(child, depth + 1, target, output);
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

#[cfg(test)]
mod tests {
    use super::*;
    use linguini_ir::{IrMessage, IrModuleBuilder, IrParameter};
    #[test]
    fn javascript_namespace_types_preserve_hostile_keys_without_declaration_files() {
        let schema = IrModuleBuilder::new()
            .push_message(IrMessage {
                name: "top.bad*/key.leaf".to_owned(),
                docs: vec!["Unsafe */ prose".to_owned()],
                parameters: vec![IrParameter {
                    name: "name*/key".to_owned(),
                    ty: "String".to_owned(),
                }],
                body: None,
            })
            .build()
            .unwrap();
        let code = messages_module(
            &schema,
            EcmaModuleOutput::new(
                crate::ecmascript::EcmaScriptTarget::JavaScript,
                "unsafe/messages.js",
                None,
            ),
            "./shared.js",
        )
        .render_code();
        assert!(!code.contains("bad*/key"));
        assert!(!code.contains("name*/key"));
        assert!(code.contains("bad*\\u002fkey"));
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-complete-project/unsafe/messages.js");
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &code).unwrap();
        }
        assert_eq!(code, std::fs::read_to_string(path).unwrap());
    }
}
