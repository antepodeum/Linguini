use crate::{Diagnostic, PublicMessage};
use linguini_syntax::{SourceId, Span};
use std::collections::{BTreeMap, BTreeSet};

/// The syntactic form of a statically resolved application message reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ApplicationReferenceKind {
    /// A parameterless message value (for example, `l.main.title`).
    Value,
    /// A message invocation (for example, `l.main.items(count)`).
    Call,
}

/// The binding which supplied the root of an application reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApplicationBinding {
    /// The local identifier used in the source expression.
    pub local: String,
    /// How the local identifier was established.
    pub provenance: ApplicationBindingProvenance,
}

/// Provenance for an application root binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ApplicationBindingProvenance {
    /// A named import, preserving both the exact module specifier and imported symbol.
    Imported {
        module_specifier: String,
        imported: String,
    },
    /// A result produced by a supported Linguini factory.
    Factory { factory: String },
    /// A conservative implicit root (`l`, `lgl`, or `messages`).
    Implicit,
}

/// Stable identity for one removable named application import binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApplicationImportBindingId {
    pub source: SourceId,
    pub declaration_start: usize,
    pub item_start: usize,
}

/// Exact source metadata required to remove an imported application root safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationImportBinding {
    pub id: ApplicationImportBindingId,
    pub module_specifier: String,
    pub imported: String,
    pub local: String,
    pub declaration_span: Span,
    pub item_span: Span,
    pub removal_span: Span,
    pub module_specifier_span: Span,
    pub imported_span: Span,
    pub local_span: Span,
    /// True only when parsing was unambiguous and every non-shadowed use is an exact static ref.
    pub exact_uses_only: bool,
}

/// A deterministic, source-aware static application message reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationReference {
    /// Canonical dotted message path, with safe generated identifiers decoded.
    pub canonical_path: String,
    /// Source span from the root through the final static member only.
    pub span: Span,
    /// Whether the path is used as a value or called.
    pub kind: ApplicationReferenceKind,
    /// Root/local binding metadata for the expression.
    pub binding: ApplicationBinding,
    /// Exact import identity when this reference originates from a removable named import.
    pub import_binding: Option<ApplicationImportBindingId>,
}

impl ApplicationReference {
    fn new(
        canonical_path: String,
        span: Span,
        kind: ApplicationReferenceKind,
        binding: ApplicationBinding,
        import_binding: Option<ApplicationImportBindingId>,
    ) -> Self {
        Self {
            canonical_path,
            span,
            kind,
            binding,
            import_binding,
        }
    }
}

/// Conservative application-side references to generated `l.*` message paths.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplicationUsage {
    static_paths: BTreeSet<UsagePath>,
    dynamic_prefixes: BTreeSet<UsagePath>,
    references: Vec<ApplicationReference>,
    imports: Vec<ApplicationImportBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UsagePath {
    display: String,
    segments: Vec<UsageSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UsageSegment {
    raw: String,
    decoded: Option<Vec<String>>,
}

struct ReferenceContext<'a> {
    tokens: &'a [Token],
    source: &'a str,
    source_id: SourceId,
    root_index: usize,
    root: &'a str,
    binding: ApplicationBinding,
    emit_reference: bool,
}

impl ApplicationUsage {
    pub fn from_source(source: &str) -> Self {
        Self::from_source_in(source, SourceId::default())
    }

    pub fn from_source_in(source: &str, source_id: SourceId) -> Self {
        let mut usage = Self::default();
        usage.extend_source_in(source, source_id);
        usage
    }

    pub fn extend_source(&mut self, source: &str) {
        self.extend_source_in(source, SourceId::default());
    }

    pub fn extend_source_in(&mut self, source: &str, source_id: SourceId) {
        let LexedSource {
            tokens,
            uncertain_syntax,
        } = lex(source);
        if uncertain_syntax {
            self.dynamic_prefixes.insert(UsagePath::root());
        }
        let ImportedSymbols {
            imports,
            mut roots,
            factories,
            bindings,
        } = imported_symbols(&tokens, source, source_id, uncertain_syntax);
        let import_start = self.imports.len();
        self.imports.extend(bindings);
        for index in 0..tokens.len() {
            let Some((name, after_symbol, _)) = symbol_reference(&tokens, source, index) else {
                continue;
            };
            if imports.contains(&index) || !factories.contains_key(name) {
                continue;
            }
            let Some(after_call) = call_end(&tokens, source, after_symbol) else {
                continue;
            };
            if member_path(&tokens, after_call).0.is_empty() {
                if let Some((binding, _)) = declaration_binding(&tokens, source, index) {
                    let factory = factories
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.to_owned());
                    roots.insert(
                        binding.clone(),
                        ApplicationBinding {
                            local: binding,
                            provenance: ApplicationBindingProvenance::Factory { factory },
                        },
                    );
                }
            }
        }

