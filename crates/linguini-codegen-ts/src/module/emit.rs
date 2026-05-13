use std::collections::BTreeMap;

use linguini_ir::{IrBranchPattern, IrMessage, IrModule};

use super::expr::{
    branch_switch, form_object, is_static_text, text_expression, text_expression_with_context,
};
use super::formatters::module_uses_formatters;
use super::names::{
    escape_comment, escape_string, function_name, property_key, safe_identifier, string_literal,
    ts_type,
};
use super::TypeScriptOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExports {
    pub top_level: Vec<String>,
    pub groups: Vec<String>,
}

pub fn emit_imports(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    let uses_forms = !module.forms.is_empty();
    let uses_formatters = module_uses_formatters(module);
    if uses_forms || uses_formatters {
        let mut imports = Vec::new();
        if uses_formatters {
            imports.push("formatCurrency");
            imports.push("formatDate");
        }
        if uses_forms {
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
            if let Some(selector) = &variant.selector {
                output.push_str(&format!(
                    "  {}: ({}: string) => {},\n",
                    property_key(&variant.name),
                    safe_identifier(selector),
                    branch_switch(&safe_identifier(selector), &variant.entries, options)
                ));
            } else {
                output.push_str(&format!(
                    "  {}: {},\n",
                    property_key(&variant.name),
                    form_object(&variant.entries, options)
                ));
            }
        }
        output.push_str("} as const;\n\n");
    }
}

pub fn emit_local_functions(module: &IrModule, options: &TypeScriptOptions, output: &mut String) {
    for function in &module.functions {
        let params = function
            .parameters
            .iter()
            .map(|name| format!("{}: string", safe_identifier(name)))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "function {}({params}): string {{\n",
            function.name
        ));
        for branch in &function.branches {
            match &branch.pattern {
                IrBranchPattern::Names(names) => output.push_str(&format!(
                    "  if ({}) return {};\n",
                    tuple_condition(&function.parameters, names),
                    text_expression(&branch.value, options)
                )),
                IrBranchPattern::Else => {
                    output.push_str(&format!(
                        "  return {};\n",
                        text_expression(&branch.value, options)
                    ));
                }
            }
        }
        if !function
            .branches
            .iter()
            .any(|branch| branch.pattern == IrBranchPattern::Else)
        {
            output.push_str("  return \"\";\n");
        }
        output.push_str("}\n\n");
    }
}

pub fn emit_messages(
    schema: &IrModule,
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: &mut String,
) -> ModuleExports {
    let grouped = group_messages(schema);
    let mut exports = ModuleExports {
        top_level: Vec::new(),
        groups: Vec::new(),
    };

    for signature in &schema.messages {
        if signature.name.contains('.') {
            continue;
        }
        if emit_message_function(signature, locale, options, output) {
            exports.top_level.push(function_name(&signature.name));
        }
    }

    for (group, messages) in grouped {
        emit_group_object(&group, &messages, locale, options, output);
        exports.groups.push(group);
    }

    exports
}

pub fn emit_shared(output: &mut String) {
    output.push_str("export type FormatterOptions = Record<string, string>;\n\n");
    output.push_str("export function formatCurrency(\n");
    output.push_str("  value: number | string,\n");
    output.push_str("  locale: string,\n");
    output.push_str("  options: FormatterOptions = {},\n");
    output.push_str("): string {\n");
    output.push_str("  const currency = options.code ?? \"USD\";\n");
    output.push_str("  return new Intl.NumberFormat(locale, {\n");
    output.push_str("    style: \"currency\",\n");
    output.push_str("    currency,\n");
    output.push_str("  }).format(Number(value));\n");
    output.push_str("}\n\n");
    output.push_str("export function formatDate(\n");
    output.push_str("  value: Date | number | string,\n");
    output.push_str("  locale: string,\n");
    output.push_str("  options: FormatterOptions = {},\n");
    output.push_str("): string {\n");
    output.push_str("  if (typeof value === \"string\") return value;\n");
    output.push_str("  const intlOptions: Record<string, string> = {};\n");
    output.push_str("  if (options.style) intlOptions.dateStyle = options.style;\n");
    output.push_str(
        "  return new Intl.DateTimeFormat(locale, intlOptions as Intl.DateTimeFormatOptions).format(value);\n",
    );
    output.push_str("}\n\n");
    output.push_str("export function selectBranch(\n");
    output.push_str("  key: string,\n");
    output.push_str("  branches: Record<string, string>,\n");
    output.push_str("): string {\n");
    output.push_str("  return branches[key] ?? branches.other ?? \"\";\n");
    output.push_str("}\n");
}

