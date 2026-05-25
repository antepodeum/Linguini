use chumsky::{input::ValueInput, prelude::*};

use crate::{
    Expression, FormAttribute, FormDeclaration, FormEntry, FormVariant, FunctionBranch,
    FunctionBranchValue, FunctionDeclaration, FunctionParameter, LocaleDeclaration, LocaleFile,
    LocaleValue, MapBranch, MessageImplementation, MessageImplementationGroup, Placeholder,
    RawText, Span, TextPart, TextPattern, TokenKind, VariableDeclaration,
};

use super::{annotation, doc_comment, enum_declaration, keyword, name, Extra};

pub(super) fn locale_parser<'tokens, I>(
) -> impl Parser<'tokens, I, LocaleFile, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    declaration()
        .repeated()
        .collect::<Vec<_>>()
        .then_ignore(end())
        .map_with(|declarations, extra| LocaleFile {
            declarations,
            span: extra.span(),
        })
}

fn declaration<'tokens, I>() -> impl Parser<'tokens, I, LocaleDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .then(choice((
            keyword("override")
                .ignore_then(declaration_body())
                .map(|declaration| LocaleDeclaration::Override(Box::new(declaration))),
            declaration_body(),
        )))
        .map(|(docs, mut declaration)| {
            declaration.set_docs(docs);
            declaration
        })
}

fn declaration_body<'tokens, I>(
) -> impl Parser<'tokens, I, LocaleDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    choice((
        enum_declaration().map(LocaleDeclaration::Enum),
        variable_declaration().map(LocaleDeclaration::Variable),
        impl_declaration().map(LocaleDeclaration::Form),
        form_function_declaration().map(LocaleDeclaration::Function),
        function_declaration().map(LocaleDeclaration::Function),
        group_or_message(),
    ))
}

fn variable_declaration<'tokens, I>(
) -> impl Parser<'tokens, I, VariableDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    keyword("let")
        .ignore_then(name())
        .then_ignore(just(TokenKind::Equals))
        .then(text_pattern())
        .map_with(|(name, value), extra| VariableDeclaration {
            docs: Vec::new(),
            name,
            value,
            span: extra.span(),
        })
}

fn impl_declaration<'tokens, I>() -> impl Parser<'tokens, I, FormDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    keyword("impl")
        .ignore_then(name())
        .then(
            form_variant()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace)),
        )
        .map_with(|(name, variants), extra| FormDeclaration {
            docs: Vec::new(),
            name,
            variants,
            span: extra.span(),
        })
}

fn form_variant<'tokens, I>() -> impl Parser<'tokens, I, FormVariant, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    name()
        .then(
            form_entry_parser()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace)),
        )
        .map_with(|(name, entries), extra| FormVariant {
            name,
            entries,
            span: extra.span(),
        })
}

fn form_entry_parser<'tokens, I>() -> impl Parser<'tokens, I, FormEntry, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    recursive(|entry| {
        let branch = map_branch().map(FormEntry::Branch);
        let attribute_name = keyword("form").or_not().ignore_then(name()).then_ignore(
            name()
                .then_ignore(just(TokenKind::Comma).or_not())
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
                .or_not(),
        );
        let attribute = attribute_name
            .then(choice((
                just(TokenKind::Equals)
                    .ignore_then(text_pattern())
                    .map(LocaleValue::Text),
                choice((
                    map_branch()
                        .repeated()
                        .at_least(1)
                        .collect::<Vec<_>>()
                        .map(LocaleValue::Map),
                    entry
                        .clone()
                        .repeated()
                        .at_least(1)
                        .collect::<Vec<_>>()
                        .map(LocaleValue::Object),
                ))
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace)),
            )))
            .map_with(|(name, value), extra| {
                FormEntry::Attribute(FormAttribute {
                    name,
                    value,
                    span: extra.span(),
                })
            });

        choice((branch, attribute))
    })
}

fn function_declaration<'tokens, I>(
) -> impl Parser<'tokens, I, FunctionDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    keyword("fn")
        .ignore_then(name())
        .then(function_parameters())
        .then(
            function_branch()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace)),
        )
        .map_with(
            |((name, parameters), branches), extra| FunctionDeclaration {
                docs: Vec::new(),
                name,
                parameters,
                branches,
                span: extra.span(),
            },
        )
}

fn form_function_declaration<'tokens, I>(
) -> impl Parser<'tokens, I, FunctionDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    keyword("form")
        .ignore_then(name())
        .then(function_parameters())
        .then(
            function_branch()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace)),
        )
        .map_with(
            |((name, parameters), branches), extra| FunctionDeclaration {
                docs: Vec::new(),
                name,
                parameters,
                branches,
                span: extra.span(),
            },
        )
}

fn function_parameters<'tokens, I>(
) -> impl Parser<'tokens, I, Vec<FunctionParameter>, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    name()
        .then(just(TokenKind::Colon).ignore_then(name()).or_not())
        .map_with(|(first, ty), extra| {
            if let Some(ty) = ty {
                FunctionParameter {
                    name: Some(first),
                    ty,
                    span: extra.span(),
                }
            } else {
                FunctionParameter {
                    name: None,
                    ty: first,
                    span: extra.span(),
                }
            }
        })
        .separated_by(just(TokenKind::Comma))
        .at_least(1)
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
}

