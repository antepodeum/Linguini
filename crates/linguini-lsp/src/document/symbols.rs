use super::{LinguiniDocument, Symbol};
use linguini_format::SourceKind;
use linguini_syntax::{
    parse_locale_with_recovery, parse_schema_with_recovery, DocComment, LocaleDeclaration,
    MessageSignature, Name, Parameter, SchemaDeclaration, TextPart, TextPattern,
};

pub(super) fn symbols(document: &LinguiniDocument) -> Vec<Symbol> {
    match document.kind {
        SourceKind::Schema => parse_schema_with_recovery(&document.text)
            .ast
            .map(|file| {
                file.declarations
                    .iter()
                    .flat_map(schema_declaration_symbols)
                    .collect()
            })
            .unwrap_or_default(),
        SourceKind::Locale => parse_locale_with_recovery(&document.text)
            .ast
            .map(|file| {
                file.declarations
                    .iter()
                    .flat_map(locale_declaration_symbols)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn schema_declaration_symbols(declaration: &SchemaDeclaration) -> Vec<Symbol> {
    match declaration {
        SchemaDeclaration::Enum(item) => vec![symbol(&item.name, "enum", &item.docs)],
        SchemaDeclaration::TypeAlias(item) => vec![symbol(&item.name, "type", &item.docs)],
        SchemaDeclaration::Message(item) => {
            vec![schema_message_symbol(item, None)]
        }
        SchemaDeclaration::Group(item) => {
            let mut output = vec![symbol(&item.name, "message group", &item.docs)];
            output.extend(
                item.messages
                    .iter()
                    .map(|message| schema_message_symbol(message, Some(&item.name.value))),
            );
            output
        }
    }
}

fn locale_declaration_symbols(declaration: &LocaleDeclaration) -> Vec<Symbol> {
    match declaration {
        LocaleDeclaration::Enum(item) => vec![symbol(&item.name, "enum", &item.docs)],
        LocaleDeclaration::Form(item) => vec![symbol(&item.name, "impl", &item.docs)],
        LocaleDeclaration::Function(item) => vec![symbol(&item.name, "function", &item.docs)],
        LocaleDeclaration::Message(item) => vec![locale_message_symbol(
            &item.name,
            None,
            &item.docs,
            &item.value,
        )],
        LocaleDeclaration::Group(item) => {
            let mut output = vec![symbol(&item.name, "message group", &item.docs)];
            output.extend(item.messages.iter().map(|message| {
                locale_message_symbol(
                    &message.name,
                    Some(&item.name.value),
                    &message.docs,
                    &message.value,
                )
            }));
            output
        }
        LocaleDeclaration::Override(inner) => locale_declaration_symbols(inner),
    }
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

fn schema_message_symbol(message: &MessageSignature, group: Option<&str>) -> Symbol {
    let mut symbol = symbol(&message.name, "message", &message.docs);
    let display_name = qualified_name(group, &message.name.value);
    symbol.preview = Some(schema_message_preview(&display_name, &message.parameters));
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

fn schema_message_preview(name: &str, parameters: &[Parameter]) -> String {
    let arguments = parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}: {}",
                parameter.name.value,
                sample_value_for_type(&parameter.ty.value)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name}({arguments})")
}

fn sample_value_for_type(ty: &str) -> &'static str {
    match ty {
        "Number" => "3",
        "Decimal" => "12.5",
        "String" => "\"Sample\"",
        "Date" => "2026-05-15",
        _ => "sample",
    }
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

