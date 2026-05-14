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
    pub(super) fn as_text(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
        }
    }
}

pub(crate) fn generate_project_data(root: &Path) -> CliResult<String> {
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

    Ok(render_generated_data(&schema, &locale_modules))
}

fn load_merged_schema(root: &Path, config: &LinguiniConfig) -> CliResult<IrModule> {
    let mut schema = IrModule::default();
    for source in load_schema_sources(root, config)? {
        merge_module(&mut schema, lower_schema(&source.ast));
    }
    Ok(schema)
}

fn render_generated_data(schema: &IrModule, locales: &BTreeMap<String, IrModule>) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{} {}\n",
        color("linguini", Style::BoldCyan),
        color("generate", Style::BoldWhite)
    ));

    for (locale, module) in locales {
        output.push('\n');
        output.push_str(&format!(
            "{} {}\n",
            color("locale", Style::Blue),
            color(locale, Style::BoldWhite)
        ));
        for message in &schema.messages {
            render_message_cases(schema, module, locale, message, &mut output);
        }
    }

    output
}

fn render_message_cases(
    schema: &IrModule,
    module: &IrModule,
    locale: &str,
    message: &IrMessage,
    output: &mut String,
) {
    output.push_str(&format!(
        "  {} {}\n",
        color("message", Style::Magenta),
        color(&message.name, Style::BoldWhite)
    ));
    let cases = message_cases(schema, message);
    let renderer = render::Renderer::new(schema, module, locale);

    for inputs in &cases {
        output.push_str("    ");
        output.push_str(&format_args(inputs));
        output.push('\n');
        output.push_str(&format!(
            "      {} {}\n",
            color("=>", Style::Green),
            renderer.render_message(&message.name, inputs)
        ));
    }
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

fn format_args(inputs: &BTreeMap<String, SampleValue>) -> String {
    if inputs.is_empty() {
        return color("(no args)", Style::Dim).to_string();
    }

    inputs
        .iter()
        .map(|(name, value)| {
            format!(
                "{}{}{}",
                color(name, Style::Yellow),
                color("=", Style::Dim),
                color(&value.as_text(), Style::Cyan)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone, Copy)]
enum Style {
    Blue,
    BoldCyan,
    BoldWhite,
    Cyan,
    Dim,
    Green,
    Magenta,
    Yellow,
}

fn color(value: &str, style: Style) -> String {
    let code = match style {
        Style::Blue => "34",
        Style::BoldCyan => "1;36",
        Style::BoldWhite => "1;37",
        Style::Cyan => "36",
        Style::Dim => "2",
        Style::Green => "32",
        Style::Magenta => "35",
        Style::Yellow => "33",
    };
    format!("\x1b[{code}m{value}\x1b[0m")
}

#[cfg(test)]
fn strip_ansi(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if code == 'm' {
                    break;
                }
            }
            continue;
        }
        output.push(character);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguini_syntax::{parse_locale, parse_schema};

    #[test]
    fn generated_data_covers_every_enum_and_number_sample() {
        let schema = lower_schema(
            &parse_schema("enum Fruit { apple pear }\ndelivery(fruit: Fruit, count: Number)\n")
                .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "impl Fruit {\n  apple {\n    form nom(Plural) {\n      one => apple\n      _ => apples\n    }\n  }\n  pear {\n    form nom(Plural) {\n      one => pear\n      _ => pears\n    }\n  }\n}\ndelivery = {count} {fruit.nom(count)}\n",
            )
            .expect("locale"),
        );
        let mut locales = BTreeMap::new();
        locales.insert("en".to_owned(), locale);

        let output = render_generated_data(&schema, &locales);
        let plain = strip_ansi(&output);

        assert!(output.contains("\x1b["));
        assert!(plain.contains("locale en"));
        assert!(plain.contains("message delivery"));
        assert!(plain.contains("fruit=apple"));
        assert!(plain.contains("fruit=pear"));
        assert!(plain.contains("count=5"));
        assert!(plain.contains("=> 1 apple"));
        assert!(plain.contains("=> 5 apples"));
    }
}
