use super::{parsed_locale, parsed_schema, LinguiniDocument, Symbol};
use linguini_format::SourceKind;
use linguini_syntax::{
    DocComment, LocaleDeclaration, MessageSignature, Name, Parameter, SchemaDeclaration,
    SchemaFile, TextPart, TextPattern,
};
use std::collections::BTreeMap;

pub(super) fn symbols(document: &LinguiniDocument) -> Vec<Symbol> {
    match document.kind {
        SourceKind::Schema => parsed_schema(document)
            .and_then(|parsed| parsed.ast.as_ref())
            .map(|file| {
                let samples = schema_sample_values(file);
                file.declarations
                    .iter()
                    .flat_map(|declaration| schema_declaration_symbols(declaration, &samples))
                    .collect()
            })
            .unwrap_or_default(),
        SourceKind::Locale => parsed_locale(document)
            .and_then(|parsed| parsed.ast.as_ref())
            .map(|file| {
                file.declarations
                    .iter()
                    .flat_map(locale_declaration_symbols)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn schema_declaration_symbols(
    declaration: &SchemaDeclaration,
    samples: &BTreeMap<String, String>,
) -> Vec<Symbol> {
    match declaration {
        SchemaDeclaration::Enum(item) => vec![symbol(&item.name, "enum", &item.docs)],
        SchemaDeclaration::TypeAlias(item) => vec![symbol(&item.name, "type", &item.docs)],
        SchemaDeclaration::Message(item) => {
            vec![schema_message_symbol(item, None, samples)]
        }
        SchemaDeclaration::Group(item) => schema_group_symbols(item, None, samples),
    }
}

fn schema_group_symbols(
    group: &linguini_syntax::MessageGroup,
    parent: Option<&str>,
    samples: &BTreeMap<String, String>,
) -> Vec<Symbol> {
    let path = qualified_name(parent, &group.name.value);
    let mut output = vec![symbol(&group.name, "message group", &group.docs)];
    output.extend(
        group
            .messages
            .iter()
            .map(|message| schema_message_symbol(message, Some(&path), samples)),
    );
    for child in &group.groups {
        output.extend(schema_group_symbols(child, Some(&path), samples));
    }
    output
}

fn locale_declaration_symbols(declaration: &LocaleDeclaration) -> Vec<Symbol> {
    match declaration {
        LocaleDeclaration::Enum(item) => vec![symbol(&item.name, "enum", &item.docs)],
        LocaleDeclaration::Variable(item) => vec![symbol(&item.name, "variable", &item.docs)],
        LocaleDeclaration::Form(item) => vec![symbol(&item.name, "impl", &item.docs)],
        LocaleDeclaration::Function(item) => vec![symbol(&item.name, "function", &item.docs)],
        LocaleDeclaration::Message(item) => vec![locale_message_symbol(
            &item.name,
            None,
            &item.docs,
            &item.value,
        )],
        LocaleDeclaration::Group(item) => locale_group_symbols(item, None),
        LocaleDeclaration::Override(inner) => locale_declaration_symbols(inner),
    }
}

fn locale_group_symbols(
    group: &linguini_syntax::MessageImplementationGroup,
    parent: Option<&str>,
) -> Vec<Symbol> {
    let path = qualified_name(parent, &group.name.value);
    let mut output = vec![symbol(&group.name, "message group", &group.docs)];
    output.extend(group.messages.iter().map(|message| {
        locale_message_symbol(&message.name, Some(&path), &message.docs, &message.value)
    }));
    for child in &group.groups {
        output.extend(locale_group_symbols(child, Some(&path)));
    }
    output
}

fn symbol(name: &Name, detail: &str, docs: &[DocComment]) -> Symbol {
    Symbol {
        name: name.value.clone(),
        detail: detail.to_owned(),
        span: name.span,
        docs: docs.iter().map(|doc| doc.text.trim().to_owned()).collect(),
        preview: None,
    }
}

fn schema_message_symbol(
    message: &MessageSignature,
    group: Option<&str>,
    samples: &BTreeMap<String, String>,
) -> Symbol {
    let mut symbol = symbol(&message.name, "message", &message.docs);
    let display_name = qualified_name(group, &message.name.value);
    symbol.preview = Some(schema_message_preview(
        &display_name,
        &message.parameters,
        samples,
    ));
    symbol
}

fn locale_message_symbol(
    name: &Name,
    group: Option<&str>,
    docs: &[DocComment],
    value: &TextPattern,
) -> Symbol {
    let mut symbol = symbol(name, "message", docs);
    let display_name = qualified_name(group, &name.value);
    symbol.preview = Some(format!("{display_name} -> {}", text_preview(value)));
    symbol
}

fn schema_message_preview(
    name: &str,
    parameters: &[Parameter],
    samples: &BTreeMap<String, String>,
) -> String {
    let arguments = parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}: {}",
                parameter.name.value,
                sample_value_for_type(&parameter.ty.value, samples)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name}({arguments})")
}

fn sample_value_for_type(ty: &str, samples: &BTreeMap<String, String>) -> String {
    match ty {
        "Number" => "3".to_owned(),
        "Decimal" => "12.5".to_owned(),
        "String" => "\"Sample\"".to_owned(),
        "Date" => "2026-05-15".to_owned(),
        _ => samples
            .get(ty)
            .cloned()
            .unwrap_or_else(|| "sample".to_owned()),
    }
}

fn schema_sample_values(file: &SchemaFile) -> BTreeMap<String, String> {
    file.declarations
        .iter()
        .filter_map(|declaration| match declaration {
            SchemaDeclaration::Enum(item) => item
                .variants
                .first()
                .map(|variant| (item.name.value.clone(), variant.value.clone())),
            SchemaDeclaration::TypeAlias(item) => Some((
                item.name.value.clone(),
                sample_value_for_type(&item.target.value, &BTreeMap::new()),
            )),
            SchemaDeclaration::Message(_) | SchemaDeclaration::Group(_) => None,
        })
        .collect()
}

fn text_preview(value: &TextPattern) -> String {
    value
        .parts
        .iter()
        .map(|part| match part {
            TextPart::Text(text) => text.value.clone(),
            TextPart::Placeholder(placeholder) => {
                format!("{{{}}}", expression_preview(&placeholder.expression))
            }
        })
        .collect::<String>()
}

fn expression_preview(expression: &linguini_syntax::Expression) -> String {
    if let linguini_syntax::ExpressionKind::InlineFunction { inputs, .. } = &expression.kind {
        return format!(
            "fn({}) {{ … }}",
            inputs
                .iter()
                .map(|input| match input {
                    linguini_syntax::InlineFunctionInput::Selector { value, .. } => {
                        expression_preview(value)
                    }
                    linguini_syntax::InlineFunctionInput::Binding { name, value, .. } => {
                        format!("{}: {}", name.value, expression_preview(value))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let mut output = expression
        .path
        .iter()
        .map(|name| name.value.as_str())
        .collect::<Vec<_>>()
        .join(".");
    if !expression.arguments.is_empty() {
        output.push('(');
        output.push_str(
            &expression
                .arguments
                .iter()
                .map(expression_preview)
                .collect::<Vec<_>>()
                .join(", "),
        );
        output.push(')');
    }
    output
}

fn qualified_name(group: Option<&str>, name: &str) -> String {
    match group {
        Some(group) => format!("{group}.{name}"),
        None => name.to_owned(),
    }
}
