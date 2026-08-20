use chumsky::{input::IterInput, input::ValueInput, prelude::*};

mod locale_parser;
#[cfg(test)]
mod locale_tests;
mod validate;

use validate::{validate_locale, validate_schema};

use crate::{
    lex_in, lex_schema_in, lex_schema_with_recovery_in, lex_with_recovery_in, Annotation,
    AnnotationArgument, DocComment, EnumDeclaration, LocaleFile, MessageGroup, MessageSignature,
    Name, Parameter, SchemaDeclaration, SchemaFile, SourceId, Span, StringLiteral, Token,
    TokenKind, TypeAliasDeclaration,
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

/// A validated syntax tree and the lossless token stream produced by the same lexer pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSource<T> {
    pub ast: T,
    pub tokens: Vec<Token>,
}

pub fn validate_locale_ast(file: &LocaleFile) -> Vec<ParseError> {
    validate_locale(file)
}

pub fn validate_schema_ast(file: &SchemaFile) -> Vec<ParseError> {
    validate_schema(file)
}

pub fn parse_locale(source: &str) -> Result<LocaleFile, Vec<ParseError>> {
    parse_locale_with_tokens(source).map(|parsed| parsed.ast)
}

pub fn parse_locale_in(source: &str, source_id: SourceId) -> Result<LocaleFile, Vec<ParseError>> {
    parse_locale_with_tokens_in(source, source_id).map(|parsed| parsed.ast)
}

/// Parses and validates locale source while retaining trivia from the same lexer pass.
pub fn parse_locale_with_tokens(source: &str) -> Result<ParsedSource<LocaleFile>, Vec<ParseError>> {
    parse_locale_with_tokens_in(source, SourceId::default())
}

/// Parses and validates locale source with an explicit source ID while retaining its tokens.
pub fn parse_locale_with_tokens_in(
    source: &str,
    source_id: SourceId,
) -> Result<ParsedSource<LocaleFile>, Vec<ParseError>> {
    let tokens = lex_in(source, source_id).map_err(|error| {
        vec![ParseError {
            message: error.message,
            span: error.span,
        }]
    })?;
    let syntax_tokens = strip_trivia(&tokens);
    let eof = Span::in_source(source_id, source.len(), source.len());
    let (ast, errors) = locale_parser::locale_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    let mut errors: Vec<_> = errors.into_iter().map(parse_error_from_rich).collect();
    if let Some(ast) = ast {
        errors.extend(validate_locale(&ast));
        if errors.is_empty() {
            Ok(ParsedSource { ast, tokens })
        } else {
            Err(errors)
        }
    } else {
        if errors.is_empty() {
            errors.push(ParseError {
                message: "parser produced no syntax tree".to_owned(),
                span: eof,
            });
        }
        Err(errors)
    }
}

pub fn parse_locale_with_recovery(source: &str) -> ParseOutput<LocaleFile> {
    parse_locale_with_recovery_in(source, SourceId::default())
}

pub fn parse_locale_with_recovery_in(source: &str, source_id: SourceId) -> ParseOutput<LocaleFile> {
    let lexed = lex_with_recovery_in(source, source_id);
    let mut errors: Vec<_> = lexed
        .errors
        .into_iter()
        .map(|error| ParseError {
            message: error.message,
            span: error.span,
        })
        .collect();
    let syntax_tokens = strip_trivia(&lexed.tokens);
    let eof = Span::in_source(source_id, source.len(), source.len());
    let (ast, parse_errors) = locale_parser::locale_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    errors.extend(parse_errors.into_iter().map(parse_error_from_rich));
    ParseOutput { ast, errors }
}

pub fn parse_schema(source: &str) -> Result<SchemaFile, Vec<ParseError>> {
    parse_schema_with_tokens(source).map(|parsed| parsed.ast)
}

pub fn parse_schema_in(source: &str, source_id: SourceId) -> Result<SchemaFile, Vec<ParseError>> {
    parse_schema_with_tokens_in(source, source_id).map(|parsed| parsed.ast)
}

/// Parses and validates schema source while retaining trivia from the same lexer pass.
pub fn parse_schema_with_tokens(source: &str) -> Result<ParsedSource<SchemaFile>, Vec<ParseError>> {
    parse_schema_with_tokens_in(source, SourceId::default())
}

/// Parses and validates schema source with an explicit source ID while retaining its tokens.
pub fn parse_schema_with_tokens_in(
    source: &str,
    source_id: SourceId,
) -> Result<ParsedSource<SchemaFile>, Vec<ParseError>> {
    let tokens = lex_schema_in(source, source_id).map_err(|error| {
        vec![ParseError {
            message: error.message,
            span: error.span,
        }]
    })?;
    let syntax_tokens = strip_trivia(&tokens);
    let eof = Span::in_source(source_id, source.len(), source.len());
    let (ast, errors) = schema_parser()
        .parse(IterInput::new(syntax_tokens.into_iter(), eof))
        .into_output_errors();

    let mut errors: Vec<_> = errors.into_iter().map(parse_error_from_rich).collect();
    if let Some(ast) = ast {
        errors.extend(validate_schema(&ast));
        if errors.is_empty() {
            Ok(ParsedSource { ast, tokens })
        } else {
            Err(errors)
        }
    } else {
        if errors.is_empty() {
            errors.push(ParseError {
                message: "parser produced no syntax tree".to_owned(),
                span: eof,
            });
        }
        Err(errors)
    }
}

