use linguini_ir::{IrMessage, IrModule};

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaNamedImport,
    EcmaReExport, EcmaStatement,
};

use super::emit::{schema_type_aliases, schema_type_names};
use super::exports::{default_context, ValueExport};
use super::names::{declaration_import_path, emit_docs, safe_file_stem, safe_identifier};
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
        imports.push(ValueExport::new(namespace).import(format!("./{locale}/{file_stem}.js")));
    }
    if let Some(type_import) = type_import(schema, "../shared") {
        imports.push(type_import);
    }
    emit_type_alias_declarations(schema, &mut output);
    for namespace in namespaces {
        output.push_str(&ValueExport::new(namespace).statement(true));
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
        if !exports.iter().any(|export| export.name == identifier) {
            output.push('\n');
            output.push_str(&ValueExport::alias(namespace_alias, "lgl").statement(false));
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
        specifier: declaration_import_path(shared_import_path),
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
                declaration_import_path(shared_import_path),
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
        output.push_str(&crate::templates::render(
            "declarations/alias",
            minijinja::context! { name => public_name, ty => generated_name },
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
        EcmaModuleKind::Declaration,
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

fn emit_message_declarations(schema: &IrModule, output: &mut String) -> Vec<ValueExport> {
    let nested = nested_message_tree(schema);
    let mut exports = Vec::new();

    for signature in schema.messages() {
        if signature.name.contains('.') {
            continue;
        }
        emit_function_declaration(signature, output);
        exports.push(ValueExport::new(&signature.name));
    }

    for (group, messages) in nested.children {
        emit_message_object_declaration(&group, &messages, output);
        exports.push(ValueExport::new(&group));
    }

    exports
}

fn emit_message_object_declaration(name: &str, tree: &MessageTree, output: &mut String) {
    emit_docs(&tree.docs, "", output);
    let export = ValueExport::new(name);
    let (ty, _) = super::type_tree::MessageTreeType::locale_declaration().render(tree);
    output.push_str(&crate::templates::render(
        "declarations/constant",
        minijinja::context! { name => &export.binding, ty, exported => export.is_direct(), blank_line => export.is_direct() },
    ));
    export.append_alias(output);
}

fn emit_function_declaration(signature: &IrMessage, output: &mut String) {
    let export = ValueExport::new(&signature.name);
    output.push_str(
        &MessageCallSignature::from_message(signature).overload_declarations(
            &export.binding,
            &signature.docs,
            export.is_direct(),
        ),
    );
    export.append_alias(output);
}

fn emit_default_declaration(exports: &[ValueExport], output: &mut String) {
    emit_default_declaration_with_namespaces(exports, &[], output);
}

fn emit_default_declaration_with_namespaces(
    exports: &[ValueExport],
    namespaces: &[String],
    output: &mut String,
) {
    output.push_str(&crate::templates::render(
        "declarations/default",
        default_context(exports, namespaces),
    ));
}