        let mut index = 0;
        while index < tokens.len() {
            if imports.contains(&index) {
                index += 1;
                continue;
            }
            let Some((root, after_symbol, bracket_property)) =
                symbol_reference(&tokens, source, index)
            else {
                index += 1;
                continue;
            };
            if factories.contains_key(root) {
                if let Some(after_call) = call_end(&tokens, source, after_symbol) {
                    let (segments, dynamic, next) = member_path(&tokens, after_call);
                    if segments.is_empty() {
                        if declaration_binding(&tokens, source, index)
                            .map_or(true, |(_, exported)| exported)
                        {
                            self.dynamic_prefixes.insert(UsagePath::root());
                        }
                    } else {
                        let factory = factories
                            .get(root)
                            .cloned()
                            .unwrap_or_else(|| root.to_owned());
                        self.record_path(
                            ReferenceContext {
                                tokens: &tokens,
                                source,
                                source_id,
                                root_index: index,
                                root,
                                binding: ApplicationBinding {
                                    local: root.to_owned(),
                                    provenance: ApplicationBindingProvenance::Factory { factory },
                                },
                                emit_reference: true,
                            },
                            segments,
                            dynamic,
                            next,
                        );
                    }
                    index += 1;
                    continue;
                }
                self.dynamic_prefixes.insert(UsagePath::root());
                index += 1;
                continue;
            }
            if !bracket_property && is_binding_declaration(&tokens, source, index) {
                index += 1;
                continue;
            }
            if !roots.contains_key(root)
                || (!bracket_property
                    && is_member_property(&tokens, index)
                    && !matches!(root, "l" | "lgl" | "messages"))
            {
                index += 1;
                continue;
            }

            let (segments, dynamic, next) = member_path(&tokens, after_symbol);
            let binding = roots
                .get(root)
                .cloned()
                .unwrap_or_else(|| ApplicationBinding {
                    local: root.to_owned(),
                    provenance: ApplicationBindingProvenance::Implicit,
                });
            let member_property = is_member_property(&tokens, index)
                || (bracket_property && is_bracket_member_property(&tokens, source, index));
            let imported = matches!(
                &binding.provenance,
                &ApplicationBindingProvenance::Imported { .. }
            );
            let emit_reference = !imported
                || (!member_property && !is_shadowed_import_binding(&tokens, source, index, root));
            self.record_path(
                ReferenceContext {
                    tokens: &tokens,
                    source,
                    source_id,
                    root_index: index,
                    root,
                    binding,
                    emit_reference,
                },
                segments,
                dynamic,
                next,
            );
            index = next.max(index + 1);
        }
        self.finalize_import_safety(source, &tokens, &imports, import_start);
        self.poison_duplicate_import_locals();
        self.sort_references();
        self.sort_imports();
    }

    fn sort_references(&mut self) {
        self.references.sort_by(|left, right| {
            (
                left.span.source,
                left.span.start,
                left.span.end,
                &left.canonical_path,
                left.kind,
                &left.binding,
            )
                .cmp(&(
                    right.span.source,
                    right.span.start,
                    right.span.end,
                    &right.canonical_path,
                    right.kind,
                    &right.binding,
                ))
        });
    }

    fn sort_imports(&mut self) {
        self.imports.sort_by_key(|binding| binding.id);
    }

    fn record_path(
        &mut self,
        context: ReferenceContext<'_>,
        segments: Vec<UsageSegment>,
        dynamic: bool,
        next: usize,
    ) {
        if segments.is_empty() {
            self.dynamic_prefixes.insert(UsagePath::root());
            return;
        }
        let path = UsagePath::new(segments);
        let invoked = is_invocation(context.tokens, context.source, next);
        if dynamic {
            self.dynamic_prefixes.insert(path);
        } else {
            if invoked {
                self.static_paths.insert(path.clone());
            } else {
                // Keep value reads conservative for unused-message analysis while
                // still exposing their exact static spans to bundler transforms.
                self.dynamic_prefixes.insert(path.clone());
            }
            let Some(root_token) =
                reference_root_token(context.tokens, context.source, context.root_index)
            else {
                return;
            };
            let end = context
                .tokens
                .get(next.saturating_sub(1))
                .map_or(root_token.end, |token| token.end);
            let kind = if invoked {
                ApplicationReferenceKind::Call
            } else {
                ApplicationReferenceKind::Value
            };
            if context.emit_reference {
                let import_binding = self.import_binding_id(&context.binding, context.source_id);
                self.references.push(ApplicationReference::new(
                    path.display,
                    Span::in_source(context.source_id, root_token.start, end),
                    kind,
                    if context.binding.local == context.root {
                        context.binding
                    } else {
                        ApplicationBinding {
                            local: context.root.to_owned(),
                            provenance: context.binding.provenance,
                        }
                    },
                    import_binding,
                ));
            }
        }
    }

    fn import_binding_id(
        &self,
        binding: &ApplicationBinding,
        source: SourceId,
    ) -> Option<ApplicationImportBindingId> {
        let ApplicationBindingProvenance::Imported { .. } = &binding.provenance else {
            return None;
        };
        let mut matches = self
            .imports
            .iter()
            .filter(|candidate| candidate.id.source == source && candidate.local == binding.local);
        let id = matches.next()?.id;
        matches.next().is_none().then_some(id)
    }

    fn finalize_import_safety(
        &mut self,
        source: &str,
        tokens: &[Token],
        import_tokens: &BTreeSet<usize>,
        import_start: usize,
    ) {
        for binding_index in import_start..self.imports.len() {
            let id = self.imports[binding_index].id;
            let local = self.imports[binding_index].local.clone();
            let duplicate = self
                .imports
                .iter()
                .filter(|candidate| candidate.id.source == id.source && candidate.local == local)
                .count()
                > 1;
            let mut safe = self.imports[binding_index].exact_uses_only && !duplicate;
            for (token_index, token) in tokens.iter().enumerate() {
                if !matches!(&token.kind, TokenKind::Identifier(name) if name == &local)
                    || import_tokens.contains(&token_index)
                    || is_member_property(tokens, token_index)
                {
                    continue;
                }
                if is_binding_declaration(tokens, source, token_index) {
                    if is_top_level(tokens, source, token_index) {
                        safe = false;
                    }
                    continue;
                }
                if is_shadowed_import_binding(tokens, source, token_index, &local) {
                    continue;
                }
                let reference = self.references.iter().find(|reference| {
                    reference.span.source == id.source
                        && reference.span.start == token.start
                        && reference.import_binding == Some(id)
                });
                let Some(reference) = reference else {
                    safe = false;
                    continue;
                };
                if source[reference.span.start..reference.span.end].contains("?.")
                    || analyzer_optional_invocation(source, reference.span.end)
                {
                    safe = false;
                }
            }
            self.imports[binding_index].exact_uses_only = safe;
        }
    }

    fn poison_duplicate_import_locals(&mut self) {
        let mut counts = BTreeMap::<(SourceId, String), usize>::new();
        for binding in &self.imports {
            *counts
                .entry((binding.id.source, binding.local.clone()))
                .or_default() += 1;
        }
        for binding in &mut self.imports {
            if counts
                .get(&(binding.id.source, binding.local.clone()))
                .is_some_and(|count| *count > 1)
            {
                binding.exact_uses_only = false;
            }
        }
        for reference in &mut self.references {
            if reference.import_binding.is_some_and(|id| {
                counts
                    .get(&(id.source, reference.binding.local.clone()))
                    .is_some_and(|count| *count > 1)
            }) {
                reference.import_binding = None;
            }
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.static_paths.extend(other.static_paths);
        self.dynamic_prefixes.extend(other.dynamic_prefixes);
        self.references.extend(other.references);
        self.imports.extend(other.imports);
        self.poison_duplicate_import_locals();
        self.sort_references();
        self.sort_imports();
    }

    pub fn static_paths(&self) -> impl Iterator<Item = &str> {
        self.static_paths.iter().map(|path| path.display.as_str())
    }

    pub fn dynamic_prefixes(&self) -> impl Iterator<Item = &str> {
        self.dynamic_prefixes
            .iter()
            .map(|path| path.display.as_str())
    }

    /// Static references in deterministic source/path order.
    pub fn references(&self) -> impl Iterator<Item = &ApplicationReference> {
        self.references.iter()
    }

    /// Imported `l`/`messages` bindings in deterministic source/declaration order.
    pub fn imports(&self) -> impl Iterator<Item = &ApplicationImportBinding> {
        self.imports.iter()
    }
}

fn reference_root_token<'a>(tokens: &'a [Token], source: &str, index: usize) -> Option<&'a Token> {
    let token = tokens.get(index)?;
    if !matches!(token.kind, TokenKind::StringLiteral(Some(_))) {
        return Some(token);
    }
    let receiver = index.checked_sub(2)?;
    if can_end_member_receiver(&tokens[receiver], source) {
        return tokens.get(receiver);
    }
    if matches!(tokens[receiver].kind, TokenKind::Dot)
        && matches!(
            receiver
                .checked_sub(1)
                .and_then(|position| tokens.get(position))
                .map(|token| &token.kind),
            Some(TokenKind::Question)
        )
    {
        return tokens.get(receiver.checked_sub(2)?);
    }
    Some(token)
}

fn is_bracket_member_property(tokens: &[Token], source: &str, index: usize) -> bool {
    if !matches!(
        tokens.get(index).map(|token| &token.kind),
        Some(TokenKind::StringLiteral(_))
    ) || !is_bracket_member(tokens, source, index)
    {
        return false;
    }
    let Some(receiver) = index.checked_sub(2) else {
        return false;
    };
    can_end_member_receiver(&tokens[receiver], source)
        || (matches!(tokens[receiver].kind, TokenKind::Dot)
            && matches!(
                receiver
                    .checked_sub(1)
                    .and_then(|position| tokens.get(position))
                    .map(|token| &token.kind),
                Some(TokenKind::Question)
            ))
}

fn is_shadowed_import_binding(
    tokens: &[Token],
    source: &str,
    reference: usize,
    name: &str,
) -> bool {
    let braces = delimiter_pairs(tokens, source, "{", "}");
    let mut shadow_ranges = Vec::new();
    for index in 0..tokens.len() {
        let Some(TokenKind::Identifier(candidate)) = tokens.get(index).map(|token| &token.kind)
        else {
            continue;
        };
        if candidate != name || is_import_declaration(tokens, source, index) {
            continue;
        }

        if let Some(scope) = variable_binding_scope(tokens, source, index, &braces) {
            shadow_ranges.push(scope);
        }
        if let Some(scope) = declaration_binding_scope(tokens, source, index, &braces) {
            shadow_ranges.push(scope);
        }
        if let Some(scope) = parameter_body_scope(tokens, source, index) {
            shadow_ranges.push(scope);
        }
        if let Some(scope) = expression_name_scope(tokens, source, index, &braces) {
            shadow_ranges.push(scope);
        }
    }

    shadow_ranges
        .into_iter()
        .any(|(start, end)| start <= reference && reference < end)
}

fn delimiter_pairs(
    tokens: &[Token],
    source: &str,
    opening: &str,
    closing: &str,
) -> Vec<(usize, usize)> {
    (0..tokens.len())
        .filter_map(|index| {
            (token_spelling(tokens, source, index) == Some(opening))
                .then(|| matching_delimiter(tokens, source, index, opening, closing))
                .flatten()
                .map(|close| (index, close))
        })
        .collect()
}

