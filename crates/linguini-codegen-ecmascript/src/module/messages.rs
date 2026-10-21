use std::collections::BTreeSet;

use linguini_ir::IrModule;

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaStatement,
};

use super::names::safe_identifier;
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
    if !target.is_declaration() {
        for name in &type_names {
            module.push_statement(EcmaStatement::documentation(
                crate::templates::render(
                    "module/import_typedefs",
                    minijinja::context! { names => [name], module => shared_import_path },
                ),
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

    let tree = schema_message_tree(schema);
    let (output, mappings) =
        super::type_tree::MessageTreeType::schema(schema, target).render(&tree);
    if target.is_declaration() {
        let output = crate::templates::render(
            "declarations/alias",
            minijinja::context! { name => "LinguiniMessages", ty => output, blank_line => false },
        );
        module
            .push_statement_with_mappings(EcmaStatement::type_declaration(output, None), mappings);
    } else {
        let mut jsdoc = String::new();
        super::names::emit_docs_with_tags(
            &[],
            &[super::names::JsDocTag::Typedef {
                name: "LinguiniMessages".to_owned(),
                ty: output.replace("*/", "*\\u002f"),
            }],
            "",
            &mut jsdoc,
        );
        module.push_statement_with_mappings(EcmaStatement::documentation(jsdoc, None), mappings);
        module.push_statement(EcmaStatement::generated(crate::templates::render(
            "module/exports",
            minijinja::context! { names => Vec::<String>::new(), blank_line => false },
        )));
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
                crate::ecmascript::EcmaModuleKind::JavaScript,
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
