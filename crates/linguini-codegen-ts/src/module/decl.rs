use linguini_ir::{IrMessage, IrModule};

use super::emit::schema_type_names;
use super::names::{escape_comment, escape_string, function_name, property_key, ts_type};
use super::templates::{render_template, SHARED_DECLARATIONS, SINGLE_INDEX_DECLARATIONS};
use super::tree::{nested_message_tree, MessageTree};
use super::TypeScriptOptions;

pub fn generate_shared_declaration(schema: &IrModule) -> String {
    let mut output = String::new();
    emit_type_declarations(schema, &mut output);
    output.push_str(SHARED_DECLARATIONS);
    output
}

pub fn generate_index_declaration(options: &TypeScriptOptions) -> String {
    let locale = options.locale.replace('-', "_");
    let locale_path = escape_string(&options.locale);
    let locale_literal = format!("\"{}\"", escape_string(&options.locale));

    render_template(
        SINGLE_INDEX_DECLARATIONS,
        &[
            ("LOCALE_IDENTIFIER", locale),
            ("LOCALE_PATH", locale_path),
            ("LOCALE_LITERAL", locale_literal),
        ],
    )
}

pub fn generate_locale_declaration(schema: &IrModule) -> String {
    generate_locale_declaration_with_shared_import(schema, "../shared", None)
}

pub fn generate_locale_declaration_with_namespaces(
    schema: &IrModule,
    locale: &str,
    namespaces: &[String],
) -> String {
    let mut output = String::new();
    for namespace in namespaces {
        output.push_str(&format!(
            "import {{ {} }} from \"./{}/{}\";\n",
            namespace,
            escape_string(locale),
            escape_string(namespace)
        ));
    }
    if !namespaces.is_empty() {
        output.push('\n');
    }
    emit_type_imports(schema, "../shared", &mut output);
    emit_type_reexports(schema, "../shared", &mut output);
    for namespace in namespaces {
        output.push_str(&format!(
            "export declare const {namespace}: typeof {namespace};\n\n"
        ));
    }
    let exports = emit_message_declarations(schema, &mut output);
    emit_default_declaration_with_namespaces(&exports, namespaces, &mut output);
    output
}

pub fn generate_locale_declaration_with_shared_import(
    schema: &IrModule,
    shared_import_path: &str,
    namespace_alias: Option<&str>,
) -> String {
    let mut output = String::new();
    emit_type_imports(schema, shared_import_path, &mut output);
    emit_type_reexports(schema, shared_import_path, &mut output);
    let exports = emit_message_declarations(schema, &mut output);
    emit_default_declaration(&exports, &mut output);
    if let Some(namespace_alias) = namespace_alias {
        if !exports.iter().any(|export| export == namespace_alias) {
            output.push_str(&format!(
                "\nexport declare const {namespace_alias}: typeof lgl;\n"
            ));
        }
    }
    output
}

fn emit_type_imports(schema: &IrModule, shared_import_path: &str, output: &mut String) {
    let type_names = schema_type_names(schema);
    if !type_names.is_empty() {
        output.push_str(&format!(
            "import type {{ {} }} from \"{}\";\n\n",
            type_names.join(", "),
            shared_import_path
        ));
    }
}

fn emit_type_reexports(schema: &IrModule, shared_import_path: &str, output: &mut String) {
    let type_names = schema_type_names(schema);
    if !type_names.is_empty() {
        output.push_str(&format!(
            "export type {{ {} }} from \"{}\";\n\n",
            type_names.join(", "),
            shared_import_path
        ));
    }
}

fn emit_type_declarations(schema: &IrModule, output: &mut String) {
    for item in &schema.enums {
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

    for item in &schema.type_aliases {
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

fn emit_message_declarations(schema: &IrModule, output: &mut String) -> Vec<String> {
    let nested = nested_message_tree(schema);
    let mut exports = Vec::new();

    for signature in &schema.messages {
        if signature.name.contains('.') {
            continue;
        }
        emit_function_declaration(signature, output);
        exports.push(function_name(&signature.name));
    }

    for (group, messages) in nested.children {
        emit_message_object_declaration(&group, &messages, output);
        exports.push(group);
    }

    exports
}

fn emit_message_object_declaration(name: &str, tree: &MessageTree, output: &mut String) {
    output.push_str(&format!("export declare const {name}: "));
    emit_object_type(tree, 0, output);
    output.push_str(";\n\n");
}

fn emit_object_type(tree: &MessageTree, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    let child_indent = "  ".repeat(depth + 1);
    output.push_str("{\n");
    for entry in &tree.messages {
        output.push_str(&format!(
            "{child_indent}readonly {}: {};\n",
            property_key(&entry.property),
            group_property_type(&entry.signature)
        ));
    }
    for (name, child) in &tree.children {
        output.push_str(&format!("{child_indent}readonly {}: ", property_key(name)));
        emit_object_type(child, depth + 1, output);
        output.push_str(";\n");
    }
    output.push_str(&indent);
    output.push('}');
}

fn emit_function_declaration(signature: &IrMessage, output: &mut String) {
    for doc in &signature.docs {
        output.push_str(&format!("/** {} */\n", escape_comment(doc)));
    }
    output.push_str(&format!(
        "export declare function {}({}): string;\n\n",
        function_name(&signature.name),
        signature_params(signature)
    ));
}

fn group_property_type(signature: &IrMessage) -> String {
    if signature.parameters.is_empty() {
        "string".to_owned()
    } else {
        format!("({}) => string", signature_params(signature))
    }
}

fn emit_default_declaration(exports: &[String], output: &mut String) {
    emit_default_declaration_with_namespaces(exports, &[], output);
}

fn emit_default_declaration_with_namespaces(
    exports: &[String],
    namespaces: &[String],
    output: &mut String,
) {
    output.push_str("declare const lgl: {\n");
    for name in exports.iter().chain(namespaces.iter()) {
        output.push_str(&format!("  readonly {name}: typeof {name};\n"));
    }
    output.push_str("};\n\n");
    output.push_str("export default lgl;\n");
}

fn signature_params(signature: &IrMessage) -> String {
    signature
        .parameters
        .iter()
        .map(|parameter| format!("{}: {}", parameter.name, ts_type(&parameter.ty)))
        .collect::<Vec<_>>()
        .join(", ")
}