fn variable_binding_scope(
    tokens: &[Token],
    source: &str,
    binding: usize,
    braces: &[(usize, usize)],
) -> Option<(usize, usize)> {
    let previous = binding.checked_sub(1)?;
    let direct = matches!(
        tokens.get(previous).map(|token| &token.kind),
        Some(TokenKind::Identifier(keyword))
            if matches!(keyword.as_str(), "const" | "let" | "var")
    );
    if direct {
        return Some(scope_for_index(
            tokens,
            source,
            braces,
            binding,
            tokens.len(),
        ));
    }

    let mut cursor = previous;
    let mut pattern_open = None;
    while cursor > 0 {
        if matches!(token_spelling(tokens, source, cursor), Some("{" | "[")) {
            pattern_open = Some(cursor);
            break;
        }
        if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) || matches!(
            token_spelling(tokens, source, cursor),
            Some("=" | ";" | "=>")
        ) {
            return None;
        }
        cursor -= 1;
    }
    let pattern_open = pattern_open?;
    let keyword = (0..pattern_open).rev().find(|&index| {
        matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Identifier(keyword))
                if matches!(keyword.as_str(), "const" | "let" | "var")
        )
    })?;
    if let Some(pattern_start) = (keyword + 1..pattern_open)
        .find(|&index| matches!(token_spelling(tokens, source, index), Some("{" | "[")))
    {
        let pattern_close = matching_delimiter(
            tokens,
            source,
            pattern_start,
            token_spelling(tokens, source, pattern_start)?,
            if token_spelling(tokens, source, pattern_start) == Some("{") {
                "}"
            } else {
                "]"
            },
        )?;
        if binding > pattern_close && token_spelling(tokens, source, pattern_close + 1) == Some(":")
        {
            return None;
        }
    }
    if matches!(
        tokens.get(keyword + 1).map(|token| &token.kind),
        Some(TokenKind::Identifier(_))
    ) && (keyword + 1..pattern_open)
        .any(|index| token_spelling(tokens, source, index) == Some(":"))
    {
        return None;
    }
    if (keyword + 1..pattern_open).any(|index| {
        matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) || token_spelling(tokens, source, index) == Some("=")
    }) {
        return None;
    }
    Some(scope_for_index(
        tokens,
        source,
        braces,
        binding,
        tokens.len(),
    ))
}

fn declaration_binding_scope(
    tokens: &[Token],
    source: &str,
    binding: usize,
    braces: &[(usize, usize)],
) -> Option<(usize, usize)> {
    let keyword = binding.checked_sub(1)?;
    let declaration = matches!(
        tokens.get(keyword).map(|token| &token.kind),
        Some(TokenKind::Identifier(name)) if name == "function" || name == "class"
    );
    if !declaration || expression_keyword(tokens, source, keyword) {
        return None;
    }
    Some(scope_for_index(
        tokens,
        source,
        braces,
        binding,
        tokens.len(),
    ))
}

fn expression_keyword(tokens: &[Token], source: &str, keyword: usize) -> bool {
    let Some(previous) = keyword.checked_sub(1) else {
        return false;
    };
    matches!(
        token_spelling(tokens, source, previous),
        Some("=" | ":" | "," | "(" | "[")
    ) || matches!(
        tokens.get(previous).map(|token| &token.kind),
        Some(TokenKind::Identifier(name)) if name == "return"
    )
}

fn parameter_body_scope(
    tokens: &[Token],
    source: &str,
    parameter: usize,
) -> Option<(usize, usize)> {
    if token_spelling(tokens, source, parameter + 1) == Some("=")
        && token_spelling(tokens, source, parameter + 2) == Some(">")
    {
        let body_start = parameter + 3;
        if token_spelling(tokens, source, body_start) == Some("{") {
            let body_close = matching_delimiter(tokens, source, body_start, "{", "}")?;
            return Some((body_start, body_close));
        }
        let end = concise_arrow_end(tokens, source, body_start);
        return Some((body_start, end));
    }
    let (open, close) = enclosing_delimiter(tokens, source, parameter, "(", ")")?;
    let function_parameter = function_parameter_list(tokens, source, open);
    let method_parameter = method_parameter_list(tokens, source, open) && !function_parameter;
    let tail = signature_tail(tokens, source, close);
    let arrow_parameter = tail.is_some_and(|tail| matches!(tail, SignatureTail::Arrow(_)));
    if !function_parameter && !method_parameter && !arrow_parameter {
        return None;
    }
    if arrow_parameter {
        let SignatureTail::Arrow(arrow) = tail? else {
            unreachable!();
        };
        let body_start = arrow + 2;
        if token_spelling(tokens, source, body_start) == Some("{") {
            let body_open = body_start;
            let body_close = matching_delimiter(tokens, source, body_open, "{", "}")?;
            return Some((body_open, body_close));
        }
        return Some((body_start, concise_arrow_end(tokens, source, body_start)));
    }
    let body_open = match tail? {
        SignatureTail::Body(body_open) => body_open,
        SignatureTail::Arrow(_) => unreachable!(),
    };
    if (function_parameter || method_parameter)
        && token_spelling(tokens, source, body_open) == Some("{")
    {
        let body_close = matching_delimiter(tokens, source, body_open, "{", "}")?;
        return Some((body_open, body_close));
    }
    None
}

#[derive(Debug, Clone, Copy)]
enum SignatureTail {
    Arrow(usize),
    Body(usize),
}

fn method_parameter_list(tokens: &[Token], source: &str, open: usize) -> bool {
    let mut cursor = open.checked_sub(1);
    if cursor.is_some_and(|index| token_spelling(tokens, source, index) == Some("*")) {
        cursor = cursor.and_then(|index| index.checked_sub(1));
    }
    if cursor.is_some_and(|index| token_spelling(tokens, source, index) == Some(">")) {
        let Some(generic_close) = cursor else {
            return false;
        };
        let Some(generic_open) = matching_angle_open(tokens, source, generic_close) else {
            return false;
        };
        cursor = generic_open.checked_sub(1);
        if cursor.is_some_and(|index| token_spelling(tokens, source, index) == Some("*")) {
            cursor = cursor.and_then(|index| index.checked_sub(1));
        }
    }
    let Some(cursor) = cursor else {
        return false;
    };
    let Some(TokenKind::Identifier(name)) = tokens.get(cursor).map(|token| &token.kind) else {
        return false;
    };
    !matches!(
        name.as_str(),
        "if" | "for" | "while" | "switch" | "with" | "catch"
    )
}

fn matching_angle_open(tokens: &[Token], source: &str, close: usize) -> Option<usize> {
    let mut depth = 0_usize;
    for index in (0..=close).rev() {
        match token_spelling(tokens, source, index) {
            Some(">") => depth += 1,
            Some("<") => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn signature_tail(tokens: &[Token], source: &str, close: usize) -> Option<SignatureTail> {
    let mut index = close + 1;
    let mut saw_type = false;
    while index < tokens.len() {
        if token_spelling(tokens, source, index) == Some("=")
            && token_spelling(tokens, source, index + 1) == Some(">")
        {
            return Some(SignatureTail::Arrow(index));
        }
        if token_spelling(tokens, source, index) == Some(":") {
            saw_type = true;
            index += 1;
            continue;
        }
        if matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) || token_spelling(tokens, source, index) == Some("}")
        {
            return None;
        }
        if token_spelling(tokens, source, index) == Some("{") {
            let body_close = matching_delimiter(tokens, source, index, "{", "}")?;
            if saw_type {
                let after_close = body_close + 1;
                if token_spelling(tokens, source, after_close) == Some("=")
                    && token_spelling(tokens, source, after_close + 1) == Some(">")
                {
                    return Some(SignatureTail::Arrow(after_close));
                }
                if matches!(
                    token_spelling(tokens, source, after_close),
                    Some(">" | "," | "{" | "=")
                ) {
                    index = after_close + 1;
                    continue;
                }
                return Some(SignatureTail::Body(index));
            }
            return Some(SignatureTail::Body(index));
        }
        index += 1;
    }
    None
}

fn concise_arrow_end(tokens: &[Token], source: &str, start: usize) -> usize {
    let mut depth = delimiter_depth_before(tokens, source, start);
    let baseline = depth;
    for index in start..tokens.len() {
        match token_spelling(tokens, source, index) {
            Some("(") => depth.0 += 1,
            Some(")") => {
                if depth.0 == baseline.0 {
                    return index;
                }
                depth.0 = depth.0.saturating_sub(1);
            }
            Some("[") => depth.1 += 1,
            Some("]") => {
                if depth.1 == baseline.1 {
                    return index;
                }
                depth.1 = depth.1.saturating_sub(1);
            }
            Some("{") => depth.2 += 1,
            Some("}") => {
                if depth.2 == baseline.2 {
                    return index;
                }
                depth.2 = depth.2.saturating_sub(1);
            }
            Some(",") if depth == baseline => return index,
            Some(";") => return index,
            _ => {}
        }
    }
    tokens.len()
}

fn delimiter_depth_before(tokens: &[Token], source: &str, end: usize) -> (usize, usize, usize) {
    let mut depth = (0_usize, 0_usize, 0_usize);
    for index in 0..end {
        match token_spelling(tokens, source, index) {
            Some("(") => depth.0 += 1,
            Some(")") => depth.0 = depth.0.saturating_sub(1),
            Some("[") => depth.1 += 1,
            Some("]") => depth.1 = depth.1.saturating_sub(1),
            Some("{") => depth.2 += 1,
            Some("}") => depth.2 = depth.2.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn function_parameter_list(tokens: &[Token], source: &str, open: usize) -> bool {
    let mut cursor = open;
    while let Some(previous) = cursor.checked_sub(1) {
        cursor = previous;
        if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) || matches!(
            token_spelling(tokens, source, cursor),
            Some("{" | "}" | "=" | ";")
        ) {
            break;
        }
        if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Identifier(keyword)) if keyword == "function" || keyword == "catch"
        ) || matches!(
            token_spelling(tokens, source, cursor),
            Some("function" | "catch")
        ) {
            return true;
        }
    }
    false
}

fn enclosing_delimiter(
    tokens: &[Token],
    source: &str,
    index: usize,
    opening: &str,
    closing: &str,
) -> Option<(usize, usize)> {
    (0..index)
        .rev()
        .filter(|&open| token_spelling(tokens, source, open) == Some(opening))
        .find_map(|open| {
            let close = matching_delimiter(tokens, source, open, opening, closing)?;
            (index < close).then_some((open, close))
        })
}

fn expression_name_scope(
    tokens: &[Token],
    source: &str,
    name: usize,
    _braces: &[(usize, usize)],
) -> Option<(usize, usize)> {
    let keyword = name.checked_sub(1)?;
    let kind = match tokens.get(keyword).map(|token| &token.kind) {
        Some(TokenKind::Identifier(kind)) if kind == "function" || kind == "class" => kind,
        _ => return None,
    };
    let _ = kind;
    let body_open = if kind == "class" {
        (name + 1..tokens.len())
            .find(|&index| token_spelling(tokens, source, index) == Some("{"))?
    } else {
        let open = (name + 1..tokens.len())
            .find(|&index| token_spelling(tokens, source, index) == Some("("))?;
        let close = matching_delimiter(tokens, source, open, "(", ")")?;
        (close + 1..tokens.len())
            .find(|&index| token_spelling(tokens, source, index) == Some("{"))?
    };
    let body_close = matching_delimiter(tokens, source, body_open, "{", "}")?;
    let before_keyword = keyword.checked_sub(1);
    let expression = before_keyword.is_some_and(|index| {
        matches!(
            token_spelling(tokens, source, index),
            Some("=" | ":" | "," | "(" | "[" | "return")
        ) || matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Identifier(value)) if value == "return"
        )
    });
    expression.then_some((body_open, body_close))
}

