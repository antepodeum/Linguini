use crate::FormatError;
use linguini_syntax::{
    Expression, ExpressionKind, FormEntry, FunctionBranch, FunctionBranchValue,
    InlineFunctionInput, LocaleDeclaration, LocaleFile, LocaleValue, MapBranch, SourceId, Span,
    TextBlockMode, TextPart, TextPattern, Token, TokenKind,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
struct PatternSpan {
    span: Span,
    mode: TextBlockMode,
}

/// Semantic facts which are ambiguous in a trivia-preserving token stream.
///
/// The parser remains the authority for placeholder braces and text-block modes. The formatter
/// only uses tokens for source spelling and comments, which are intentionally absent from the AST.
#[derive(Debug, Default)]
pub(crate) struct FormatSemantics {
    patterns: Vec<PatternSpan>,
    placeholder_starts: BTreeSet<(SourceId, usize)>,
}

impl FormatSemantics {
    pub(crate) fn schema() -> Self {
        Self::default()
    }

    pub(crate) fn locale(file: &LocaleFile, source: &str) -> Result<Self, FormatError> {
        let mut semantics = Self::default();
        for declaration in &file.declarations {
            semantics.declaration(declaration, source)?;
        }
        semantics
            .patterns
            .sort_by_key(|pattern| (pattern.span.source, pattern.span.start, pattern.span.end));
        Ok(semantics)
    }

    pub(crate) fn validate_tokens(
        &self,
        source: &str,
        tokens: &[Token],
    ) -> Result<(), FormatError> {
        let mut placeholder_starts = self.placeholder_starts.clone();

        for token in tokens {
            if token.span.source != SourceId::default()
                || source.get(token.span.start..token.span.end).is_none()
            {
                return Err(FormatError::InvalidTokenSpan(token.span));
            }
            match token.kind {
                TokenKind::RawText(_) => {
                    if self.pattern_mode(token.span).is_none() {
                        return Err(FormatError::SyntaxMismatch(format!(
                            "text token at {}..{} is absent from the parsed text pattern",
                            token.span.start, token.span.end
                        )));
                    }
                }
                TokenKind::TripleQuote => {
                    if self.pattern_mode(token.span) != Some(TextBlockMode::Dedented) {
                        return Err(FormatError::SyntaxMismatch(format!(
                            "dedented text delimiter at {}..{} disagrees with the syntax tree",
                            token.span.start, token.span.end
                        )));
                    }
                }
                TokenKind::RawTripleQuote => {
                    if self.pattern_mode(token.span) != Some(TextBlockMode::Raw) {
                        return Err(FormatError::SyntaxMismatch(format!(
                            "raw text delimiter at {}..{} disagrees with the syntax tree",
                            token.span.start, token.span.end
                        )));
                    }
                }
                TokenKind::LBrace => {
                    placeholder_starts.remove(&(token.span.source, token.span.start));
                }
                _ => {}
            }
        }

        if let Some((_, start)) = placeholder_starts.into_iter().next() {
            return Err(FormatError::SyntaxMismatch(format!(
                "placeholder at byte {start} has no opening-brace token"
            )));
        }

        Ok(())
    }

    pub(crate) fn is_placeholder_open(&self, span: Span) -> bool {
        self.placeholder_starts.contains(&(span.source, span.start))
    }

    pub(crate) fn is_verbatim_text(&self, token: &Token) -> bool {
        matches!(token.kind, TokenKind::RawText(_)) && self.pattern_mode(token.span).is_some()
    }

    pub(crate) fn is_opening_text_delimiter(&self, token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::TripleQuote | TokenKind::RawTripleQuote
        ) && self
            .lookup_pattern(token.span)
            .is_some_and(|pattern| pattern.span.start == token.span.start)
    }

    fn declaration(
        &mut self,
        declaration: &LocaleDeclaration,
        source: &str,
    ) -> Result<(), FormatError> {
        match declaration {
            LocaleDeclaration::Enum(_) => {}
            LocaleDeclaration::Variable(variable) => self.pattern(&variable.value, source)?,
            LocaleDeclaration::Form(form) => {
                for variant in &form.variants {
                    self.entries(&variant.entries, source)?;
                }
            }
            LocaleDeclaration::Function(function) => {
                self.function_branches(&function.branches, source)?;
            }
            LocaleDeclaration::Message(message) => self.pattern(&message.value, source)?,
            LocaleDeclaration::Group(group) => self.group(group, source)?,
            LocaleDeclaration::Override(inner) => self.declaration(inner, source)?,
        }
        Ok(())
    }

    fn group(
        &mut self,
        group: &linguini_syntax::MessageImplementationGroup,
        source: &str,
    ) -> Result<(), FormatError> {
        for message in &group.messages {
            self.pattern(&message.value, source)?;
        }
        for nested in &group.groups {
            self.group(nested, source)?;
        }
        Ok(())
    }

    fn entries(&mut self, entries: &[FormEntry], source: &str) -> Result<(), FormatError> {
        for entry in entries {
            match entry {
                FormEntry::Attribute(attribute) => self.value(&attribute.value, source)?,
                FormEntry::Branch(branch) => self.map_branch(branch, source)?,
            }
        }
        Ok(())
    }

    fn value(&mut self, value: &LocaleValue, source: &str) -> Result<(), FormatError> {
        match value {
            LocaleValue::Text(pattern) => self.pattern(pattern, source)?,
            LocaleValue::Map(branches) => {
                for branch in branches {
                    self.map_branch(branch, source)?;
                }
            }
            LocaleValue::Object(entries) => self.entries(entries, source)?,
        }
        Ok(())
    }

    fn map_branch(&mut self, branch: &MapBranch, source: &str) -> Result<(), FormatError> {
        self.pattern(&branch.value, source)
    }

    fn function_branches(
        &mut self,
        branches: &[FunctionBranch],
        source: &str,
    ) -> Result<(), FormatError> {
        for branch in branches {
            match &branch.value {
                FunctionBranchValue::Text(pattern) => self.pattern(pattern, source)?,
                FunctionBranchValue::Dispatch(nested) => {
                    self.function_branches(nested, source)?;
                }
            }
        }
        Ok(())
    }

    fn pattern(&mut self, pattern: &TextPattern, source: &str) -> Result<(), FormatError> {
        validate_span(source, pattern.span)?;
        self.patterns.push(PatternSpan {
            span: pattern.span,
            mode: pattern.mode,
        });

        for part in &pattern.parts {
            let span = match part {
                TextPart::Text(text) => text.span,
                TextPart::Placeholder(placeholder) => {
                    self.placeholder_starts
                        .insert((placeholder.span.source, placeholder.span.start));
                    self.expression(&placeholder.expression, source)?;
                    placeholder.span
                }
            };
            validate_span(source, span)?;
            if !contains(pattern.span, span) {
                return Err(FormatError::InvalidSyntaxSpan(span));
            }
        }
        Ok(())
    }

    fn expression(&mut self, expression: &Expression, source: &str) -> Result<(), FormatError> {
        for argument in &expression.arguments {
            self.expression(argument, source)?;
        }
        if let ExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
            for input in inputs {
                let value = match input {
                    InlineFunctionInput::Binding { value, .. }
                    | InlineFunctionInput::Selector { value, .. } => value,
                };
                self.expression(value, source)?;
            }
            self.function_branches(branches, source)?;
        }
        Ok(())
    }

    fn pattern_mode(&self, span: Span) -> Option<TextBlockMode> {
        self.lookup_pattern(span).map(|pattern| pattern.mode)
    }

    fn lookup_pattern(&self, span: Span) -> Option<&PatternSpan> {
        let index = self
            .patterns
            .partition_point(|pattern| pattern.span.start <= span.start);
        self.patterns[..index]
            .iter()
            .rev()
            .find(|pattern| contains(pattern.span, span))
    }
}

fn validate_span(source: &str, span: Span) -> Result<(), FormatError> {
    if span.source != SourceId::default() || source.get(span.start..span.end).is_none() {
        return Err(FormatError::InvalidSyntaxSpan(span));
    }
    Ok(())
}

fn contains(outer: Span, inner: Span) -> bool {
    outer.source == inner.source && outer.start <= inner.start && inner.end <= outer.end
}
