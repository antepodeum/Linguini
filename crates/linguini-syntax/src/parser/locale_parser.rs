use chumsky::{input::IterInput, input::ValueInput, prelude::*};

use crate::{
    lex, lex_with_recovery, BranchPattern, Expression, FormAttribute, FormDeclaration, FormEntry,
    FormVariant, FunctionBranch, FunctionDeclaration, LocaleDeclaration, LocaleFile, LocaleValue,
    MapBranch, MessageImplementation, MessageImplementationGroup, Name, Placeholder, RawText, Span,
    TextPart, TextPattern, TokenKind,
};

use super::{
    annotation, doc_comment, enum_declaration, keyword, name, parse_error_from_rich, strip_trivia,
    Extra, ParseError, ParseOutput,
};

pub fn parse_locale(source: &str) -> Result<LocaleFile, Vec<ParseError>> {
    let tokens = lex(source).map_err(|error| {
        vec![ParseError {
            message: error.message,
            span: error.span,
        }]
    })?;
    let syntax_tokens: Vec<_> = strip_trivia(tokens)
        .into_iter()
        .map(|token| (token.kind, token.span))
        .collect();
    let eof = Span::new(source.len(), source.len());
    let (ast, errors) = locale_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    if errors.is_empty() {
        Ok(ast.expect("parser produced an AST without errors"))
    } else {
        Err(errors.into_iter().map(parse_error_from_rich).collect())
    }
}

pub fn parse_locale_with_recovery(source: &str) -> ParseOutput<LocaleFile> {
    let lexed = lex_with_recovery(source);
    let mut errors: Vec<_> = lexed
        .errors
        .into_iter()
        .map(|error| ParseError {
            message: error.message,
            span: error.span,
        })
        .collect();
    let syntax_tokens: Vec<_> = strip_trivia(lexed.tokens)
        .into_iter()
        .filter(|token| !matches!(token.kind, TokenKind::Error(_)))
        .map(|token| (token.kind, token.span))
        .collect();
    let eof = Span::new(source.len(), source.len());
    let (ast, parse_errors) = locale_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    errors.extend(parse_errors.into_iter().map(parse_error_from_rich));

    ParseOutput { ast, errors }
}

fn locale_parser<'tokens, I>() -> impl Parser<'tokens, I, LocaleFile, Extra<'tokens>> + Clone
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
            enum_declaration().map(LocaleDeclaration::Enum),
            form_declaration().map(LocaleDeclaration::Form),
            function_declaration().map(LocaleDeclaration::Function),
            group_or_message(),
        )))
        .map(|(docs, mut declaration)| {
            declaration.set_docs(docs);
            declaration
        })
}

fn form_declaration<'tokens, I>() -> impl Parser<'tokens, I, FormDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    keyword("form")
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
        .then(just(TokenKind::Colon).ignore_then(name()).or_not())
        .then(
            form_entry_parser()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace)),
        )
        .map_with(|((name, selector), entries), extra| FormVariant {
            name,
            selector,
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
        let attribute = name()
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
        .then(
            name()
                .separated_by(just(TokenKind::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen)),
        )
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

fn function_branch<'tokens, I>() -> impl Parser<'tokens, I, FunctionBranch, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    choice((
        keyword("else").map_with(|_, extra| {
            BranchPattern::Else(Name {
                value: "else".to_string(),
                span: extra.span(),
            })
        }),
        name()
            .separated_by(just(TokenKind::Comma))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(BranchPattern::Names),
    ))
    .then_ignore(just(TokenKind::Arrow))
    .then(text_pattern())
    .map_with(|(pattern, value), extra| FunctionBranch {
        pattern,
        value,
        span: extra.span(),
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
        select! { TokenKind::RawText(value) => value }.map_with(|value, extra| {
            TextPart::Text(RawText {
                value,
                span: extra.span(),
            })
        }),
        placeholder().map(TextPart::Placeholder),
    ))
    .repeated()
    .at_least(1)
    .collect::<Vec<_>>()
    .map_with(|parts, extra| TextPattern {
        parts,
        span: extra.span(),
    })
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
            LocaleDeclaration::Form(declaration) => declaration.docs = docs,
            LocaleDeclaration::Function(declaration) => declaration.docs = docs,
            LocaleDeclaration::Message(declaration) => declaration.docs = docs,
            LocaleDeclaration::Group(declaration) => declaration.docs = docs,
        }
    }
}