fn is_import_declaration(tokens: &[Token], source: &str, index: usize) -> bool {
    let mut cursor = index;
    while let Some(previous) = cursor.checked_sub(1) {
        cursor = previous;
        if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) || token_spelling(tokens, source, cursor) == Some(";")
        {
            break;
        }
        if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Identifier(name)) if name == "import"
        ) {
            return true;
        }
    }
    false
}

fn scope_for_index(
    tokens: &[Token],
    source: &str,
    braces: &[(usize, usize)],
    index: usize,
    token_count: usize,
) -> (usize, usize) {
    braces
        .iter()
        .filter(|(open, close)| {
            *open < index && index < *close && !is_binding_pattern_delimiter(tokens, source, *open)
        })
        .min_by_key(|(open, close)| close - open)
        .copied()
        .unwrap_or((0, token_count))
}

fn is_binding_pattern_delimiter(tokens: &[Token], source: &str, open: usize) -> bool {
    let mut cursor = open;
    while let Some(previous) = cursor.checked_sub(1) {
        cursor = previous;
        if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) || token_spelling(tokens, source, cursor) == Some("=")
        {
            return false;
        }
        if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Identifier(keyword))
                if matches!(keyword.as_str(), "const" | "let" | "var")
        ) {
            return true;
        }
    }
    false
}

