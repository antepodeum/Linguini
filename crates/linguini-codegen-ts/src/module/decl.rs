use std::collections::BTreeMap;

use linguini_ir::{IrMessage, IrModule};

use super::names::{escape_comment, escape_string, function_name, property_key, ts_type};
use super::TypeScriptOptions;

pub fn generate_shared_declaration() -> String {
    "export declare function selectBranch(\n  key: string,\n  branches: Record<string, string>,\n): string;\n"
        .to_owned()
}

pub fn generate_index_declaration(options: &TypeScriptOptions) -> String {
    let locale = options.locale.replace('-', "_");
    let locale_path = escape_string(&options.locale);
    let locale_literal = format!("\"{}\"", escape_string(&options.locale));
    let mut output = String::new();
    output.push_str(&format!(
        "import {locale} from \"./locales/{locale_path}\";\n\n"
    ));
    output.push_str(&format!(
        "declare const localeModules: {{ readonly {locale}: typeof {locale} }};\n\n"
    ));
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str(&format!(
        "type LinguiniLanguageInput = LinguiniLanguage | {locale_literal};\n\n"
    ));
    output.push_str(
        "export declare function createLinguini(language: LinguiniLanguageInput): Linguini;\n\n",
    );
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): { readonly lgl: Linguini };\n");
    output
}

pub fn generate_locale_declaration(schema: &IrModule) -> String {
    let mut output = String::new();
    emit_type_declarations(schema, &mut output);
    let exports = emit_message_declarations(schema, &mut output);
    emit_default_declaration(&exports, &mut output);
    output
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
    let mut exports = Vec::new();

    for signature in &schema.messages {
        if signature.name.contains('.') {
            continue;
        }
        emit_function_declaration(signature, output);
        exports.push(function_name(&signature.name));
    }

    for (group, messages) in group_messages(schema) {
        output.push_str(&format!("export declare const {group}: {{\n"));
        for signature in messages {
            let property = signature.name.split('.').nth(1).unwrap_or(&signature.name);
            output.push_str(&format!(
                "  readonly {}: {};\n",
                property_key(property),
                group_property_type(&signature)
            ));
        }
        output.push_str("};\n\n");
        exports.push(group);
    }

    exports
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
    output.push_str("declare const lgl: {\n");
    for name in exports {
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
