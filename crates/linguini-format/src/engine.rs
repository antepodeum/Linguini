use crate::{
    ir::{FormatIr, FormatItem},
    FormatOptions,
};
use linguini_syntax::{Span, Token, TokenKind};
use std::borrow::Cow;

pub(crate) fn render_tokens(source: &str, tokens: &[Token], options: &FormatOptions) -> String {
    lower_tokens(source, tokens).render(options)
}

fn lower_tokens(source: &str, tokens: &[Token]) -> FormatIr {
    let mut ir = FormatIr::default();
    let mut previous: Option<&TokenKind> = None;
    let mut pending_space = false;
    let mut paren_depth = 0usize;
    let mut brace_stack = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        let next = next_significant_kind(tokens, index + 1);
        match &token.kind {
            TokenKind::Whitespace => {
                pending_space = true;
            }
            TokenKind::Newline => {
                if should_collapse_newline(previous, next, paren_depth) {
                    pending_space = true;
                } else {
                    ir.push(FormatItem::HardLine);
                    pending_space = false;
                }
            }
            TokenKind::Comment(text) => {
                if pending_space {
                    ir.push(FormatItem::Space);
                }
                ir.text(format!("//{}", text.trim_end()));
                pending_space = false;
            }
            TokenKind::DocComment(text) => {
                ir.text(format!("///{}", text.trim_end()));
                pending_space = false;
            }
            TokenKind::RBrace => {
                let placeholder_brace = brace_stack.pop().unwrap_or(false);
                if !placeholder_brace {
                    ir.push(FormatItem::Dedent);
                }
                lower_token_text(
                    source,
                    token,
                    &mut ir,
                    previous,
                    pending_space,
                    placeholder_brace,
                );
                pending_space = false;
            }
            TokenKind::LBrace => {
                let placeholder_brace = is_text_placeholder_start(previous, next);
                lower_token_text(
                    source,
                    token,
                    &mut ir,
                    previous,
                    pending_space,
                    placeholder_brace,
                );
                brace_stack.push(placeholder_brace);
                if !placeholder_brace {
                    ir.push(FormatItem::Indent);
                }
                pending_space = false;
            }
            TokenKind::LParen => {
                lower_token_text(
                    source,
                    token,
                    &mut ir,
                    previous,
                    pending_space,
                    *brace_stack.last().unwrap_or(&false),
                );
                paren_depth += 1;
                pending_space = false;
            }
            TokenKind::RParen => {
                paren_depth = paren_depth.saturating_sub(1);
                lower_token_text(
                    source,
                    token,
                    &mut ir,
                    previous,
                    pending_space,
                    *brace_stack.last().unwrap_or(&false),
                );
                pending_space = false;
            }
            _ => {
                lower_token_text(
                    source,
                    token,
                    &mut ir,
                    previous,
                    pending_space,
                    *brace_stack.last().unwrap_or(&false),
                );
                pending_space = false;
            }
        }

        if !matches!(token.kind, TokenKind::Whitespace | TokenKind::Newline) {
            previous = Some(&token.kind);
        }
    }

    ir
}

fn next_significant_kind(tokens: &[Token], start: usize) -> Option<&TokenKind> {
    tokens
        .iter()
        .skip(start)
        .find(|token| !matches!(token.kind, TokenKind::Whitespace | TokenKind::Newline))
        .map(|token| &token.kind)
}

fn is_text_placeholder_start(previous: Option<&TokenKind>, next: Option<&TokenKind>) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    if matches!(
        previous,
        TokenKind::Equals | TokenKind::Arrow | TokenKind::RawText(_) | TokenKind::TripleQuote
    ) {
        return true;
    }

    matches!(previous, TokenKind::RBrace) && is_placeholder_content_start(next)
}

fn is_placeholder_content_start(next: Option<&TokenKind>) -> bool {
    matches!(
        next,
        Some(
            TokenKind::Ident(_)
                | TokenKind::LocaleTag(_)
                | TokenKind::String(_)
                | TokenKind::At
                | TokenKind::RBrace
        )
    )
}