fn matching_delimiter(
    tokens: &[Token],
    source: &str,
    open: usize,
    opening: &str,
    closing: &str,
) -> Option<usize> {
    let mut depth = 0_usize;
    for index in open..tokens.len() {
        match token_spelling(tokens, source, index) {
            Some(spelling) if spelling == opening => depth += 1,
            Some(spelling) if spelling == closing => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn token_spelling<'a>(tokens: &'a [Token], source: &'a str, index: usize) -> Option<&'a str> {
    let token = tokens.get(index)?;
    match token.kind {
        TokenKind::Other => Some(&source[token.start..token.end]),
        TokenKind::OpenBracket => Some("["),
        TokenKind::CloseBracket => Some("]"),
        TokenKind::Semicolon => Some(";"),
        _ => None,
    }
}

pub fn analyze_unused_messages(
    schema_messages: &[PublicMessage],
    usage: &ApplicationUsage,
    ignored_prefixes: &[String],
) -> Vec<Diagnostic> {
    schema_messages
        .iter()
        .filter(|message| {
            !usage
                .static_paths
                .iter()
                .any(|path| path.overlaps(&message.name))
                && !usage
                    .dynamic_prefixes
                    .iter()
                    .any(|prefix| prefix.overlaps(&message.name))
                && !ignored_prefixes
                    .iter()
                    .any(|prefix| path_is_within(&message.name, prefix))
        })
        .map(|message| {
            Diagnostic::warning(
                format!(
                    "schema message `{}` is not referenced by configured application sources",
                    message.name
                ),
                message.span,
            )
            .as_lint("unused_message")
            .with_note(
                "remove the message or add its canonical path to \
                 `analysis.unused_messages.ignore` when it is selected dynamically",
            )
        })
        .collect()
}

fn path_is_within(path: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

impl UsagePath {
    fn root() -> Self {
        Self {
            display: String::new(),
            segments: Vec::new(),
        }
    }

    fn new(segments: Vec<UsageSegment>) -> Self {
        let display = segments
            .iter()
            .flat_map(|segment| {
                segment.decoded.as_ref().map_or_else(
                    || vec![segment.raw.as_str()],
                    |decoded| decoded.iter().map(String::as_str).collect(),
                )
            })
            .collect::<Vec<_>>()
            .join(".");
        Self { display, segments }
    }

    fn overlaps(&self, canonical: &str) -> bool {
        if self.segments.is_empty() || canonical.is_empty() {
            return true;
        }
        let canonical = canonical.split('.').collect::<Vec<_>>();
        let mut reachable = BTreeSet::from([0_usize]);
        for segment in &self.segments {
            let mut next = BTreeSet::new();
            for position in reachable {
                if position == canonical.len() {
                    return true;
                }
                let raw = std::slice::from_ref(&segment.raw);
                let alternatives =
                    std::iter::once(raw).chain(segment.decoded.iter().map(Vec::as_slice));
                for alternative in alternatives {
                    let remaining = &canonical[position..];
                    let shared = remaining.len().min(alternative.len());
                    if remaining[..shared]
                        .iter()
                        .zip(&alternative[..shared])
                        .any(|(canonical, candidate)| *canonical != candidate.as_str())
                    {
                        continue;
                    }
                    if remaining.len() <= alternative.len() {
                        return true;
                    }
                    next.insert(position + alternative.len());
                }
            }
            if next.is_empty() {
                return false;
            }
            reachable = next;
        }
        !reachable.is_empty()
    }
}

impl UsageSegment {
    fn new(raw: String) -> Self {
        let decoded = decode_generated_identifier(&raw)
            .filter(|decoded| decoded != &raw)
            .map(|decoded| decoded.split('.').map(str::to_owned).collect());
        Self { raw, decoded }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Identifier(String),
    StringLiteral(Option<String>),
    OpaqueLiteral,
    Dot,
    Question,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Other,
}

fn symbol_reference<'a>(
    tokens: &'a [Token],
    source: &str,
    index: usize,
) -> Option<(&'a str, usize, bool)> {
    match &tokens.get(index)?.kind {
        TokenKind::Identifier(name) => Some((name, index + 1, false)),
        TokenKind::StringLiteral(Some(name)) if is_bracket_member(tokens, source, index) => {
            Some((name, index + 2, true))
        }
        _ => None,
    }
}

fn is_bracket_member(tokens: &[Token], source: &str, index: usize) -> bool {
    if !matches!(
        index.checked_sub(1).and_then(|index| tokens.get(index)),
        Some(Token {
            kind: TokenKind::OpenBracket,
            ..
        })
    ) || !matches!(
        tokens.get(index + 1),
        Some(Token {
            kind: TokenKind::CloseBracket,
            ..
        })
    ) {
        return false;
    }
    let Some(receiver) = index.checked_sub(2) else {
        return false;
    };
    if can_end_member_receiver(&tokens[receiver], source) {
        return true;
    }
    matches!(tokens[receiver].kind, TokenKind::Dot)
        && matches!(
            receiver
                .checked_sub(1)
                .and_then(|index| tokens.get(index))
                .map(|token| &token.kind),
            Some(TokenKind::Question)
        )
        && receiver
            .checked_sub(2)
            .and_then(|index| tokens.get(index))
            .is_some_and(|token| can_end_member_receiver(token, source))
}

fn can_end_member_receiver(token: &Token, source: &str) -> bool {
    matches!(
        token.kind,
        TokenKind::Identifier(_) | TokenKind::StringLiteral(_) | TokenKind::CloseBracket
    ) || (matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == ")")
}

struct LexedSource {
    tokens: Vec<Token>,
    uncertain_syntax: bool,
}

fn lex(source: &str) -> LexedSource {
    let mut tokens = Vec::new();
    let mut position = 0;
    let mut uncertain_syntax = false;
    lex_code(
        source,
        &mut position,
        false,
        &mut tokens,
        &mut uncertain_syntax,
    );
    LexedSource {
        tokens,
        uncertain_syntax,
    }
}

fn lex_code(
    source: &str,
    position: &mut usize,
    stop_at_template_brace: bool,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) {
    let bytes = source.as_bytes();
    let mut brace_depth = 0_usize;
    let context_start = tokens.len();
    while *position < bytes.len() {
        let byte = bytes[*position];
        if byte.is_ascii_whitespace() {
            *position += 1;
            continue;
        }
        if byte == b'/' && bytes.get(*position + 1) == Some(&b'/') {
            *position += 2;
            while *position < bytes.len() && bytes[*position] != b'\n' {
                *position += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(*position + 1) == Some(&b'*') {
            *position += 2;
            while *position + 1 < bytes.len()
                && !(bytes[*position] == b'*' && bytes[*position + 1] == b'/')
            {
                *position += 1;
            }
            if *position + 1 == bytes.len() {
                *uncertain_syntax = true;
                *position = bytes.len();
            } else {
                *position += 2;
            }
            continue;
        }
        if byte == b'/' && can_start_regex(&tokens[context_start..], source) {
            if !lex_regex(source, position, tokens) {
                *uncertain_syntax = true;
            }
            continue;
        }
        if byte == b'<'
            && can_start_markup(&tokens[context_start..], source, *position)
            && lex_markup(source, position, tokens, uncertain_syntax)
        {
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            if !lex_string(source, position, tokens) {
                *uncertain_syntax = true;
            }
            continue;
        }
        if byte == b'`' {
            if !lex_template(source, position, tokens, uncertain_syntax) {
                *uncertain_syntax = true;
            }
            continue;
        }
        if byte == b'{' && lex_svelte_block_close(source, position) {
            continue;
        }
        if byte == b'{' {
            brace_depth += 1;
            push_token(tokens, TokenKind::Other, *position, *position + 1);
            *position += 1;
            continue;
        }
        if byte == b'}' {
            if stop_at_template_brace && brace_depth == 0 {
                *position += 1;
                return;
            }
            brace_depth = brace_depth.saturating_sub(1);
            push_token(tokens, TokenKind::Other, *position, *position + 1);
            *position += 1;
            continue;
        }

        let Some(character) = source[*position..].chars().next() else {
            break;
        };
        if is_identifier_start(character) {
            let start = *position;
            *position += character.len_utf8();
            while *position < bytes.len() {
                let Some(character) = source[*position..].chars().next() else {
                    break;
                };
                if !is_identifier_continue(character) {
                    break;
                }
                *position += character.len_utf8();
            }
            push_token(
                tokens,
                TokenKind::Identifier(source[start..*position].to_owned()),
                start,
                *position,
            );
            continue;
        }

        let kind = match byte {
            b'.' => TokenKind::Dot,
            b'?' => TokenKind::Question,
            b'[' => TokenKind::OpenBracket,
            b']' => TokenKind::CloseBracket,
            b';' => TokenKind::Semicolon,
            _ => TokenKind::Other,
        };
        push_token(tokens, kind, *position, *position + 1);
        *position += 1;
    }
    if stop_at_template_brace {
        *uncertain_syntax = true;
    }
}

fn can_start_regex(tokens: &[Token], source: &str) -> bool {
    let Some(previous) = tokens.last() else {
        return true;
    };
    match &previous.kind {
        TokenKind::Identifier(name) => matches!(
            name.as_str(),
            "await"
                | "case"
                | "delete"
                | "do"
                | "else"
                | "in"
                | "instanceof"
                | "new"
                | "of"
                | "return"
                | "throw"
                | "typeof"
                | "void"
                | "yield"
        ),
        TokenKind::OpenBracket | TokenKind::Question | TokenKind::Semicolon => true,
        TokenKind::Other => {
            let spelling = &source[previous.start..previous.end];
            if matches!(spelling, "+" | "-")
                && tokens
                    .get(tokens.len().saturating_sub(2))
                    .is_some_and(|before| {
                        matches!(before.kind, TokenKind::Other)
                            && &source[before.start..before.end] == spelling
                    })
            {
                return false;
            }
            matches!(
                spelling,
                "(" | "{"
                    | ","
                    | ":"
                    | "="
                    | "!"
                    | "&"
                    | "|"
                    | "+"
                    | "-"
                    | "*"
                    | "%"
                    | "^"
                    | "~"
                    | "<"
                    | ">"
            )
        }
        TokenKind::StringLiteral(_)
        | TokenKind::OpaqueLiteral
        | TokenKind::Dot
        | TokenKind::CloseBracket => false,
    }
}

fn lex_regex(source: &str, position: &mut usize, tokens: &mut Vec<Token>) -> bool {
    let bytes = source.as_bytes();
    let start = *position;
    *position += 1;
    let mut in_character_class = false;
    while *position < bytes.len() {
        match bytes[*position] {
            b'\\' => *position = (*position + 2).min(bytes.len()),
            b'[' if !in_character_class => {
                in_character_class = true;
                *position += 1;
            }
            b']' if in_character_class => {
                in_character_class = false;
                *position += 1;
            }
            b'/' if !in_character_class => {
                *position += 1;
                while *position < bytes.len() {
                    let Some(flag) = source[*position..].chars().next() else {
                        break;
                    };
                    if !is_identifier_continue(flag) {
                        break;
                    }
                    *position += flag.len_utf8();
                }
                push_token(tokens, TokenKind::OpaqueLiteral, start, *position);
                return true;
            }
            b'\n' | b'\r' => {
                *position = start + 1;
                push_token(tokens, TokenKind::Other, start, *position);
                return false;
            }
            _ => *position += 1,
        }
    }
    *position = start + 1;
    push_token(tokens, TokenKind::Other, start, *position);
    false
}

fn can_start_markup(tokens: &[Token], source: &str, position: usize) -> bool {
    let line_start = source[..position].rfind('\n').map_or(0, |index| index + 1);
    if source[line_start..position].trim().is_empty() {
        return true;
    }
    let Some(previous) = tokens.last() else {
        return true;
    };
    match &previous.kind {
        TokenKind::Identifier(name) => matches!(name.as_str(), "return" | "yield"),
        TokenKind::OpenBracket | TokenKind::Question | TokenKind::Semicolon => true,
        TokenKind::Other => {
            let spelling = &source[previous.start..previous.end];
            matches!(spelling, "(" | "{" | "," | ":" | "=" | "!" | "&" | "|")
                || (spelling == ">"
                    && tokens
                        .get(tokens.len().saturating_sub(2))
                        .is_some_and(|before| {
                            matches!(before.kind, TokenKind::Other)
                                && &source[before.start..before.end] == "="
                        }))
        }
        TokenKind::StringLiteral(_)
        | TokenKind::OpaqueLiteral
        | TokenKind::Dot
        | TokenKind::CloseBracket => false,
    }
}

fn lex_markup(
    source: &str,
    position: &mut usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) -> bool {
    if source[*position..].starts_with("<!--") {
        if let Some(end) = source[*position + 4..].find("-->") {
            *position += 4 + end + 3;
        } else {
            *position = source.len();
            *uncertain_syntax = true;
        }
        return true;
    }
    if source[*position..].starts_with("<!") {
        if let Some(end) = source[*position + 2..].find('>') {
            *position += 2 + end + 1;
        } else {
            *position = source.len();
            *uncertain_syntax = true;
        }
        return true;
    }

    let Some((name, after_name)) = markup_tag_name(source, *position) else {
        return false;
    };
    let self_closing = markup_opening_is_self_closing(source, after_name);
    if !self_closing
        && !is_void_markup_element(&name)
        && find_closing_tag(source, after_name, &name).is_none()
    {
        return false;
    }

    *position = after_name;
    if !lex_markup_opening(source, position, tokens, uncertain_syntax) {
        return true;
    }
    if self_closing || is_void_markup_element(&name) {
        return true;
    }

    if name.eq_ignore_ascii_case("script") || name.eq_ignore_ascii_case("style") {
        let Some(close) = find_closing_tag(source, *position, &name) else {
            *position = source.len();
            *uncertain_syntax = true;
            return true;
        };
        if name.eq_ignore_ascii_case("script") {
            lex_code_fragment(source, *position, close, tokens, uncertain_syntax);
        }
        *position = close;
        consume_closing_tag(source, position);
        return true;
    }

    while *position < source.len() {
        if closing_tag_matches(source, *position, &name) {
            consume_closing_tag(source, position);
            return true;
        }
        if source[*position..].starts_with("<!--")
            && lex_markup(source, position, tokens, uncertain_syntax)
        {
            continue;
        }
        if source.as_bytes()[*position] == b'<'
            && lex_markup(source, position, tokens, uncertain_syntax)
        {
            continue;
        }
        if source.as_bytes()[*position] == b'{' {
            if lex_svelte_block_close(source, position) {
                continue;
            }
            *position += 1;
            lex_code(source, position, true, tokens, uncertain_syntax);
            continue;
        }
        let Some(character) = source[*position..].chars().next() else {
            break;
        };
        *position += character.len_utf8();
    }
    *uncertain_syntax = true;
    true
}

fn markup_tag_name(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'<') || bytes.get(start + 1) == Some(&b'/') {
        return None;
    }
    if bytes.get(start + 1) == Some(&b'>') {
        return Some((String::new(), start + 1));
    }
    let first = source[start + 1..].chars().next()?;
    if !is_identifier_start(first) || first == '$' {
        return None;
    }
    let mut end = start + 1 + first.len_utf8();
    while end < source.len() {
        let character = source[end..].chars().next()?;
        if !(is_identifier_continue(character) || matches!(character, '-' | ':' | '.')) {
            break;
        }
        end += character.len_utf8();
    }
    let boundary = source.as_bytes().get(end).copied()?;
    if !boundary.is_ascii_whitespace() && !matches!(boundary, b'/' | b'>' | b'{') {
        return None;
    }
    Some((source[start + 1..end].to_owned(), end))
}

fn markup_opening_is_self_closing(source: &str, mut position: usize) -> bool {
    let bytes = source.as_bytes();
    let mut quote = None;
    let mut brace_depth = 0_usize;
    while position < bytes.len() {
        let byte = bytes[position];
        if let Some(active_quote) = quote {
            if byte == b'\\' {
                position = (position + 2).min(bytes.len());
            } else {
                position += 1;
                if byte == active_quote {
                    quote = None;
                }
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => {
                quote = Some(byte);
                position += 1;
            }
            b'{' => {
                brace_depth += 1;
                position += 1;
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                position += 1;
            }
            b'>' if brace_depth == 0 => {
                return source[..position].trim_end().ends_with('/');
            }
            _ => position += 1,
        }
    }
    false
}

fn lex_markup_opening(
    source: &str,
    position: &mut usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) -> bool {
    let bytes = source.as_bytes();
    while *position < bytes.len() {
        match bytes[*position] {
            b'\'' | b'"' => {
                let quote = bytes[*position];
                let expression = quoted_markup_attribute_is_expression(source, *position);
                *position += 1;
                let content_start = *position;
                while *position < bytes.len() && bytes[*position] != quote {
                    *position += 1;
                }
                if *position == bytes.len() {
                    *uncertain_syntax = true;
                    return false;
                }
                if expression {
                    lex_code_fragment(source, content_start, *position, tokens, uncertain_syntax);
                }
                *position += 1;
            }
            b'{' => {
                *position += 1;
                lex_code(source, position, true, tokens, uncertain_syntax);
            }
            b'>' => {
                *position += 1;
                return true;
            }
            _ => *position += 1,
        }
    }
    *uncertain_syntax = true;
    false
}

fn quoted_markup_attribute_is_expression(source: &str, quote: usize) -> bool {
    let bytes = source.as_bytes();
    let mut position = quote;
    while position > 0 && bytes[position - 1].is_ascii_whitespace() {
        position -= 1;
    }
    if position == 0 || bytes[position - 1] != b'=' {
        return false;
    }
    position -= 1;
    while position > 0 && bytes[position - 1].is_ascii_whitespace() {
        position -= 1;
    }
    let end = position;
    while position > 0
        && !bytes[position - 1].is_ascii_whitespace()
        && !matches!(bytes[position - 1], b'<' | b'>')
    {
        position -= 1;
    }
    let name = &source[position..end];
    name.starts_with(['@', ':', '#']) || name.starts_with("v-")
}

fn lex_svelte_block_close(source: &str, position: &mut usize) -> bool {
    let Some(rest) = source.get(*position..) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix("{/") else {
        return false;
    };
    let name_end = rest
        .find(|character: char| !is_identifier_continue(character))
        .unwrap_or(rest.len());
    let name = &rest[..name_end];
    if !matches!(name, "await" | "each" | "if" | "key" | "snippet") {
        return false;
    }
    let Some(close) = rest[name_end..].find('}') else {
        return false;
    };
    if !rest[name_end..name_end + close].trim().is_empty() {
        return false;
    }
    *position += 2 + name_end + close + 1;
    true
}

fn lex_code_fragment(
    source: &str,
    start: usize,
    end: usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) {
    let LexedSource {
        tokens: mut fragment,
        uncertain_syntax: fragment_uncertain,
    } = lex(&source[start..end]);
    for token in &mut fragment {
        token.start += start;
        token.end += start;
    }
    tokens.extend(fragment);
    *uncertain_syntax |= fragment_uncertain;
}

fn find_closing_tag(source: &str, mut position: usize, name: &str) -> Option<usize> {
    while let Some(relative) = source[position..].find("</") {
        let candidate = position + relative;
        if closing_tag_matches(source, candidate, name) {
            return Some(candidate);
        }
        position = candidate + 2;
    }
    None
}

fn closing_tag_matches(source: &str, position: usize, name: &str) -> bool {
    let Some(rest) = source.get(position + 2..) else {
        return false;
    };
    if name.is_empty() {
        return rest.starts_with('>');
    }
    let Some(candidate) = rest.get(..name.len()) else {
        return false;
    };
    candidate.eq_ignore_ascii_case(name)
        && rest
            .as_bytes()
            .get(name.len())
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'>')
}

fn consume_closing_tag(source: &str, position: &mut usize) {
    if let Some(end) = source[*position..].find('>') {
        *position += end + 1;
    } else {
        *position = source.len();
    }
}

fn is_void_markup_element(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn lex_string(source: &str, position: &mut usize, tokens: &mut Vec<Token>) -> bool {
    let bytes = source.as_bytes();
    let start = *position;
    let quote = bytes[start];
    *position += 1;
    let content_start = *position;
    let mut simple = true;
    while *position < bytes.len() {
        match bytes[*position] {
            b'\\' => {
                simple = false;
                *position = (*position + 2).min(bytes.len());
            }
            byte if byte == quote => {
                let value = simple.then(|| source[content_start..*position].to_owned());
                *position += 1;
                push_token(tokens, TokenKind::StringLiteral(value), start, *position);
                return true;
            }
            b'\n' | b'\r' => {
                push_token(tokens, TokenKind::StringLiteral(None), start, *position);
                return false;
            }
            _ => *position += 1,
        }
    }
    push_token(tokens, TokenKind::StringLiteral(None), start, *position);
    false
}

fn lex_template(
    source: &str,
    position: &mut usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) -> bool {
    let bytes = source.as_bytes();
    let start = *position;
    *position += 1;
    while *position < bytes.len() {
        match bytes[*position] {
            b'\\' => *position = (*position + 2).min(bytes.len()),
            b'`' => {
                *position += 1;
                push_token(tokens, TokenKind::OpaqueLiteral, start, *position);
                return true;
            }
            b'$' if bytes.get(*position + 1) == Some(&b'{') => {
                *position += 2;
                lex_code(source, position, true, tokens, uncertain_syntax);
            }
            _ => *position += 1,
        }
    }
    push_token(tokens, TokenKind::OpaqueLiteral, start, *position);
    false
}

fn push_token(tokens: &mut Vec<Token>, kind: TokenKind, start: usize, end: usize) {
    tokens.push(Token { kind, start, end });
}

fn is_identifier_start(character: char) -> bool {
    matches!(character, '_' | '$') || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    is_identifier_start(character) || character.is_alphanumeric()
}

struct ImportedSymbols {
    imports: BTreeSet<usize>,
    roots: BTreeMap<String, ApplicationBinding>,
    factories: BTreeMap<String, String>,
    bindings: Vec<ApplicationImportBinding>,
}

fn imported_symbols(
    tokens: &[Token],
    source: &str,
    source_id: SourceId,
    uncertain_syntax: bool,
) -> ImportedSymbols {
    let mut imported = BTreeSet::new();
    let mut bindings = Vec::new();
    let mut roots = ["l", "lgl", "messages"]
        .into_iter()
        .map(|local| {
            (
                local.to_owned(),
                ApplicationBinding {
                    local: local.to_owned(),
                    provenance: ApplicationBindingProvenance::Implicit,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut factories = [
        "configureLinguini",
        "createLinguini",
        "createLinguiniProvider",
    ]
    .into_iter()
    .map(|name| (name.to_owned(), name.to_owned()))
    .collect::<BTreeMap<_, _>>();
    let mut index = 0;
    while index < tokens.len() {
        if !matches!(&tokens[index].kind, TokenKind::Identifier(name) if name == "import") {
            index += 1;
            continue;
        }
        if !is_top_level(tokens, source, index)
            || is_member_property(tokens, index)
            || tokens.get(index + 1).is_some_and(|token| {
                matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == "("
            })
            || matches!(
                tokens.get(index + 1).map(|token| &token.kind),
                Some(TokenKind::Dot)
            )
        {
            index += 1;
            continue;
        }

        let end = import_end(tokens, source, index);
        let side_effect = matches!(
            tokens.get(index + 1).map(|token| &token.kind),
            Some(TokenKind::StringLiteral(_))
        );
        let has_from_clause = tokens[index + 1..end]
            .iter()
            .enumerate()
            .any(|(offset, token)| {
                matches!(&token.kind, TokenKind::Identifier(name) if name == "from")
                    && tokens[index + offset + 2..end]
                        .iter()
                        .any(|token| matches!(token.kind, TokenKind::StringLiteral(_)))
            });
        if !side_effect && !has_from_clause {
            index += 1;
            continue;
        }
        for imported_index in index..end {
            imported.insert(imported_index);
        }
        let module_specifier = (index + 1..end)
            .find_map(|candidate| {
                matches!(&tokens[candidate].kind, TokenKind::Identifier(name) if name == "from")
                    .then(|| tokens.get(candidate + 1))
                    .flatten()
                    .and_then(|token| match &token.kind {
                        TokenKind::StringLiteral(Some(module)) => Some(module.clone()),
                        _ => None,
                    })
            })
            .unwrap_or_default();
        bindings.extend(named_application_import_bindings(
            tokens,
            source,
            source_id,
            index,
            end,
            uncertain_syntax,
        ));
        for alias_index in index..end.saturating_sub(2) {
            let TokenKind::Identifier(imported_name) = &tokens[alias_index].kind else {
                continue;
            };
            if matches!(
                &tokens[alias_index + 1].kind,
                TokenKind::Identifier(name) if name == "as"
            ) {
                if let TokenKind::Identifier(alias) = &tokens[alias_index + 2].kind {
                    if roots.contains_key(imported_name)
                        && (!matches!(imported_name.as_str(), "l" | "messages")
                            || bindings.iter().any(|binding| {
                                binding.id.declaration_start == tokens[index].start
                                    && binding.imported == *imported_name
                                    && binding.local == *alias
                            }))
                    {
                        roots.insert(
                            alias.clone(),
                            ApplicationBinding {
                                local: alias.clone(),
                                provenance: ApplicationBindingProvenance::Imported {
                                    module_specifier: module_specifier.clone(),
                                    imported: imported_name.clone(),
                                },
                            },
                        );
                    }
                    if factories.contains_key(imported_name) {
                        factories.insert(alias.clone(), imported_name.clone());
                    }
                }
            }
        }
        // Preserve imported symbols without an explicit `as` alias too. Named
        // imports retain their local spelling, while aliases above retain the
        // canonical imported symbol.
        let mut in_named = false;
        for candidate in index + 1..end {
            match &tokens[candidate].kind {
                TokenKind::Other
                    if &source[tokens[candidate].start..tokens[candidate].end] == "{" =>
                {
                    in_named = true;
                }
                TokenKind::Other
                    if &source[tokens[candidate].start..tokens[candidate].end] == "}" =>
                {
                    in_named = false;
                }
                TokenKind::Identifier(name) if in_named => {
                    if name == "as" || name == "from" {
                        continue;
                    }
                    if matches!(
                        candidate
                            .checked_sub(1)
                            .and_then(|previous| tokens.get(previous))
                            .map(|token| &token.kind),
                        Some(TokenKind::Identifier(previous)) if previous == "as"
                    ) {
                        continue;
                    }
                    let next_is_alias = matches!(
                        tokens.get(candidate + 1).map(|token| &token.kind),
                        Some(TokenKind::Identifier(alias)) if alias == "as"
                    );
                    if next_is_alias {
                        continue;
                    }
                    if roots.contains_key(name)
                        && (!matches!(name.as_str(), "l" | "messages")
                            || bindings.iter().any(|binding| {
                                binding.id.declaration_start == tokens[index].start
                                    && binding.imported == *name
                                    && binding.local == *name
                            }))
                    {
                        roots.insert(
                            name.clone(),
                            ApplicationBinding {
                                local: name.clone(),
                                provenance: ApplicationBindingProvenance::Imported {
                                    module_specifier: module_specifier.clone(),
                                    imported: name.clone(),
                                },
                            },
                        );
                    }
                    if factories.contains_key(name) {
                        factories.insert(name.clone(), name.clone());
                    }
                }
                _ => {}
            }
        }
        index = end.max(index + 1);
    }
    ImportedSymbols {
        imports: imported,
        roots,
        factories,
        bindings,
    }
}

fn named_application_import_bindings(
    tokens: &[Token],
    source: &str,
    source_id: SourceId,
    declaration_start: usize,
    declaration_end: usize,
    uncertain_syntax: bool,
) -> Vec<ApplicationImportBinding> {
    let open = declaration_start + 1;
    if !token_is_other(tokens.get(open), source, "{") {
        return Vec::new();
    }
    let mut cursor = open + 1;
    let mut previous_comma = None;
    let mut parsed_items = Vec::<(usize, usize, Option<usize>, Option<usize>)>::new();
    let close = loop {
        if token_is_other(tokens.get(cursor), source, "}") {
            break cursor;
        }
        let Some(Token {
            kind: TokenKind::Identifier(_),
            ..
        }) = tokens.get(cursor)
        else {
            return Vec::new();
        };
        let imported_index = cursor;
        cursor += 1;
        let local_index = if matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Identifier(name)) if name == "as"
        ) {
            cursor += 1;
            let Some(Token {
                kind: TokenKind::Identifier(_),
                ..
            }) = tokens.get(cursor)
            else {
                return Vec::new();
            };
            let local = cursor;
            cursor += 1;
            local
        } else {
            imported_index
        };
        let Some(TokenKind::Identifier(local)) = tokens.get(local_index).map(|token| &token.kind)
        else {
            return Vec::new();
        };
        if !is_strict_module_binding_identifier(local) {
            return Vec::new();
        }
        if token_is_other(tokens.get(cursor), source, "}") {
            parsed_items.push((imported_index, local_index, previous_comma, None));
            break cursor;
        }
        if !token_is_other(tokens.get(cursor), source, ",") {
            return Vec::new();
        }
        let comma = cursor;
        parsed_items.push((imported_index, local_index, previous_comma, Some(comma)));
        previous_comma = Some(comma);
        cursor += 1;
        if token_is_other(tokens.get(cursor), source, "}") {
            break cursor;
        }
    };
    cursor = close + 1;
    if !matches!(
        tokens.get(cursor).map(|token| &token.kind),
        Some(TokenKind::Identifier(name)) if name == "from"
    ) {
        return Vec::new();
    }
    cursor += 1;
    let Some(Token {
        kind: TokenKind::StringLiteral(Some(module_specifier)),
        ..
    }) = tokens.get(cursor)
    else {
        return Vec::new();
    };
    if module_specifier.is_empty() {
        return Vec::new();
    }
    let module_index = cursor;
    cursor += 1;
    if cursor < declaration_end
        && (!matches!(
            tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) || cursor + 1 != declaration_end)
    {
        return Vec::new();
    }
    let module_token = &tokens[module_index];
    let declaration_token_end = tokens
        .get(declaration_end.saturating_sub(1))
        .map_or(module_token.end, |token| token.end);
    let declaration_span = Span::in_source(
        source_id,
        tokens[declaration_start].start,
        declaration_token_end,
    );
    let declaration_source = &source[declaration_span.start..declaration_span.end];
    let following_token_start = tokens
        .get(declaration_end)
        .map_or(source.len(), |token| token.start);
    let raw_module_tail = &source[module_token.end..following_token_start];
    let structurally_safe = !uncertain_syntax
        && !declaration_source.contains("//")
        && !declaration_source.contains("/*")
        && !raw_module_tail.contains("//")
        && !raw_module_tail.contains("/*");

    let item_count = parsed_items.len();
    let sole_named_declaration = open == declaration_start + 1 && item_count == 1;
    let mut bindings = Vec::new();
    for (imported_index, local_index, previous_comma, next_comma) in parsed_items {
        let TokenKind::Identifier(imported) = &tokens[imported_index].kind else {
            continue;
        };
        if !matches!(imported.as_str(), "l" | "messages") {
            continue;
        }
        let TokenKind::Identifier(local) = &tokens[local_index].kind else {
            continue;
        };
        let item_span = Span::in_source(
            source_id,
            tokens[imported_index].start,
            tokens[local_index].end,
        );
        let removal_span = if sole_named_declaration {
            declaration_span
        } else if let Some(comma) = next_comma {
            Span::in_source(source_id, item_span.start, tokens[comma].end)
        } else if let Some(comma) = previous_comma {
            Span::in_source(source_id, tokens[comma].start, item_span.end)
        } else {
            continue;
        };
        let id = ApplicationImportBindingId {
            source: source_id,
            declaration_start: declaration_span.start,
            item_start: item_span.start,
        };
        bindings.push(ApplicationImportBinding {
            id,
            module_specifier: module_specifier.clone(),
            imported: imported.clone(),
            local: local.clone(),
            declaration_span,
            item_span,
            removal_span,
            module_specifier_span: Span::in_source(source_id, module_token.start, module_token.end),
            imported_span: Span::in_source(
                source_id,
                tokens[imported_index].start,
                tokens[imported_index].end,
            ),
            local_span: Span::in_source(
                source_id,
                tokens[local_index].start,
                tokens[local_index].end,
            ),
            exact_uses_only: structurally_safe,
        });
    }
    bindings
}

fn token_is_other(token: Option<&Token>, source: &str, expected: &str) -> bool {
    token.is_some_and(|token| {
        matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == expected
    })
}

fn is_strict_module_binding_identifier(identifier: &str) -> bool {
    !matches!(
        identifier,
        "arguments"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "eval"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

fn is_top_level(tokens: &[Token], source: &str, end: usize) -> bool {
    let mut parentheses = 0_i32;
    let mut braces = 0_i32;
    let mut brackets = 0_i32;
    for token in &tokens[..end] {
        match &token.kind {
            TokenKind::OpenBracket => brackets += 1,
            TokenKind::CloseBracket => brackets -= 1,
            TokenKind::Other => match &source[token.start..token.end] {
                "(" => parentheses += 1,
                ")" => parentheses -= 1,
                "{" => braces += 1,
                "}" => braces -= 1,
                _ => {}
            },
            _ => {}
        }
    }
    parentheses == 0 && braces == 0 && brackets == 0
}

fn call_end(tokens: &[Token], source: &str, start: usize) -> Option<usize> {
    let first = tokens.get(start)?;
    if !matches!(first.kind, TokenKind::Other) || &source[first.start..first.end] != "(" {
        return None;
    }
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if !matches!(token.kind, TokenKind::Other) {
            continue;
        }
        match &source[token.start..token.end] {
            "(" => depth += 1,
            ")" => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn declaration_binding(tokens: &[Token], source: &str, factory: usize) -> Option<(String, bool)> {
    let equals = factory.checked_sub(1)?;
    let equals_token = tokens.get(equals)?;
    if !matches!(equals_token.kind, TokenKind::Other)
        || &source[equals_token.start..equals_token.end] != "="
    {
        return None;
    }

    let statement_start = tokens[..equals]
        .iter()
        .rposition(|token| matches!(token.kind, TokenKind::Semicolon))
        .map_or(0, |index| index + 1);
    for declaration in (statement_start..equals).rev() {
        if !matches!(
            &tokens[declaration].kind,
            TokenKind::Identifier(name) if matches!(name.as_str(), "const" | "let" | "var")
        ) {
            continue;
        }
        if tokens[declaration + 1..equals].iter().any(|token| {
            matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == "="
        }) {
            return None;
        }
        let TokenKind::Identifier(binding) = &tokens.get(declaration + 1)?.kind else {
            return None;
        };
        let exported = tokens[statement_start..declaration]
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Identifier(name) if name == "export"));
        return Some((binding.clone(), exported));
    }
    None
}

fn is_binding_declaration(tokens: &[Token], source: &str, index: usize) -> bool {
    if matches!(
        index.checked_sub(1).and_then(|index| tokens.get(index)),
        Some(Token {
            kind: TokenKind::Identifier(name),
            ..
        }) if matches!(name.as_str(), "const" | "let" | "var")
    ) {
        return true;
    }
    tokens.get(index + 1).is_some_and(|token| {
        matches!(token.kind, TokenKind::Other)
            && matches!(&source[token.start..token.end], "=" | ":")
    })
}

fn is_invocation(tokens: &[Token], source: &str, index: usize) -> bool {
    let Some(token) = tokens.get(index) else {
        return false;
    };
    if matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == "(" {
        return true;
    }
    matches!(
        tokens.get(index..index + 2),
        Some([
            Token {
                kind: TokenKind::Question,
                ..
            },
            Token {
                kind: TokenKind::Other,
                start,
                end,
            }
        ]) if &source[*start..*end] == "("
    )
}

fn import_end(tokens: &[Token], source: &str, start: usize) -> usize {
    let mut saw_module = false;
    for index in start + 1..tokens.len() {
        if matches!(tokens[index].kind, TokenKind::StringLiteral(_)) {
            saw_module = true;
        }
        if matches!(tokens[index].kind, TokenKind::Semicolon) {
            return index + 1;
        }
        if saw_module
            && tokens
                .get(index + 1)
                .is_some_and(|next| source[tokens[index].end..next.start].contains('\n'))
        {
            if tokens.get(index + 1).is_some_and(|next| {
                matches!(
                    &next.kind,
                    TokenKind::Identifier(name) if matches!(name.as_str(), "assert" | "with")
                )
            }) {
                continue;
            }
            return index + 1;
        }
    }
    tokens.len()
}

fn analyzer_optional_invocation(source: &str, span_end: usize) -> bool {
    let Some(after_span) = source.get(span_end..) else {
        return false;
    };
    let Some(after_chain) = strip_application_trivia(after_span).strip_prefix("?.") else {
        return false;
    };
    strip_application_trivia(after_chain).starts_with('(')
}

fn strip_application_trivia(mut source: &str) -> &str {
    loop {
        let trimmed = source.trim_start();
        if let Some(comment) = trimmed.strip_prefix("//") {
            source = comment
                .find(['\r', '\n'])
                .map_or("", |newline| &comment[newline..]);
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix("/*") {
            let Some(end) = comment.find("*/") else {
                return trimmed;
            };
            source = &comment[end + 2..];
            continue;
        }
        return trimmed;
    }
}

fn is_member_property(tokens: &[Token], index: usize) -> bool {
    matches!(
        index.checked_sub(1).and_then(|index| tokens.get(index)),
        Some(Token {
            kind: TokenKind::Dot,
            ..
        })
    ) || matches!(
        index.checked_sub(2).map(|index| &tokens[index..index + 2]),
        Some([
            Token {
                kind: TokenKind::Question,
                ..
            },
            Token {
                kind: TokenKind::Dot,
                ..
            }
        ])
    )
}

fn member_path(tokens: &[Token], mut index: usize) -> (Vec<UsageSegment>, bool, usize) {
    let mut segments = Vec::new();
    loop {
        let member_start = if matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Dot)
        ) {
            Some(index + 1)
        } else if matches!(
            tokens.get(index..index + 2),
            Some([
                Token {
                    kind: TokenKind::Question,
                    ..
                },
                Token {
                    kind: TokenKind::Dot,
                    ..
                }
            ])
        ) {
            Some(index + 2)
        } else {
            None
        };
        if let Some(member) = member_start {
            if matches!(
                tokens.get(member).map(|token| &token.kind),
                Some(TokenKind::OpenBracket)
            ) {
                index = member;
            } else if let Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) = tokens.get(member)
            {
                segments.push(UsageSegment::new(name.clone()));
                index = member + 1;
                continue;
            } else {
                return (segments, false, index);
            }
        }

        if !matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::OpenBracket)
        ) {
            return (segments, false, index);
        }
        match tokens.get(index + 1).map(|token| &token.kind) {
            Some(TokenKind::StringLiteral(Some(name)))
                if matches!(
                    tokens.get(index + 2).map(|token| &token.kind),
                    Some(TokenKind::CloseBracket)
                ) =>
            {
                segments.push(UsageSegment::new(name.clone()));
                index += 3;
            }
            _ => return (segments, true, index + 1),
        }
    }
}

fn decode_generated_identifier(name: &str) -> Option<String> {
    let encoded = name.strip_prefix("__lgl_name_")?;
    if encoded.len() % 2 != 0 {
        return None;
    }
    let bytes = encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}