fn function_branch<'tokens, I>() -> impl Parser<'tokens, I, FunctionBranch, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    recursive(|branch| {
        name()
            .then(choice((
                just(TokenKind::Arrow)
                    .ignore_then(text_pattern())
                    .map(FunctionBranchValue::Text),
                branch
                    .repeated()
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
                    .map(FunctionBranchValue::Dispatch),
            )))
            .map_with(|(key, value), extra| FunctionBranch {
                key,
                value,
                span: extra.span(),
            })
    })
}

fn group_or_message<'tokens, I>(
) -> impl Parser<'tokens, I, LocaleDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    name()
        .then(choice((
            just(TokenKind::Equals)
                .ignore_then(text_pattern())
                .map_with(|value, extra| MessageOrGroup::Message {
                    value,
                    span: extra.span(),
                }),
            message_implementation()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
                .map_with(|messages, extra| MessageOrGroup::Group {
                    messages,
                    span: extra.span(),
                }),
        )))
        .map(|(name, item)| match item {
            MessageOrGroup::Message { value, span } => {
                LocaleDeclaration::Message(MessageImplementation {
                    docs: Vec::new(),
                    span: name.span.union(span),
                    name,
                    value,
                })
            }
            MessageOrGroup::Group { messages, span } => {
                LocaleDeclaration::Group(MessageImplementationGroup {
                    docs: Vec::new(),
                    span: name.span.union(span),
                    name,
                    messages,
                })
            }
        })
}

fn message_implementation<'tokens, I>(
) -> impl Parser<'tokens, I, MessageImplementation, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .then(name())
        .then_ignore(just(TokenKind::Equals))
        .then(text_pattern())
        .map_with(|((docs, name), value), extra| MessageImplementation {
            docs,
            name,
            value,
            span: extra.span(),
        })
}

fn map_branch<'tokens, I>() -> impl Parser<'tokens, I, MapBranch, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    name()
        .separated_by(just(TokenKind::Comma))
        .at_least(1)
        .collect::<Vec<_>>()
        .then_ignore(just(TokenKind::Arrow))
        .then(text_pattern())
        .map_with(|(keys, value), extra| MapBranch {
            keys,
            value,
            span: extra.span(),
        })
}

fn text_pattern<'tokens, I>() -> impl Parser<'tokens, I, TextPattern, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    choice((
        select! { TokenKind::RawText(value) => value }.map_with(|value, extra| TextAtom {
            part: TextPart::Text(RawText {
                value,
                span: extra.span(),
            }),
            trim_edges: true,
        }),
        select! { TokenKind::String(value) => value }.map_with(|value, extra| TextAtom {
            part: TextPart::Text(RawText {
                value,
                span: extra.span(),
            }),
            trim_edges: false,
        }),
        placeholder().map(|placeholder| TextAtom {
            part: TextPart::Placeholder(placeholder),
            trim_edges: false,
        }),
    ))
    .repeated()
    .at_least(1)
    .collect::<Vec<_>>()
    .map_with(|atoms, extra| TextPattern {
        parts: trim_text_atoms(atoms),
        span: extra.span(),
    })
}

struct TextAtom {
    part: TextPart,
    trim_edges: bool,
}

fn trim_text_atoms(mut atoms: Vec<TextAtom>) -> Vec<TextPart> {
    if let Some(first) = atoms.first_mut() {
        if first.trim_edges {
            if let TextPart::Text(text) = &mut first.part {
                text.value = text.value.trim_start().to_owned();
            }
        }
    }

    if let Some(last) = atoms.last_mut() {
        if last.trim_edges {
            if let TextPart::Text(text) = &mut last.part {
                text.value = text.value.trim_end().to_owned();
            }
        }
    }

    atoms
        .into_iter()
        .filter_map(|atom| match atom.part {
            TextPart::Text(text) if atom.trim_edges && text.value.is_empty() => None,
            part => Some(part),
        })
        .collect()
}

fn placeholder<'tokens, I>() -> impl Parser<'tokens, I, Placeholder, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    expression()
        .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
        .map_with(|expression, extra| Placeholder {
            expression,
            span: extra.span(),
        })
}

fn expression<'tokens, I>() -> impl Parser<'tokens, I, Expression, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    recursive(|expr| {
        name()
            .separated_by(just(TokenKind::Dot))
            .at_least(1)
            .collect::<Vec<_>>()
            .then(
                expr.separated_by(just(TokenKind::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
                    .or_not(),
            )
            .then(annotation().repeated().collect::<Vec<_>>())
            .map_with(|((path, arguments), annotations), extra| Expression {
                path,
                arguments: arguments.unwrap_or_default(),
                annotations,
                span: extra.span(),
            })
    })
}

enum MessageOrGroup {
    Message {
        value: TextPattern,
        span: Span,
    },
    Group {
        messages: Vec<MessageImplementation>,
        span: Span,
    },
}

trait LocaleDeclarationDocs {
    fn set_docs(&mut self, docs: Vec<crate::DocComment>);
}

impl LocaleDeclarationDocs for LocaleDeclaration {
    fn set_docs(&mut self, docs: Vec<crate::DocComment>) {
        match self {
            LocaleDeclaration::Enum(declaration) => declaration.docs = docs,
            LocaleDeclaration::Variable(declaration) => declaration.docs = docs,
            LocaleDeclaration::Form(declaration) => declaration.docs = docs,
            LocaleDeclaration::Function(declaration) => declaration.docs = docs,
            LocaleDeclaration::Message(declaration) => declaration.docs = docs,
            LocaleDeclaration::Group(declaration) => declaration.docs = docs,
            LocaleDeclaration::Override(declaration) => declaration.set_docs(docs),
        }
    }
}
