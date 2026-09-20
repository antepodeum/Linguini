use linguini_ir::{IrMessage, IrModule};

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaReExport,
    EcmaScriptTarget, EcmaStatement,
};

use super::emit::{schema_type_aliases, schema_type_names};
use super::names::{emit_docs, function_name, property_key, safe_file_stem, safe_identifier};
use super::signature::MessageCallSignature;
use super::tree::{nested_message_tree, MessageTree};

pub fn generate_locale_declaration_with_namespaces(
    schema: &IrModule,
    locale: &str,
    namespaces: &[String],
) -> String {
    let mut imports = Vec::new();
    let mut output = String::new();
    for namespace in namespaces {
        let file_stem = safe_file_stem(namespace);
        let identifier = safe_identifier(namespace);
        imports.push(EcmaImport::named(
            format!("./{locale}/{file_stem}"),
            vec![EcmaNamedImport::new(&identifier, &identifier)],
        ));
    }
    if let Some(type_import) = type_import(schema, "../shared") {
        imports.push(type_import);
    }
    emit_type_alias_declarations(schema, &mut output);
    for namespace in namespaces {
        let identifier = safe_identifier(namespace);
        output.push_str(&format!(
            "export declare const {identifier}: typeof {identifier};\n\n"
        ));
    }
    let exports = emit_message_declarations(schema, &mut output);
    emit_default_declaration_with_namespaces(&exports, namespaces, &mut output);
    render_declaration_module(
        format!("locales/{locale}.d.ts"),
        imports,
        type_re_exports(schema, "../shared"),
        output,
    )
}

pub fn generate_locale_declaration_with_shared_import(
    schema: &IrModule,
    output_path: &str,
    shared_import_path: &str,
    namespace_alias: Option<&str>,
) -> String {
    let imports = type_import(schema, shared_import_path)
        .into_iter()
        .collect::<Vec<_>>();
    let mut output = String::new();
    emit_type_alias_declarations(schema, &mut output);
    let exports = emit_message_declarations(schema, &mut output);
    emit_default_declaration(&exports, &mut output);
    if let Some(namespace_alias) = namespace_alias {
        let identifier = safe_identifier(namespace_alias);
        if !exports.iter().any(|export| export == &identifier) {
            output.push_str(&format!(
                "\nexport declare const {identifier}: typeof lgl;\n"
            ));
        }
    }
    render_declaration_module(
        output_path.to_owned(),
        imports,
        type_re_exports(schema, shared_import_path),
        output,
    )
}

fn type_import(schema: &IrModule, shared_import_path: &str) -> Option<EcmaImport> {
    let type_names = schema_type_names(schema);
    (!type_names.is_empty()).then(|| EcmaImport {
        specifier: shared_import_path.to_owned(),
        bindings: EcmaImportBindings::TypeNamed(
            type_names
                .into_iter()
                .map(|name| EcmaNamedImport::new(&name, &name))
                .collect(),
        ),
    })
}

fn type_re_exports(schema: &IrModule, shared_import_path: &str) -> Vec<EcmaReExport> {
    let type_names = schema_type_names(schema);
    (!type_names.is_empty())
        .then(|| {
            EcmaReExport::type_named(
                shared_import_path,
                type_names
                    .into_iter()
                    .map(|name| EcmaNamedImport::new(&name, &name))
                    .collect(),
            )
        })
        .into_iter()
        .collect()
}

fn emit_type_alias_declarations(schema: &IrModule, output: &mut String) {
    for (public_name, generated_name) in schema_type_aliases(schema) {
        output.push_str(&format!(
            "export type {public_name} = {generated_name};\n\n"
        ));
    }
}

fn render_declaration_module(
    output_path: String,
    imports: Vec<EcmaImport>,
    re_exports: Vec<EcmaReExport>,
    body: String,
) -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        output_path,
        None,
    ));
    module.extend_imports(imports);
    for item in re_exports {
        module.push_re_export(item);
    }
    module.push_statement(EcmaStatement::type_declaration(body, None));
    module.render_code()
}

fn emit_message_declarations(schema: &IrModule, output: &mut String) -> Vec<String> {
    let nested = nested_message_tree(schema);
    let mut exports = Vec::new();

    for signature in schema.messages() {
        if signature.name.contains('.') {
            continue;
        }
        emit_function_declaration(signature, output);
        exports.push(function_name(&signature.name));
    }

    for (group, messages) in nested.children {
        emit_message_object_declaration(&group, &messages, output);
        exports.push(safe_identifier(&group));
    }

    exports
}

fn emit_message_object_declaration(name: &str, tree: &MessageTree, output: &mut String) {
    emit_docs(&tree.docs, "", output);
    output.push_str(&format!("export declare const {}: ", safe_identifier(name)));
    emit_object_type(tree, 0, output);
    output.push_str(";\n\n");
}

fn emit_object_type(tree: &MessageTree, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    let child_indent = "  ".repeat(depth + 1);
    output.push_str("{\n");
    for entry in &tree.messages {
        let signature = MessageCallSignature::from_message(&entry.signature);
        if !signature.is_parameterized() {
            emit_docs(&entry.signature.docs, &child_indent, output);
        }
        output.push_str(&format!(
            "{child_indent}readonly {}: {};\n",
            property_key(&entry.property),
            signature.callable_type_with_docs(&entry.signature.docs, &child_indent)
        ));
    }
    for (name, child) in &tree.children {
        emit_docs(&child.docs, &child_indent, output);
        output.push_str(&format!("{child_indent}readonly {}: ", property_key(name)));
        emit_object_type(child, depth + 1, output);
        output.push_str(";\n");
    }
    output.push_str(&indent);
    output.push('}');
}

fn emit_function_declaration(signature: &IrMessage, output: &mut String) {
    let name = function_name(&signature.name);
    output.push_str(
        &MessageCallSignature::from_message(signature)
            .overload_declarations(name.as_str(), &signature.docs),
    );
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
    for name in exports {
        output.push_str(&format!("  readonly {name}: typeof {name};\n"));
    }
    for namespace in namespaces {
        let identifier = safe_identifier(namespace);
        output.push_str(&format!("  readonly {identifier}: typeof {identifier};\n"));
    }
    output.push_str("};\n\n");
    output.push_str("export default lgl;\n");
}
