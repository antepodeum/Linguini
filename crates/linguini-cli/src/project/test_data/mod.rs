mod render;

use std::collections::BTreeMap;
use std::path::Path;

use linguini_config::LinguiniConfig;
use linguini_ir::{lower_locale, lower_schema, IrMessage, IrModule, IrParameter};

use crate::CliResult;

use super::codegen::{merge_module, merge_module_fallback};
use super::io::read_project_config;
use super::sources::{load_locale_sources, load_schema_sources, locale_index};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum SampleValue {
    String(String),
    Number(i64),
}

impl SampleValue {
    fn as_json(&self) -> String {
        match self {
            Self::String(value) => json_string(value),
            Self::Number(value) => value.to_string(),
        }
    }

    pub(super) fn as_text(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
        }
    }
}

pub(crate) fn generate_test_data(root: &Path) -> CliResult<String> {
    let config = read_project_config(root)?;
    let schema = load_merged_schema(root, &config)?;
    let locales = load_locale_sources(root, &config)?;
    let locale_index = locale_index(&locales);
    let mut locale_modules = BTreeMap::new();

    for locale in &config.project.locales {
        let mut module = IrModule::default();
        for source in load_schema_sources(root, &config)? {
            let namespace = source.file.namespace;
            let locale_key = (namespace.clone(), locale.clone());
            let default_key = (namespace, config.project.default_locale.clone());
            if let Some(locale_file) = locale_index.get(&locale_key) {
                merge_module(&mut module, lower_locale(&locale_file.ast));
            }
            if locale != &config.project.default_locale {
                if let Some(default_file) = locale_index.get(&default_key) {
                    merge_module_fallback(&mut module, lower_locale(&default_file.ast));
                }
            }
        }
        locale_modules.insert(locale.clone(), module);
    }

    Ok(render_test_data(&schema, &locale_modules))
}

fn load_merged_schema(root: &Path, config: &LinguiniConfig) -> CliResult<IrModule> {
    let mut schema = IrModule::default();
    for source in load_schema_sources(root, config)? {
        merge_module(&mut schema, lower_schema(&source.ast));
    }
    Ok(schema)
}

fn render_test_data(schema: &IrModule, locales: &BTreeMap<String, IrModule>) -> String {
    let mut output = String::new();
    output.push_str("{\n  \"locales\": {\n");

    for (locale_index, (locale, module)) in locales.iter().enumerate() {
        if locale_index > 0 {
            output.push_str(",\n");
        }
        output.push_str(&format!("    {}: {{\n", json_string(locale)));
        output.push_str("      \"messages\": {\n");
        for (message_index, message) in schema.messages.iter().enumerate() {
            if message_index > 0 {
                output.push_str(",\n");
            }
            render_message_cases(schema, module, locale, message, &mut output);
        }
        output.push_str("\n      }\n    }");
    }

    output.push_str("\n  }\n}\n");
    output
}

fn render_message_cases(
    schema: &IrModule,
    module: &IrModule,
    locale: &str,
    message: &IrMessage,
    output: &mut String,
) {
    output.push_str(&format!("        {}: [", json_string(&message.name)));
    let cases = message_cases(schema, message);
    let renderer = render::Renderer::new(schema, module, locale);

    for (index, inputs) in cases.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str("\n          { \"args\": ");
        output.push_str(&json_args(inputs));
        output.push_str(", \"output\": ");
        output.push_str(&json_string(
            &renderer.render_message(&message.name, inputs),
        ));
        output.push_str(" }");
    }
    if !cases.is_empty() {
        output.push('\n');
        output.push_str("        ");
    }
    output.push(']');
}

fn message_cases(schema: &IrModule, message: &IrMessage) -> Vec<BTreeMap<String, SampleValue>> {
    let mut cases = vec![BTreeMap::new()];
    for parameter in &message.parameters {
        cases = expand_cases(schema, cases, parameter);
    }
    cases
}

fn expand_cases(
    schema: &IrModule,
    cases: Vec<BTreeMap<String, SampleValue>>,
    parameter: &IrParameter,
) -> Vec<BTreeMap<String, SampleValue>> {
    let values = sample_values(schema, &parameter.ty);
    let mut expanded = Vec::new();
    for case in cases {
        for value in &values {
            let mut next = case.clone();
            next.insert(parameter.name.clone(), value.clone());
            expanded.push(next);
        }
    }
    expanded
}

fn sample_values(schema: &IrModule, ty: &str) -> Vec<SampleValue> {
    let resolved = resolve_type(schema, ty);
    if let Some(enumeration) = schema.enums.iter().find(|item| item.name == ty) {
        return enumeration
            .variants
            .iter()
            .map(|variant| SampleValue::String(variant.clone()))
            .collect();
    }

    match resolved.as_str() {
        "Number" | "Decimal" | "Integer" => {
            [0, 1, 2, 5].into_iter().map(SampleValue::Number).collect()
        }
        "Date" | "DateTime" => vec![SampleValue::String("2026-05-13".to_owned())],
        _ => vec![SampleValue::String("sample".to_owned())],
    }
}

fn resolve_type(schema: &IrModule, ty: &str) -> String {
    schema
        .type_aliases
        .iter()
        .find(|alias| alias.name == ty)
        .map(|alias| resolve_type(schema, &alias.target))
        .unwrap_or_else(|| ty.to_owned())
}

fn json_args(inputs: &BTreeMap<String, SampleValue>) -> String {
    let items = inputs
        .iter()
        .map(|(name, value)| format!("{}: {}", json_string(name), value.as_json()))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {items} }}")
}

pub(super) fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguini_syntax::{parse_locale, parse_schema};

    #[test]
    fn test_data_covers_every_enum_and_number_sample() {
        let schema = lower_schema(
            &parse_schema("enum Fruit { apple pear }\ndelivery(fruit: Fruit, count: Number)\n")
                .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "form Fruit {\n  apple {\n    nom {\n      one => apple\n      other => apples\n    }\n  }\n  pear {\n    nom {\n      one => pear\n      other => pears\n    }\n  }\n}\ndelivery = {count} {fruit.nom(count)}\n",
            )
            .expect("locale"),
        );
        let mut locales = BTreeMap::new();
        locales.insert("en".to_owned(), locale);

        let json = render_test_data(&schema, &locales);

        assert!(json.contains("\"fruit\": \"apple\""));
        assert!(json.contains("\"fruit\": \"pear\""));
        assert!(json.contains("\"count\": 5"));
        assert!(json.contains("\"output\": \"1 apple\""));
        assert!(json.contains("\"output\": \"5 apples\""));
    }
}
