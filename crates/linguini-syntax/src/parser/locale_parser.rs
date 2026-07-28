use chumsky::{input::ValueInput, prelude::*};

use crate::{
    Expression, ExpressionKind, FormAttribute, FormDeclaration, FormEntry, FormVariant,
    FunctionBranch, FunctionBranchValue, FunctionDeclaration, FunctionKind, FunctionParameter,
    LocaleDeclaration, LocaleFile, LocaleValue, MapBranch, MessageImplementation,
    MessageImplementationGroup, Placeholder, RawText, Span, TextBlockMode, TextPart, TextPattern,
    TokenKind, VariableDeclaration,
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
            keyword_span("override").then(declaration_body()).map(
                |(keyword_span, mut declaration)| {
                    declaration.extend_span(keyword_span);
                    LocaleDeclaration::Override(Box::new(declaration))
                },
            ),
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
                kind: FunctionKind::Function,
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
                kind: FunctionKind::Form,
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
    locale_group_member().map(|member| match member {
        LocaleGroupMember::Message(message) => LocaleDeclaration::Message(message),
        LocaleGroupMember::Group(group) => LocaleDeclaration::Group(group),
    })
}

fn locale_group_member<'tokens, I>(
) -> impl Parser<'tokens, I, LocaleGroupMember, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    recursive(|member| {
        let body = name()
            .then(choice((
                just(TokenKind::Equals)
                    .ignore_then(text_pattern())
                    .map_with(|value, extra| MessageOrGroup::Message {
                        value,
                        span: extra.span(),
                    }),
                member
                    .clone()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
                    .map_with(|members, extra| MessageOrGroup::Group {
                        members,
                        span: extra.span(),
                    }),
            )))
            .map(|(name, item)| match item {
                MessageOrGroup::Message { value, span } => {
                    LocaleGroupMember::Message(MessageImplementation {
                        docs: Vec::new(),
                        span: name.span.union(span),
                        name,
                        value,
                    })
                }
                MessageOrGroup::Group { members, span } => {
                    let (messages, groups) = split_locale_group_members(members);
                    LocaleGroupMember::Group(MessageImplementationGroup {
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

fn split_locale_group_members(
    members: Vec<LocaleGroupMember>,
) -> (Vec<MessageImplementation>, Vec<MessageImplementationGroup>) {
    let mut messages = Vec::new();
    let mut groups = Vec::new();
    for member in members {
        match member {
            LocaleGroupMember::Message(message) => messages.push(message),
            LocaleGroupMember::Group(group) => groups.push(group),
        }
    }
    (messages, groups)
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
        raw_multiline_pattern(),
        dedented_multiline_pattern(),
        inline_pattern(),
    ))
}

fn inline_pattern<'tokens, I>() -> impl Parser<'tokens, I, TextPattern, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    text_atom()
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .map_with(|atoms, extra| TextPattern {
            parts: trim_text_atoms(atoms),
            mode: TextBlockMode::Inline,
            span: extra.span(),
        })
}

fn text_atom<'tokens, I>() -> impl Parser<'tokens, I, TextAtom, Extra<'tokens>> + Clone
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
}

fn dedented_multiline_pattern<'tokens, I>(
) -> impl Parser<'tokens, I, TextPattern, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    block_part()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just(TokenKind::TripleQuote), just(TokenKind::TripleQuote))
        .map_with(|parts, extra| TextPattern {
            parts: dedent_block(parts),
            mode: TextBlockMode::Dedented,
            span: extra.span(),
        })
}

fn raw_multiline_pattern<'tokens, I>(
) -> impl Parser<'tokens, I, TextPattern, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    block_part()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(
            just(TokenKind::RawTripleQuote),
            just(TokenKind::RawTripleQuote),
        )
        .map_with(|parts, extra| TextPattern {
            parts,
            mode: TextBlockMode::Raw,
            span: extra.span(),
        })
}

fn block_part<'tokens, I>() -> impl Parser<'tokens, I, TextPart, Extra<'tokens>> + Clone
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

#[derive(Debug)]
enum BlockUnit {
    Character { value: char, span: Span },
    Placeholder(Placeholder),
}

fn dedent_block(parts: Vec<TextPart>) -> Vec<TextPart> {
    let mut units = block_units(parts);
    remove_structural_edge_lines(&mut units);
    let prefix = common_leading_whitespace(&units);
    if !prefix.is_empty() {
        units = remove_line_prefix(units, &prefix);
    }
    block_parts(units)
}

fn block_units(parts: Vec<TextPart>) -> Vec<BlockUnit> {
    let mut units = Vec::new();
    for part in parts {
        match part {
            TextPart::Placeholder(placeholder) => {
                units.push(BlockUnit::Placeholder(placeholder));
            }
            TextPart::Text(text) => {
                let source_width_matches =
                    text.span.end.saturating_sub(text.span.start) == text.value.len();
                let mut cursor = 0;
                while cursor < text.value.len() {
                    let rest = &text.value[cursor..];
                    if rest.starts_with("\r\n") {
                        units.push(BlockUnit::Character {
                            value: '\n',
                            span: text_subspan(&text, cursor, cursor + 2, source_width_matches),
                        });
                        cursor += 2;
                        continue;
                    }
                    let Some(character) = rest.chars().next() else {
                        break;
                    };
                    let width = character.len_utf8();
                    units.push(BlockUnit::Character {
                        value: if character == '\r' { '\n' } else { character },
                        span: text_subspan(&text, cursor, cursor + width, source_width_matches),
                    });
                    cursor += width;
                }
            }
        }
    }
    units
}

