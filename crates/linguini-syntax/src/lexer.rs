use chumsky::prelude::*;

use crate::{Span, Token, TokenKind};

type Extra<'src> = extra::Err<Simple<'src, char>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Code,
    RawText,
    MultilineText,
    Placeholder(ResumeMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumeMode {
    RawText,
    MultilineText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Lexeme {
    token: Token,
    next_mode: Option<Mode>,
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(source).lex()
}

struct Lexer<'src> {
    source: &'src str,
    offset: usize,
    mode: Mode,
    tokens: Vec<Token>,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            offset: 0,
            mode: Mode::Code,
            tokens: Vec::new(),
        }
    }

    fn lex(mut self) -> Result<Vec<Token>, LexError> {
        while self.offset < self.source.len() {
            let lexeme = match self.mode {
                Mode::Code => parse_at(code_token(), self.source, self.offset)?,
                Mode::RawText => parse_at(raw_text_token(false), self.source, self.offset)?,
                Mode::MultilineText => parse_at(raw_text_token(true), self.source, self.offset)?,
                Mode::Placeholder(resume) => {
                    parse_at(placeholder_token(resume), self.source, self.offset)?
                }
            };

            self.offset = lexeme.token.span.end;
            if let Some(mode) = lexeme.next_mode {
                self.mode = mode;
            }
            self.tokens.push(lexeme.token);
        }

        match self.mode {
            Mode::Code | Mode::RawText => Ok(self.tokens),
            Mode::MultilineText => Err(LexError::new(
                "unterminated multiline text",
                Span::new(self.offset, self.offset),
            )),
            Mode::Placeholder(_) => Err(LexError::new(
                "unterminated placeholder",
                Span::new(self.offset, self.offset),
            )),
        }
    }
}

fn parse_at<'src, P>(parser: P, source: &'src str, offset: usize) -> Result<Lexeme, LexError>
where
    P: Parser<'src, &'src str, Lexeme, Extra<'src>>,
{
    let rest = &source[offset..];
    let result = parser.then_ignore(any().repeated()).parse(rest);
    match result.into_result() {
        Ok(lexeme) => Ok(local_to_source_lexeme(lexeme, rest, offset)),
        Err(errors) => {
            let span = errors
                .first()
                .map(|error| {
                    local_to_source_span(chumsky_span_to_span(*error.span()), rest, offset)
                })
                .unwrap_or_else(|| Span::new(offset, offset));
            Err(LexError::new("failed to lex token", span))
        }
    }
}

fn local_to_source_lexeme(mut lexeme: Lexeme, source: &str, offset: usize) -> Lexeme {
    lexeme.token.span = local_to_source_span(lexeme.token.span, source, offset);
    lexeme
}

fn local_to_source_span(span: Span, source: &str, offset: usize) -> Span {
    let _ = source;
    Span::new(offset + span.start, offset + span.end)
}

fn code_token<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    choice((
        newline(),
        doc_comment(),
        comment(),
        literal_token("\"\"\"", TokenKind::TripleQuote, Some(Mode::MultilineText)),
        literal_token("=>", TokenKind::Arrow, Some(Mode::RawText)),
        horizontal_whitespace(),
        string_literal(),
        ident_like(),
        punctuation(),
    ))
}

fn placeholder_token<'src>(
    resume: ResumeMode,
) -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    let end_mode = match resume {
        ResumeMode::RawText => Mode::RawText,
        ResumeMode::MultilineText => Mode::MultilineText,
    };

    choice((
        newline(),
        literal_token("}", TokenKind::RBrace, Some(end_mode)),
        code_token(),
    ))
}

fn raw_text_token<'src>(multiline: bool) -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    let newline_mode = if multiline {
        Mode::MultilineText
    } else {
        Mode::Code
    };
    let placeholder_mode = if multiline {
        Mode::Placeholder(ResumeMode::MultilineText)
    } else {
        Mode::Placeholder(ResumeMode::RawText)
    };

    choice((
        literal_token(
            "\"\"\"",
            TokenKind::TripleQuote,
            Some(if multiline {
                Mode::RawText
            } else {
                Mode::MultilineText
            }),
        ),
        newline().map(move |mut lexeme| {
            lexeme.next_mode = Some(newline_mode);
            lexeme
        }),
        literal_token("{", TokenKind::LBrace, Some(placeholder_mode)),
        raw_text_segment(multiline),
    ))
}