fn should_collapse_newline(
    previous: Option<&TokenKind>,
    next: Option<&TokenKind>,
    paren_depth: usize,
) -> bool {
    let (Some(previous), Some(next)) = (previous, next) else {
        return false;
    };

    if is_hard_layout_boundary(previous) || is_hard_layout_boundary(next) {
        return false;
    }

    if paren_depth > 0 {
        return true;
    }

    is_declaration_keyword(previous) && is_name_like(next)
        || is_name_like(previous)
            && matches!(
                next,
                TokenKind::LParen | TokenKind::LBrace | TokenKind::Equals
            )
        || is_annotation_target(previous) && matches!(next, TokenKind::At)
        || matches!(previous, TokenKind::RParen)
            && matches!(next, TokenKind::LBrace | TokenKind::At)
        || matches!(
            previous,
            TokenKind::Equals
                | TokenKind::Colon
                | TokenKind::Comma
                | TokenKind::Dot
                | TokenKind::At
        ) && is_name_like(next)
}

fn is_hard_layout_boundary(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Comment(_)
            | TokenKind::DocComment(_)
            | TokenKind::RawText(_)
            | TokenKind::TripleQuote
    )
}

fn is_declaration_keyword(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Ident(keyword)
            if matches!(keyword.as_str(), "enum" | "type" | "impl" | "fn" | "form" | "override")
    )
}

fn is_name_like(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Ident(_) | TokenKind::LocaleTag(_) | TokenKind::String(_)
    )
}

fn is_annotation_target(kind: &TokenKind) -> bool {
    is_name_like(kind) || matches!(kind, TokenKind::RParen)
}

fn lower_token_text(
    source: &str,
    token: &Token,
    ir: &mut FormatIr,
    previous: Option<&TokenKind>,
    pending_space: bool,
    placeholder_brace: bool,
) {
    let source_text = token_source(source, token);
    let text = rendered_token_text(source_text, &token.kind);
    if text.is_empty() {
        return;
    }

    if should_preserve_line_start(&token.kind) {
        ir.push(FormatItem::RawLineStart);
    }

    if should_space_before(
        previous,
        &token.kind,
        text.as_ref(),
        pending_space,
        placeholder_brace,
    ) {
        ir.push(FormatItem::Space);
    }

    if matches!(token.kind, TokenKind::Arrow) {
        ir.push(FormatItem::ArmMarkerStart);
        ir.text(text);
        ir.push(FormatItem::ArmMarkerEnd);
    } else {
        ir.text(text);
    }
}

fn rendered_token_text<'a>(source_text: &'a str, kind: &TokenKind) -> Cow<'a, str> {
    if should_preserve_token_source(kind) {
        Cow::Borrowed(source_text)
    } else {
        Cow::Borrowed(source_text.trim())
    }
}

fn should_preserve_token_source(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::RawText(_) | TokenKind::String(_) | TokenKind::TripleQuote
    )
}

fn should_preserve_line_start(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::RawText(_) | TokenKind::TripleQuote)
}

fn token_source<'a>(source: &'a str, token: &Token) -> &'a str {
    source.get(span_range(token.span)).unwrap_or_default()
}

fn span_range(span: Span) -> std::ops::Range<usize> {
    span.start..span.end
}

fn should_space_before(
    previous: Option<&TokenKind>,
    current: &TokenKind,
    current_text: &str,
    pending: bool,
    placeholder_brace: bool,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    if matches!(
        current,
        TokenKind::Comma
            | TokenKind::Colon
            | TokenKind::LParen
            | TokenKind::RParen
            | TokenKind::Dot
    ) || matches!(previous, TokenKind::LParen | TokenKind::Dot | TokenKind::At)
    {
        return false;
    }

    if current_text.chars().next().is_some_and(char::is_whitespace) {
        return false;
    }

    if placeholder_brace
        && (matches!(previous, TokenKind::LBrace) || matches!(current, TokenKind::RBrace))
    {
        return false;
    }

    if matches!(previous, TokenKind::RBrace) && matches!(current, TokenKind::RawText(_)) {
        return pending;
    }

    if matches!(previous, TokenKind::LBrace) || matches!(current, TokenKind::RBrace) {
        return true;
    }

    if matches!(
        previous,
        TokenKind::Comma | TokenKind::Colon | TokenKind::Equals | TokenKind::Arrow
    ) {
        return true;
    }

    pending
        || matches!(
            current,
            TokenKind::LBrace | TokenKind::Arrow | TokenKind::Equals
        )
        || matches!(
            previous,
            TokenKind::Ident(_) | TokenKind::LocaleTag(_) | TokenKind::String(_)
        ) && matches!(
            current,
            TokenKind::Ident(_)
                | TokenKind::LocaleTag(_)
                | TokenKind::String(_)
                | TokenKind::RawText(_)
        )
}