pub fn emit_index(options: &TypeScriptOptions, output: &mut String) {
    let locale = safe_identifier(&options.locale);
    let locale_path = escape_string(&options.locale);
    let locale_literal = string_literal(&options.locale);
    output.push_str(&format!(
        "import {locale} from \"./locales/{locale_path}\";\n\n"
    ));
    output.push_str(&format!(
        "const localeModules = {{ {locale} }} as const;\n\n"
    ));
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str(&format!(
        "type LinguiniLanguageInput = LinguiniLanguage | {locale_literal};\n\n"
    ));
    output.push_str(&format!(
        "let currentLanguage: () => LinguiniLanguageInput = () => {locale_literal};\n\n"
    ));
    output.push_str("function resolveLinguiniLanguage(): LinguiniLanguageInput {\n");
    output.push_str("  return currentLanguage();\n");
    output.push_str("}\n\n");
    output
        .push_str("export function createLinguini(language: LinguiniLanguageInput): Linguini {\n");
    output.push_str("  return localeModules[language as LinguiniLanguage];\n");
    output.push_str("}\n\n");
    output.push_str("export function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini {\n");
    output.push_str("  if (typeof options.language === \"function\") {\n");
    output.push_str("    currentLanguage = options.language;\n");
    output.push_str("  } else {\n");
    output.push_str("    const language = options.language;\n");
    output.push_str("    currentLanguage = () => language;\n");
    output.push_str("  }\n");
    output.push_str("  return lgl;\n");
    output.push_str("}\n\n");
    output.push_str("export const lgl: Linguini = new Proxy({} as Linguini, {\n");
    output.push_str("  get(_target, property) {\n");
    output.push_str(
        "    return createLinguini(resolveLinguiniLanguage())[property as keyof Linguini];\n",
    );
    output.push_str("  },\n");
    output.push_str("});\n");
}

fn emit_message_function(
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
    let body = message_body(signature, implementation, options);
    output.push_str(&format!(
        "export function {}({params}): string {{\n  return {body};\n}}\n\n",
        function_name(&signature.name)
    ));
    true
}

fn emit_group_object(
    group: &str,
    signatures: &[IrMessage],
    locale: &IrModule,
    options: &TypeScriptOptions,
    output: &mut String,
) {
    output.push_str(&format!("export const {group} = {{\n"));
    for signature in signatures {
        let property = signature.name.split('.').nth(1).unwrap_or(&signature.name);
        if let Some(implementation) = message_implementation(locale, &signature.name) {
            output.push_str(&format!(
                "  {}: {},\n",
                property_key(property),
                group_property_value(signature, implementation, options)
            ));
        }
    }
    output.push_str("} as const;\n\n");
}

fn group_property_value(
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
            message_body(signature, implementation, options)
        )
    }
}

fn message_body(
    signature: &IrMessage,
    implementation: &IrMessage,
    options: &TypeScriptOptions,
) -> String {
    let context = signature
        .parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    implementation
        .body
        .as_ref()
        .map(|body| text_expression_with_context(body, &context, options))
        .unwrap_or_else(|| "\"\"".to_owned())
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

fn message_implementation<'a>(module: &'a IrModule, name: &str) -> Option<&'a IrMessage> {
    module.messages.iter().find(|message| message.name == name)
}

fn tuple_condition(parameters: &[String], names: &[String]) -> String {
    parameters
        .iter()
        .zip(names)
        .map(|(parameter, name)| format!("{parameter} === {}", string_literal(name)))
        .collect::<Vec<_>>()
        .join(" && ")
}