fn literal_token<'src>(
    literal: &'static str,
    kind: TokenKind,
    next_mode: Option<Mode>,
) -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    just(literal).map(move |_| Lexeme {
        token: Token::new(kind.clone(), Span::new(0, literal.len())),
        next_mode,
    })
}

fn ident_like<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    any()
        .filter(|ch: &char| *ch == '_' || ch.is_alphabetic())
        .then(
            any()
                .filter(|ch: &char| *ch == '_' || *ch == '-' || ch.is_alphanumeric())
                .repeated(),
        )
        .to_slice()
        .map(|text: &str| {
            let kind = if is_locale_tag(text) {
                TokenKind::LocaleTag(text.to_string())
            } else {
                TokenKind::Ident(text.to_string())
            };
            Lexeme {
                token: Token::new(kind, Span::new(0, text.len())),
                next_mode: None,
            }
        })
}

fn punctuation<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    choice((
        just('{').to((TokenKind::LBrace, None)),
        just('}').to((TokenKind::RBrace, None)),
        just('(').to((TokenKind::LParen, None)),
        just(')').to((TokenKind::RParen, None)),
        just(',').to((TokenKind::Comma, None)),
        just(':').to((TokenKind::Colon, None)),
        just('=').to((TokenKind::Equals, Some(Mode::RawText))),
        just('.').to((TokenKind::Dot, None)),
        just('@').to((TokenKind::At, None)),
    ))
    .map(|(kind, next_mode)| Lexeme {
        token: Token::new(kind, Span::new(0, 1)),
        next_mode,
    })
}

fn newline<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    choice((just("\r\n"), just("\n"), just("\r")))
        .to_slice()
        .map(|text: &str| Lexeme {
            token: Token::new(TokenKind::Newline, Span::new(0, text.len())),
            next_mode: None,
        })
}

fn horizontal_whitespace<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    any()
        .filter(|ch: &char| ch.is_whitespace() && *ch != '\n' && *ch != '\r')
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|text: &str| Lexeme {
            token: Token::new(TokenKind::Whitespace, Span::new(0, text.len())),
            next_mode: None,
        })
}

fn comment<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    just("//")
        .ignore_then(
            any()
                .filter(|ch: &char| *ch != '\n' && *ch != '\r')
                .repeated()
                .to_slice(),
        )
        .map(|text: &str| Lexeme {
            token: Token::new(
                TokenKind::Comment(text.to_string()),
                Span::new(0, 2 + text.len()),
            ),
            next_mode: None,
        })
}

fn doc_comment<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    just("///")
        .ignore_then(
            any()
                .filter(|ch: &char| *ch != '\n' && *ch != '\r')
                .repeated()
                .to_slice(),
        )
        .map(|text: &str| Lexeme {
            token: Token::new(
                TokenKind::DocComment(text.to_string()),
                Span::new(0, 3 + text.len()),
            ),
            next_mode: None,
        })
}

fn string_literal<'src>() -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    just('"')
        .ignore_then(
            none_of("\"\\\n\r")
                .or(just('\\').ignore_then(any()))
                .repeated()
                .collect::<String>(),
        )
        .then_ignore(just('"'))
        .to_slice()
        .map(|raw: &str| Lexeme {
            token: Token::new(
                TokenKind::String(raw[1..raw.len() - 1].to_string()),
                Span::new(0, raw.len()),
            ),
            next_mode: None,
        })
}

fn raw_text_segment<'src>(multiline: bool) -> impl Parser<'src, &'src str, Lexeme, Extra<'src>> {
    any()
        .filter(move |ch: &char| {
            let _ = multiline;
            *ch != '{' && *ch != '\n' && *ch != '\r' && *ch != '"'
        })
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|text| {
            let len = text.len();
            Lexeme {
                token: Token::new(TokenKind::RawText(text), Span::new(0, len)),
                next_mode: None,
            }
        })
}

fn chumsky_span_to_span(span: SimpleSpan) -> Span {
    Span::new(span.start, span.end)
}

fn is_locale_tag(text: &str) -> bool {
    text.contains('-')
        && text.split('-').all(|part| !part.is_empty())
        && text
            .chars()
            .all(|ch| ch == '-' || ch.is_ascii_alphanumeric())
}