pub fn parse_schema_with_recovery(source: &str) -> ParseOutput<SchemaFile> {
    parse_schema_with_recovery_in(source, SourceId::default())
}

pub fn parse_schema_with_recovery_in(source: &str, source_id: SourceId) -> ParseOutput<SchemaFile> {
    let lexed = lex_schema_with_recovery_in(source, source_id);
    let mut errors: Vec<_> = lexed
        .errors
        .into_iter()
        .map(|error| ParseError {
            message: error.message,
            span: error.span,
        })
        .collect();
    let syntax_tokens = strip_trivia(&lexed.tokens);
    let eof = Span::in_source(source_id, source.len(), source.len());
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

fn strip_trivia(tokens: &[Token]) -> Vec<(TokenKind, Span)> {
    let mut output = Vec::new();
    let mut pending_doc_span = None;
    let mut line_breaks_after_doc = 0usize;
    let mut comment_after_doc = false;

    for token in tokens {
        match &token.kind {
            TokenKind::Whitespace => {}
            TokenKind::Newline => {
                if pending_doc_span.is_some() {
                    line_breaks_after_doc += 1;
                }
            }
            TokenKind::Comment(_) => {
                if pending_doc_span.is_some() {
                    comment_after_doc = true;
                }
            }
            TokenKind::DocComment(_) => {
                if let Some(span) = pending_doc_span {
                    if line_breaks_after_doc > 1 || comment_after_doc {
                        output.push((TokenKind::Error("detached doc comment".to_owned()), span));
                    }
                }
                pending_doc_span = Some(token.span);
                line_breaks_after_doc = 0;
                comment_after_doc = false;
                output.push((token.kind.clone(), token.span));
            }
            _ => {
                if let Some(span) = pending_doc_span.take() {
                    if line_breaks_after_doc > 1 || comment_after_doc {
                        output.push((TokenKind::Error("detached doc comment".to_owned()), span));
                    }
                }
                line_breaks_after_doc = 0;
                comment_after_doc = false;
                output.push((token.kind.clone(), token.span));
            }
        }
    }

    output
}

fn schema_parser<'tokens, I>() -> impl Parser<'tokens, I, SchemaFile, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    let declaration = declaration_parser();

    declaration
        .recover_with(skip_then_retry_until(any().ignored(), end()))
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
    schema_group_member().map(|member| match member {
        SchemaGroupMember::Message(message) => SchemaDeclaration::Message(message),
        SchemaGroupMember::Group(group) => SchemaDeclaration::Group(group),
    })
}

fn schema_group_member<'tokens, I>(
) -> impl Parser<'tokens, I, SchemaGroupMember, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    recursive(|member| {
        let body = name()
            .then(choice((
                parameters().map_with(|parameters, extra| GroupOrMessage::Message {
                    parameters,
                    span: extra.span(),
                }),
                member
                    .clone()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
                    .map_with(|members, extra| GroupOrMessage::Group {
                        members,
                        span: extra.span(),
                    }),
                empty().map_with(|_, extra| GroupOrMessage::Message {
                    parameters: Vec::new(),
                    span: extra.span(),
                }),
            )))
            .map(|(name, body)| match body {
                GroupOrMessage::Message { parameters, span } => {
                    SchemaGroupMember::Message(MessageSignature {
                        docs: Vec::new(),
                        span: name.span.union(span),
                        name,
                        parameters,
                    })
                }
                GroupOrMessage::Group { members, span } => {
                    let (messages, groups) = split_schema_group_members(members);
                    SchemaGroupMember::Group(MessageGroup {
                        docs: Vec::new(),
                        span: name.span.union(span),
                        name,
                        messages,
                        groups,
                    })
                }
            });

        doc_comment()
            .repeated()
            .collect::<Vec<_>>()
            .then(body)
            .map(|(docs, mut member)| {
                member.set_docs(docs);
                member
            })
    })
}

fn split_schema_group_members(
    members: Vec<SchemaGroupMember>,
) -> (Vec<MessageSignature>, Vec<MessageGroup>) {
    let mut messages = Vec::new();
    let mut groups = Vec::new();
    for member in members {
        match member {
            SchemaGroupMember::Message(message) => messages.push(message),
            SchemaGroupMember::Group(group) => groups.push(group),
        }
    }
    (messages, groups)
}

impl SchemaGroupMember {
    fn set_docs(&mut self, docs: Vec<DocComment>) {
        match self {
            Self::Message(message) => message.docs = docs,
            Self::Group(group) => group.docs = docs,
        }
    }
}

enum SchemaGroupMember {
    Message(MessageSignature),
    Group(MessageGroup),
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
        .at_least(1)
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
                .at_least(1)
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))
                .or_not(),
        )
        .map_with(|(name, arguments), extra| Annotation {
            kind: crate::FormatterKind::from_name(&name.value),
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
        members: Vec<SchemaGroupMember>,
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
