mod render;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::io::{self, IsTerminal};
use std::path::Path;

use linguini_ir::{lower_locale, lower_schema, IrMessage, IrModule};

use crate::{CliError, CliResult};

use super::codegen::{merge_module, merge_module_fallback, namespaced_module};
use super::io::read_project_config;
use super::sources::{load_locale_sources, load_schema_sources, locale_index};
use super::ParsedSchemaSource;

const MAX_SAMPLE_CASES_PER_MESSAGE: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SampleValue {
    String(String),
    Number(String),
    Boolean(bool),
}

impl SampleValue {
    pub(super) fn as_text(&self) -> String {
        match self {
            Self::String(value) | Self::Number(value) => value.clone(),
            Self::Boolean(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SampleError {
    AliasCycle {
        chain: Vec<String>,
    },
    CaseLimit {
        message: String,
        limit: usize,
    },
    Render {
        locale: String,
        message: String,
        source: render::RenderError,
    },
}

impl fmt::Display for SampleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AliasCycle { chain } => {
                write!(
                    formatter,
                    "sample type alias cycle detected: {}",
                    chain.join(" -> ")
                )
            }
            Self::CaseLimit { message, limit } => write!(
                formatter,
                "message `{message}` needs more than {limit} bounded sample cases"
            ),
            Self::Render {
                locale,
                message,
                source,
            } => write!(
                formatter,
                "could not render locale `{locale}` message `{message}`: {source}"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OutputStyle {
    color: bool,
}

impl OutputStyle {
    fn stdout() -> Self {
        Self::for_capabilities(
            io::stdout().is_terminal(),
            env::var_os("NO_COLOR").is_some(),
        )
    }

    const fn for_capabilities(is_terminal: bool, no_color: bool) -> Self {
        Self {
            color: is_terminal && !no_color,
        }
    }

    #[cfg(test)]
    const fn plain() -> Self {
        Self { color: false }
    }

    #[cfg(test)]
    const fn colored() -> Self {
        Self { color: true }
    }
}

pub(crate) fn generate_project_data(root: &Path) -> CliResult<String> {
    generate_project_data_with_style(root, OutputStyle::stdout())
}

fn generate_project_data_with_style(root: &Path, style: OutputStyle) -> CliResult<String> {
    let config = read_project_config(root)?;
    let schema_sources = load_schema_sources(root, &config)?;
    let schema = load_merged_schema(&schema_sources)?;
    let locales = load_locale_sources(root, &config)?;
    let locale_index = locale_index(&locales)?;
    let mut locale_modules = BTreeMap::new();

    for locale in config.project().locales() {
        let mut module = IrModule::default();
        for source in &schema_sources {
            let namespace = &source.file.namespace;
            let locale_key = (namespace.clone(), locale.clone());
            let default_key = (
                namespace.clone(),
                config.project().default_locale().to_owned(),
            );
            if let Some(locale_file) = locale_index.get(&locale_key) {
                merge_module(
                    &mut module,
                    namespaced_module(lower_locale(&locale_file.ast), namespace),
                )?;
            }
            if locale != config.project().default_locale() {
                if let Some(default_file) = locale_index.get(&default_key) {
                    merge_module_fallback(
                        &mut module,
                        namespaced_module(lower_locale(&default_file.ast), namespace),
                    )?;
                }
            }
        }
        locale_modules.insert(locale.clone(), module);
    }

    render_generated_data(&schema, &locale_modules, style)
        .map_err(|error| CliError::Diagnostics(format!("generate: {error}\n")))
}

fn load_merged_schema(schema_sources: &[ParsedSchemaSource]) -> CliResult<IrModule> {
    let mut schema = IrModule::default();
    for source in schema_sources {
        merge_module(
            &mut schema,
            namespaced_module(lower_schema(&source.ast), &source.file.namespace),
        )?;
    }
    Ok(schema)
}

fn render_generated_data(
    schema: &IrModule,
    locales: &BTreeMap<String, IrModule>,
    style: OutputStyle,
) -> Result<String, SampleError> {
    let mut output = String::new();
    output.push_str(&format!(
        "{} {}\n",
        color("linguini", Style::BoldCyan, style),
        color("generate", Style::BoldWhite, style)
    ));

    for (locale, module) in locales {
        output.push('\n');
        output.push_str(&format!(
            "{} {}\n",
            color("locale", Style::Blue, style),
            color(locale, Style::BoldWhite, style)
        ));
        for message in schema.messages() {
            render_message_cases(schema, module, locale, message, style, &mut output)?;
        }
    }

    Ok(output)
}

fn render_message_cases(
    schema: &IrModule,
    module: &IrModule,
    locale: &str,
    message: &IrMessage,
    style: OutputStyle,
    output: &mut String,
) -> Result<(), SampleError> {
    output.push_str(&format!(
        "  {} {}\n",
        color("message", Style::Magenta, style),
        color(&message.name, Style::BoldWhite, style)
    ));
    let cases = message_cases(schema, message)?;
    let renderer = render::Renderer::new(schema, module, locale);

    for inputs in &cases {
        output.push_str("    ");
        output.push_str(&format_args(inputs, style));
        output.push('\n');
        let rendered = renderer
            .render_message(&message.name, inputs)
            .map_err(|source| SampleError::Render {
                locale: locale.to_owned(),
                message: message.name.clone(),
                source,
            })?;
        output.push_str(&format!(
            "      {} {rendered}\n",
            color("=>", Style::Green, style)
        ));
    }
    Ok(())
}

fn message_cases(
    schema: &IrModule,
    message: &IrMessage,
) -> Result<Vec<BTreeMap<String, SampleValue>>, SampleError> {
    let values = message
        .parameters
        .iter()
        .map(|parameter| sample_values(schema, &parameter.ty).map(|values| (parameter, values)))
        .collect::<Result<Vec<_>, _>>()?;

    if values.is_empty() {
        return Ok(vec![BTreeMap::new()]);
    }

    let case_count = values
        .iter()
        .try_fold(1usize, |count, (_, values)| {
            count
                .checked_add(values.len().saturating_sub(1))
                .filter(|count| *count <= MAX_SAMPLE_CASES_PER_MESSAGE)
        })
        .ok_or_else(|| SampleError::CaseLimit {
            message: message.name.clone(),
            limit: MAX_SAMPLE_CASES_PER_MESSAGE,
        })?;

    let baseline = values
        .iter()
        .map(|(parameter, values)| {
            (
                parameter.name.clone(),
                values
                    .first()
                    .cloned()
                    .expect("every type has at least one sample value"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut cases = Vec::with_capacity(case_count);
    cases.push(baseline.clone());

    // Vary one parameter at a time so every sample appears without a Cartesian explosion.
    for (parameter, parameter_values) in &values {
        for value in parameter_values.iter().skip(1) {
            let mut sample = baseline.clone();
            sample.insert(parameter.name.clone(), value.clone());
            cases.push(sample);
        }
    }

    Ok(cases)
}

fn sample_values(schema: &IrModule, ty: &str) -> Result<Vec<SampleValue>, SampleError> {
    let resolved = resolve_type(schema, ty)?;
    if let Some(enumeration) = schema
        .enums()
        .iter()
        .find(|item| item.name == ty || item.name == resolved)
    {
        let values = enumeration
            .variants
            .iter()
            .map(|variant| SampleValue::String(variant.clone()))
            .collect::<Vec<_>>();
        return Ok(if values.is_empty() {
            vec![SampleValue::String("sample".to_owned())]
        } else {
            values
        });
    }

    match resolved.as_str() {
        "Number" => Ok(["0", "1", "2", "5", "-1", "9007199254740991"]
            .into_iter()
            .map(|value| SampleValue::Number(value.to_owned()))
            .collect()),
        "Decimal" => Ok(["0", "1.25", "-1.5", "1234567890.123456789"]
            .into_iter()
            .map(|value| SampleValue::Number(value.to_owned()))
            .collect()),
        "Date" => Ok(vec![SampleValue::String("2026-05-13".to_owned())]),
        "Boolean" => Ok(vec![
            SampleValue::Boolean(false),
            SampleValue::Boolean(true),
        ]),
        _ => Ok(vec![SampleValue::String("sample".to_owned())]),
    }
}

fn resolve_type(schema: &IrModule, ty: &str) -> Result<String, SampleError> {
    let mut current = ty;
    let mut visited = BTreeSet::new();
    let mut chain = Vec::new();

    loop {
        if !visited.insert(current.to_owned()) {
            chain.push(current.to_owned());
            return Err(SampleError::AliasCycle { chain });
        }
        chain.push(current.to_owned());

        let Some(alias) = schema
            .type_aliases()
            .iter()
            .find(|alias| alias.name == current)
        else {
            return Ok(current.to_owned());
        };
        current = &alias.target;
    }
}

fn format_args(inputs: &BTreeMap<String, SampleValue>, style: OutputStyle) -> String {
    if inputs.is_empty() {
        return color("(no args)", Style::Dim, style);
    }

    inputs
        .iter()
        .map(|(name, value)| {
            format!(
                "{}{}{}",
                color(name, Style::Yellow, style),
                color("=", Style::Dim, style),
                color(&value.as_text(), Style::Cyan, style)
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

fn color(value: &str, style: Style, output_style: OutputStyle) -> String {
    if !output_style.color {
        return value.to_owned();
    }

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

    fn schema(source: &str) -> IrModule {
        lower_schema(&parse_schema(source).expect("schema"))
    }

    #[test]
    fn generated_data_covers_every_enum_and_number_sample() {
        let schema = schema("enum Fruit { apple pear }\ndelivery(fruit: Fruit, count: Number)\n");
        let locale = lower_locale(
            &parse_locale(
                "impl Fruit {\n  apple {\n    form nom(Plural) {\n      one => apple\n      _ => apples\n    }\n  }\n  pear {\n    form nom(Plural) {\n      one => pear\n      _ => pears\n    }\n  }\n}\ndelivery = {count} {fruit.nom(count)}\n",
            )
            .expect("locale"),
        );
        let mut locales = BTreeMap::new();
        locales.insert("en".to_owned(), locale);

        let output = render_generated_data(&schema, &locales, OutputStyle::colored())
            .expect("render samples");
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

    #[test]
    fn plain_style_never_emits_ansi() {
        let mut locales = BTreeMap::new();
        locales.insert(
            "en".to_owned(),
            lower_locale(&parse_locale("hello = Hello\n").expect("locale")),
        );

        let output = render_generated_data(&schema("hello\n"), &locales, OutputStyle::plain())
            .expect("render samples");

        assert!(!output.contains('\x1b'));
    }

    #[test]
    fn no_color_overrides_terminal_capability() {
        assert!(OutputStyle::for_capabilities(true, false).color);
        assert!(!OutputStyle::for_capabilities(true, true).color);
        assert!(!OutputStyle::for_capabilities(false, false).color);
    }

    #[test]
    fn alias_cycles_are_reported_without_recursion() {
        let schema = schema("type A = B\ntype B = A\nmessage(value: A)\n");
        let message = schema.messages().first().expect("message");

        let error = message_cases(&schema, message).expect_err("cycle must fail");

        assert_eq!(
            error,
            SampleError::AliasCycle {
                chain: vec!["A".to_owned(), "B".to_owned(), "A".to_owned()]
            }
        );
    }

    #[test]
    fn aliases_to_enums_keep_enum_samples() {
        let schema =
            schema("enum Fruit { apple pear }\ntype Produce = Fruit\nmessage(value: Produce)\n");

        let values = sample_values(&schema, "Produce").expect("enum alias samples");

        assert_eq!(
            values,
            vec![
                SampleValue::String("apple".to_owned()),
                SampleValue::String("pear".to_owned())
            ]
        );
    }

    #[test]
    fn cases_grow_linearly_and_cover_each_parameter_value() {
        let schema = schema(
            "enum A { a1 a2 a3 }\nenum B { b1 b2 b3 }\nenum C { c1 c2 c3 }\nmessage(a: A, b: B, c: C)\n",
        );
        let cases = message_cases(&schema, schema.messages().first().expect("message"))
            .expect("sample cases");

        assert_eq!(cases.len(), 7);
        for expected in ["a1", "a2", "a3", "b1", "b2", "b3", "c1", "c2", "c3"] {
            assert!(cases
                .iter()
                .any(|case| case.values().any(|value| value.as_text() == expected)));
        }
    }

    #[test]
    fn canonical_types_include_precise_and_large_samples() {
        let schema =
            schema("message(number: Number, decimal: Decimal, date: Date, flag: Boolean)\n");
        let cases = message_cases(&schema, schema.messages().first().expect("message"))
            .expect("sample cases");

        assert!(cases.iter().any(|case| {
            case.get("number").map(SampleValue::as_text).as_deref() == Some("9007199254740991")
        }));
        assert!(cases.iter().any(|case| {
            case.get("decimal").map(SampleValue::as_text).as_deref() == Some("1234567890.123456789")
        }));
        assert!(cases
            .iter()
            .any(|case| { case.get("flag").map(SampleValue::as_text).as_deref() == Some("true") }));
    }

    #[test]
    fn namespace_qualification_matches_codegen_messages() {
        let qualified = namespaced_module(schema("delivery\n"), "shop.checkout");

        assert_eq!(qualified.messages()[0].name, "shop.checkout.delivery");
    }

    #[test]
    fn missing_locale_message_is_a_render_error() {
        let mut locales = BTreeMap::new();
        locales.insert("en".to_owned(), IrModule::default());

        let error = render_generated_data(&schema("hello\n"), &locales, OutputStyle::plain())
            .expect_err("missing implementation must fail");

        assert!(error
            .to_string()
            .contains("message implementation `hello` is missing"));
    }
}
