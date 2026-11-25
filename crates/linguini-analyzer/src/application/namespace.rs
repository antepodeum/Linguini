use super::{
    is_strict_module_binding_identifier, token_is_other, ApplicationImportBinding,
    ApplicationImportBindingId, SourceId, Span, Token, TokenKind,
};

/// Generated metadata and controls whose exports do not expose message catalogs.
pub(super) fn is_metadata_export(name: &str) -> bool {
    matches!(
        name,
        "locales"
            | "baseLocale"
            | "localeDirections"
            | "isLocale"
            | "normalizeLocale"
            | "getTextDirection"
            | "setLocale"
    )
}

/// Read one static export from a module namespace, retaining its original token end.
pub(super) fn namespace_member(tokens: &[Token], start: usize) -> Option<(&str, usize)> {
    match tokens.get(start)?.kind {
        TokenKind::Dot => match &tokens.get(start + 1)?.kind {
            TokenKind::Identifier(name) => Some((name, start + 2)),
            _ => None,
        },
        TokenKind::OpenBracket => {
            match (&tokens.get(start + 1)?.kind, &tokens.get(start + 2)?.kind) {
                (TokenKind::StringLiteral(Some(name)), TokenKind::CloseBracket) => {
                    Some((name, start + 3))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn namespace_application_import_binding(
    tokens: &[Token],
    source: &str,
    source_id: SourceId,
    start: usize,
    end: usize,
    uncertain_syntax: bool,
) -> Option<ApplicationImportBinding> {
    if !token_is_other(tokens.get(start + 1), source, "*")
        || !matches!(&tokens.get(start + 2)?.kind, TokenKind::Identifier(name) if name == "as")
        || !matches!(&tokens.get(start + 4)?.kind, TokenKind::Identifier(name) if name == "from")
    {
        return None;
    }
    let local_token = tokens.get(start + 3)?;
    let TokenKind::Identifier(local) = &local_token.kind else {
        return None;
    };
    if !is_strict_module_binding_identifier(local) {
        return None;
    }
    let module_token = tokens.get(start + 5)?;
    let TokenKind::StringLiteral(Some(module_specifier)) = &module_token.kind else {
        return None;
    };
    if module_specifier.is_empty()
        || !(end == start + 6
            || end == start + 7 && matches!(tokens[start + 6].kind, TokenKind::Semicolon))
    {
        return None;
    }
    let declaration_span = Span::in_source(source_id, tokens[start].start, tokens[end - 1].end);
    let item_span = Span::in_source(source_id, tokens[start + 1].start, local_token.end);
    let following_start = tokens.get(end).map_or(source.len(), |token| token.start);
    let declaration_and_tail = &source[declaration_span.start..following_start];
    let safe = !uncertain_syntax
        && !declaration_and_tail.contains("//")
        && !declaration_and_tail.contains("/*");
    Some(ApplicationImportBinding {
        id: ApplicationImportBindingId {
            source: source_id,
            declaration_start: declaration_span.start,
            item_start: item_span.start,
        },
        module_specifier: module_specifier.clone(),
        imported: "*".to_owned(),
        local: local.clone(),
        declaration_span,
        item_span,
        removal_span: declaration_span,
        module_specifier_span: Span::in_source(source_id, module_token.start, module_token.end),
        imported_span: Span::in_source(source_id, tokens[start + 1].start, tokens[start + 1].end),
        local_span: Span::in_source(source_id, local_token.start, local_token.end),
        exact_uses_only: safe,
        tracked_uses_only: safe,
        retain_import: false,
    })
}
