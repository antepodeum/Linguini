use chumsky::{input::IterInput, input::ValueInput, prelude::*};

mod locale_parser;
#[cfg(test)]
mod locale_tests;

use crate::{
    lex, lex_schema, lex_schema_with_recovery, lex_with_recovery, Annotation, AnnotationArgument,
    DocComment, EnumDeclaration, LocaleFile, MessageGroup, MessageSignature, Name, Parameter,
    SchemaDeclaration, SchemaFile, Span, StringLiteral, TokenKind, TypeAliasDeclaration,
};

type Extra<'tokens> = extra::Err<Rich<'tokens, TokenKind, Span>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput<T> {
    pub ast: Option<T>,
    pub errors: Vec<ParseError>,
}

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
    let (ast, errors) = locale_parser::locale_parser()
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
    let (ast, parse_errors) = locale_parser::locale_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    errors.extend(parse_errors.into_iter().map(parse_error_from_rich));

    ParseOutput { ast, errors }
}

pub fn parse_schema(source: &str) -> Result<SchemaFile, Vec<ParseError>> {
    let tokens = lex_schema(source).map_err(|error| {
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
    let (ast, errors) = schema_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    if errors.is_empty() {
        Ok(ast.expect("parser produced an AST without errors"))
    } else {
        Err(errors.into_iter().map(parse_error_from_rich).collect())
    }
}

pub fn parse_schema_with_recovery(source: &str) -> ParseOutput<SchemaFile> {
    let lexed = lex_schema_with_recovery(source);
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
    let (ast, parse_errors) = schema_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    errors.extend(parse_errors.into_iter().map(parse_error_from_rich));

    ParseOutput { ast, errors }
}

fn parse_error_from_rich(error: Rich<'_, TokenKind, Span>) -> ParseError {
    let found = error
        .found()
        .map(ToString::to_string)
        .unwrap_or_else(|| "end of input".to_owned());
    let expected = join_expected(
        error
            .expected()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    );

    ParseError {
        message: format!("found {found} expected {expected}"),
        span: *error.span(),
    }
}

fn join_expected(mut expected: Vec<String>) -> String {
    expected.sort();
    expected.dedup();

    let Some(last) = expected.pop() else {
        return "a valid syntax element".to_owned();
    };

    if expected.is_empty() {
        last
    } else {
        format!("{}, or {last}", expected.join(", "))
    }
}

fn strip_trivia(tokens: Vec<crate::Token>) -> Vec<crate::Token> {
    tokens
        .into_iter()
        .filter(|token| {
            !matches!(
                token.kind,
                TokenKind::Whitespace | TokenKind::Newline | TokenKind::Comment(_)
            )
        })
        .collect()
}

fn schema_parser<'tokens, I>() -> impl Parser<'tokens, I, SchemaFile, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    let declaration = declaration_parser();

    declaration
        .repeated()
        .collect::<Vec<_>>()
        .then_ignore(end())
        .map_with(|declarations, extra| SchemaFile {
            declarations,
            span: extra.span(),
        })
}

fn declaration_parser<'tokens, I>(
) -> impl Parser<'tokens, I, SchemaDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    let docs = doc_comment().repeated().collect::<Vec<_>>();

    docs.then(choice((
        enum_declaration().map(SchemaDeclaration::Enum),
        type_alias_declaration().map(SchemaDeclaration::TypeAlias),
        group_or_message_declaration(),
    )))
    .map(|(docs, mut declaration)| {
        declaration.set_docs(docs);
        declaration
    })
}

fn enum_declaration<'tokens, I>() -> impl Parser<'tokens, I, EnumDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    keyword("enum")
        .ignore_then(name())
        .then(
            name()
                .then_ignore(just(TokenKind::Comma).or_not())
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace)),
        )
        .map_with(|(name, variants), extra| EnumDeclaration {
            docs: Vec::new(),
            name,
            variants,
            span: extra.span(),
        })
}

fn type_alias_declaration<'tokens, I>(
) -> impl Parser<'tokens, I, TypeAliasDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    keyword("type")
        .ignore_then(name())
        .then_ignore(just(TokenKind::Equals))
        .then(name())
        .then(annotation().repeated().collect::<Vec<_>>())
        .map_with(
            |((name, target), annotations), extra| TypeAliasDeclaration {
                docs: Vec::new(),
                name,
                target,
                annotations,
                span: extra.span(),
            },
        )
}

