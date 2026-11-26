//! Shared source provenance for dependency slices and semantic artifacts.

use crate::SemanticSymbolKind;
use linguini_ir::{
    IrExpression, IrExpressionKind, IrFormEntry, IrFunctionBranch, IrFunctionBranchValue,
    IrInlineFunctionInput, IrModule, IrText, IrTextPart, IrValue,
};
use linguini_syntax::{SourceId, Span};
use std::collections::BTreeSet;

/// Source span of the effective declaration, after whole-declaration fallback composition.
pub fn semantic_symbol_span(
    locale: &IrModule,
    kind: SemanticSymbolKind,
    name: &str,
) -> Option<Span> {
    locale
        .origins()
        .iter()
        .rev()
        .find(|origin| origin.kind == kind.ir_kind() && origin.name == name)
        .map(|origin| origin.span)
}

/// Sorted, deduplicated declaration and nested-expression source identities for one symbol.
/// References contribute their own spans; dependency declarations are collected separately.
pub fn semantic_symbol_source_ids(
    locale: &IrModule,
    kind: SemanticSymbolKind,
    name: &str,
) -> Vec<SourceId> {
    let mut sources = BTreeSet::new();
    if let Some(span) = semantic_symbol_span(locale, kind, name) {
        sources.insert(span.source);
    }
    match kind {
        SemanticSymbolKind::LocaleEnum => {}
        SemanticSymbolKind::LocaleVariable => {
            if let Some(item) = locale.variables().iter().find(|item| item.name == name) {
                collect_text_source_ids(&item.value, &mut sources);
            }
        }
        SemanticSymbolKind::LocaleForm => {
            if let Some(item) = locale.forms().iter().find(|item| item.name == name) {
                for variant in &item.variants {
                    collect_form_source_ids(&variant.entries, &mut sources);
                }
            }
        }
        SemanticSymbolKind::LocaleFunction => {
            if let Some(item) = locale.functions().iter().find(|item| item.name == name) {
                for branch in &item.branches {
                    collect_function_source_ids(branch, &mut sources);
                }
            }
        }
    }
    sources.into_iter().collect()
}

pub(crate) fn collect_text_source_ids(text: &IrText, output: &mut BTreeSet<SourceId>) {
    output.insert(text.span.source);
    for part in &text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            collect_expression_source_ids(expression, output);
        }
    }
}

fn collect_expression_source_ids(expression: &IrExpression, output: &mut BTreeSet<SourceId>) {
    output.insert(expression.span.source);
    for argument in &expression.arguments {
        collect_expression_source_ids(argument, output);
    }
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        for input in inputs {
            let (value, span) = match input {
                IrInlineFunctionInput::Binding { value, span, .. }
                | IrInlineFunctionInput::Selector { value, span } => (value, span),
            };
            output.insert(span.source);
            collect_expression_source_ids(value, output);
        }
        for branch in branches {
            collect_function_source_ids(branch, output);
        }
    }
}

pub(crate) fn collect_function_source_ids(
    branch: &IrFunctionBranch,
    output: &mut BTreeSet<SourceId>,
) {
    output.insert(branch.span.source);
    match &branch.value {
        IrFunctionBranchValue::Text(text) => collect_text_source_ids(text, output),
        IrFunctionBranchValue::Dispatch(branches) => {
            for branch in branches {
                collect_function_source_ids(branch, output);
            }
        }
    }
}

pub(crate) fn collect_form_source_ids(entries: &[IrFormEntry], output: &mut BTreeSet<SourceId>) {
    for entry in entries {
        match entry {
            IrFormEntry::Attribute { value, .. } => collect_value_source_ids(value, output),
            IrFormEntry::Branch(branch) => {
                output.insert(branch.span.source);
                collect_text_source_ids(&branch.value, output);
            }
        }
    }
}

fn collect_value_source_ids(value: &IrValue, output: &mut BTreeSet<SourceId>) {
    match value {
        IrValue::Text(text) => collect_text_source_ids(text, output),
        IrValue::Map(branches) => {
            for branch in branches {
                output.insert(branch.span.source);
                collect_text_source_ids(&branch.value, output);
            }
        }
        IrValue::Object(entries) => collect_form_source_ids(entries, output),
    }
}

#[cfg(test)]
mod tests;