fn text_subspan(text: &RawText, start: usize, end: usize, exact: bool) -> Span {
    if exact {
        Span::in_source(
            text.span.source,
            text.span.start + start,
            text.span.start + end,
        )
    } else {
        text.span
    }
}

fn remove_structural_edge_lines(units: &mut Vec<BlockUnit>) {
    if let Some(first_newline) = units.iter().position(is_newline) {
        if units[..first_newline].iter().all(is_whitespace) {
            units.drain(..=first_newline);
        }
    }

    if let Some(last_newline) = units.iter().rposition(is_newline) {
        if units[last_newline + 1..].iter().all(is_whitespace) {
            units.drain(last_newline..);
        }
    }
}

fn common_leading_whitespace(units: &[BlockUnit]) -> Vec<char> {
    let mut common: Option<Vec<char>> = None;
    let mut line_start = 0;
    for line_end in units
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_newline(unit).then_some(index))
        .chain(std::iter::once(units.len()))
    {
        let line = &units[line_start..line_end];
        if line.iter().any(|unit| !is_whitespace(unit)) {
            let prefix: Vec<_> = line
                .iter()
                .take_while(|unit| is_horizontal_whitespace(unit))
                .filter_map(unit_character)
                .collect();
            match &mut common {
                Some(common) => {
                    let shared = common
                        .iter()
                        .zip(&prefix)
                        .take_while(|(left, right)| left == right)
                        .count();
                    common.truncate(shared);
                }
                None => common = Some(prefix),
            }
        }
        line_start = line_end.saturating_add(1);
    }
    common.unwrap_or_default()
}

fn remove_line_prefix(units: Vec<BlockUnit>, prefix: &[char]) -> Vec<BlockUnit> {
    let mut output = Vec::with_capacity(units.len());
    let mut prefix_index = 0;
    for unit in units {
        if is_newline(&unit) {
            output.push(unit);
            prefix_index = 0;
            continue;
        }
        if prefix_index < prefix.len()
            && unit_character(&unit).is_some_and(|value| value == prefix[prefix_index])
        {
            prefix_index += 1;
            continue;
        }
        prefix_index = prefix.len();
        output.push(unit);
    }
    output
}

fn block_parts(units: Vec<BlockUnit>) -> Vec<TextPart> {
    let mut output = Vec::new();
    let mut text = String::new();
    let mut text_span: Option<Span> = None;

    for unit in units {
        match unit {
            BlockUnit::Character { value, span } => {
                text.push(value);
                text_span = Some(match text_span {
                    Some(existing) => existing.union(span),
                    None => span,
                });
            }
            BlockUnit::Placeholder(placeholder) => {
                if let Some(span) = text_span.take() {
                    output.push(TextPart::Text(RawText {
                        value: std::mem::take(&mut text),
                        span,
                    }));
                }
                output.push(TextPart::Placeholder(placeholder));
            }
        }
    }
    if let Some(span) = text_span {
        output.push(TextPart::Text(RawText { value: text, span }));
    }
    output
}

fn is_newline(unit: &BlockUnit) -> bool {
    matches!(unit, BlockUnit::Character { value: '\n', .. })
}

fn is_whitespace(unit: &BlockUnit) -> bool {
    unit_character(unit).is_some_and(char::is_whitespace)
}

fn is_horizontal_whitespace(unit: &BlockUnit) -> bool {
    unit_character(unit).is_some_and(|value| value != '\n' && value.is_whitespace())
}

fn unit_character(unit: &BlockUnit) -> Option<char> {
    match unit {
        BlockUnit::Character { value, .. } => Some(*value),
        BlockUnit::Placeholder(_) => None,
    }
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
                kind: if arguments.is_some() {
                    ExpressionKind::Call
                } else {
                    ExpressionKind::Reference
                },
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
        members: Vec<LocaleGroupMember>,
        span: Span,
    },
}

enum LocaleGroupMember {
    Message(MessageImplementation),
    Group(MessageImplementationGroup),
}

impl LocaleGroupMember {
    fn set_docs(&mut self, docs: Vec<crate::DocComment>) {
        match self {
            Self::Message(message) => message.docs = docs,
            Self::Group(group) => group.docs = docs,
        }
    }
}

fn keyword_span<'tokens, I>(
    word: &'static str,
) -> impl Parser<'tokens, I, Span, Extra<'tokens>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind, Span = Span>,
{
    just(TokenKind::Ident(word.to_owned())).map_with(|_, extra| extra.span())
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

trait LocaleDeclarationSpan {
    fn extend_span(&mut self, prefix: Span);
}

impl LocaleDeclarationSpan for LocaleDeclaration {
    fn extend_span(&mut self, prefix: Span) {
        let span = match self {
            LocaleDeclaration::Enum(declaration) => &mut declaration.span,
            LocaleDeclaration::Variable(declaration) => &mut declaration.span,
            LocaleDeclaration::Form(declaration) => &mut declaration.span,
            LocaleDeclaration::Function(declaration) => &mut declaration.span,
            LocaleDeclaration::Message(declaration) => &mut declaration.span,
            LocaleDeclaration::Group(declaration) => &mut declaration.span,
            LocaleDeclaration::Override(declaration) => {
                declaration.extend_span(prefix);
                return;
            }
        };
        *span = prefix.union(*span);
    }
}