fn group_or_message_declaration<'tokens, I>(
) -> impl Parser<'tokens, I, SchemaDeclaration, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    name()
        .then(choice((
            parameters().map_with(|parameters, extra| GroupOrMessage::Message {
                parameters,
                span: extra.span(),
            }),
            message_signature_body()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
                .map_with(|messages, extra| GroupOrMessage::Group {
                    messages,
                    span: extra.span(),
                }),
        )))
        .map(|(name, body)| match body {
            GroupOrMessage::Message { parameters, span } => {
                let declaration_span = name.span.union(span);
                SchemaDeclaration::Message(MessageSignature {
                    docs: Vec::new(),
                    name,
                    parameters,
                    span: declaration_span,
                })
            }
            GroupOrMessage::Group { messages, span } => SchemaDeclaration::Group(MessageGroup {
                docs: Vec::new(),
                span: name.span.union(span),
                name,
                messages,
            }),
        })
}

fn message_signature_body<'tokens, I>(
) -> impl Parser<'tokens, I, MessageSignature, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .then(name())
        .then(parameters().or_not())
        .map_with(|((docs, name), parameters), extra| MessageSignature {
            docs,
            name,
            parameters: parameters.unwrap_or_default(),
            span: extra.span(),
        })
}

fn parameters<'tokens, I>() -> impl Parser<'tokens, I, Vec<Parameter>, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    name()
        .then_ignore(just(TokenKind::Colon))
        .then(name())
        .map_with(|(name, ty), extra| Parameter {
            name,
            ty,
            span: extra.span(),
        })
        .separated_by(just(TokenKind::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
}

fn annotation<'tokens, I>() -> impl Parser<'tokens, I, Annotation, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    just(TokenKind::At)
        .ignore_then(name())
        .then(
            annotation_argument()
                .separated_by(just(TokenKind::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
                .or_not(),
        )
        .map_with(|(name, arguments), extra| Annotation {
            name,
            arguments: arguments.unwrap_or_default(),
            span: extra.span(),
        })
}

fn annotation_argument<'tokens, I>(
) -> impl Parser<'tokens, I, AnnotationArgument, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    name()
        .then_ignore(just(TokenKind::Equals))
        .then(string_literal())
        .map_with(|(name, value), extra| AnnotationArgument {
            name,
            value,
            span: extra.span(),
        })
}

fn doc_comment<'tokens, I>() -> impl Parser<'tokens, I, DocComment, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    select! { TokenKind::DocComment(text) => text }.map_with(|text, extra| DocComment {
        text,
        span: extra.span(),
    })
}

fn name<'tokens, I>() -> impl Parser<'tokens, I, Name, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    select! { TokenKind::Ident(value) => value }.map_with(|value, extra| Name {
        value,
        span: extra.span(),
    })
}

fn string_literal<'tokens, I>() -> impl Parser<'tokens, I, StringLiteral, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    select! { TokenKind::String(value) => value }.map_with(|value, extra| StringLiteral {
        value,
        span: extra.span(),
    })
}

fn keyword<'tokens, I>(word: &'static str) -> impl Parser<'tokens, I, (), Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    just(TokenKind::Ident(word.to_string())).ignored()
}

enum GroupOrMessage {
    Message {
        parameters: Vec<Parameter>,
        span: Span,
    },
    Group {
        messages: Vec<MessageSignature>,
        span: Span,
    },
}

trait SchemaDeclarationDocs {
    fn set_docs(&mut self, docs: Vec<DocComment>);
}

impl SchemaDeclarationDocs for SchemaDeclaration {
    fn set_docs(&mut self, docs: Vec<DocComment>) {
        match self {
            SchemaDeclaration::Enum(declaration) => declaration.docs = docs,
            SchemaDeclaration::TypeAlias(declaration) => declaration.docs = docs,
            SchemaDeclaration::Message(declaration) => declaration.docs = docs,
            SchemaDeclaration::Group(declaration) => declaration.docs = docs,
        }
    }
}
