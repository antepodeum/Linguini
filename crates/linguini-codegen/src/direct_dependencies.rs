//! Direct semantic dependency edges shared by target backends.

use linguini_ir::{
    is_plural_intrinsic, IrExpression, IrExpressionKind, IrFormEntry, IrFunctionBranch,
    IrFunctionBranchValue, IrFunctionParameter, IrInlineFunctionInput, IrModule, IrSymbolKind,
    IrText, IrTextPart, IrValue,
};
use std::collections::{BTreeMap, BTreeSet};

/// Stable semantic symbol class. Ordering is part of artifact determinism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticSymbolKind {
    LocaleEnum,
    LocaleVariable,
    LocaleForm,
    LocaleFunction,
}

impl SemanticSymbolKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocaleEnum => "enum",
            Self::LocaleVariable => "variable",
            Self::LocaleForm => "form",
            Self::LocaleFunction => "function",
        }
    }

    pub fn ir_kind(self) -> IrSymbolKind {
        match self {
            Self::LocaleEnum => IrSymbolKind::Enum,
            Self::LocaleVariable => IrSymbolKind::Variable,
            Self::LocaleForm => IrSymbolKind::Form,
            Self::LocaleFunction => IrSymbolKind::Function,
        }
    }
}

/// Direct locale symbols referenced by one message, including type-only locale enums.
pub fn direct_message_dependencies(
    schema: &IrModule,
    locale: &IrModule,
    signature: &linguini_ir::IrMessage,
    implementation: &linguini_ir::IrMessage,
) -> BTreeSet<(SemanticSymbolKind, String)> {
    let mut collector = ReferenceCollector::new(schema, locale);
    let context = signature
        .parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
        .collect();
    for parameter in &signature.parameters {
        collector.record_type(&parameter.ty);
    }
    if let Some(body) = &implementation.body {
        collector.text(body, &context);
    }
    collector.dependencies
}

/// Direct symbols used by a locale variable, form, or function.
pub fn direct_symbol_dependencies(
    schema: &IrModule,
    locale: &IrModule,
    kind: SemanticSymbolKind,
    name: &str,
) -> BTreeSet<(SemanticSymbolKind, String)> {
    let mut collector = ReferenceCollector::new(schema, locale);
    match kind {
        SemanticSymbolKind::LocaleEnum => {}
        SemanticSymbolKind::LocaleVariable => {
            if let Some(item) = locale.variables().iter().find(|item| item.name == name) {
                collector.text(&item.value, &BTreeMap::new());
            }
        }
        SemanticSymbolKind::LocaleForm => {
            if let Some(item) = locale.forms().iter().find(|item| item.name == name) {
                for variant in &item.variants {
                    collector.form_entries(&variant.entries, &BTreeMap::new());
                }
            }
        }
        SemanticSymbolKind::LocaleFunction => {
            if let Some(item) = locale.functions().iter().find(|item| item.name == name) {
                for parameter in &item.parameters {
                    collector.record_type(&parameter.ty);
                }
                let context = parameter_context(&item.parameters);
                for branch in &item.branches {
                    collector.function_branch(branch, &context);
                }
            }
        }
    }
    collector.dependencies.remove(&(kind, name.to_owned()));
    collector.dependencies
}

fn parameter_context(parameters: &[IrFunctionParameter]) -> BTreeMap<String, String> {
    parameters
        .iter()
        .filter_map(|parameter| {
            parameter
                .name
                .as_ref()
                .map(|name| (name.clone(), parameter.ty.clone()))
        })
        .collect()
}

struct ReferenceCollector<'a> {
    schema: &'a IrModule,
    locale: &'a IrModule,
    dependencies: BTreeSet<(SemanticSymbolKind, String)>,
}
impl<'a> ReferenceCollector<'a> {
    fn new(schema: &'a IrModule, locale: &'a IrModule) -> Self {
        Self {
            schema,
            locale,
            dependencies: BTreeSet::new(),
        }
    }
    fn record_type(&mut self, ty: &str) {
        let resolved = self.resolve_type(ty);
        if self.locale.enums().iter().any(|item| item.name == resolved) {
            self.dependencies
                .insert((SemanticSymbolKind::LocaleEnum, resolved));
        }
    }
    fn text(&mut self, text: &IrText, context: &BTreeMap<String, String>) {
        for part in &text.parts {
            if let IrTextPart::Placeholder(expr) = part {
                self.expression(expr, context);
            }
        }
    }
    fn expression(&mut self, expr: &IrExpression, context: &BTreeMap<String, String>) {
        for arg in &expr.arguments {
            self.expression(arg, context);
        }
        if let IrExpressionKind::InlineFunction { inputs, branches } = &expr.kind {
            let mut inner = context.clone();
            for input in inputs {
                match input {
                    IrInlineFunctionInput::Binding { name, value, .. } => {
                        self.expression(value, context);
                        inner.insert(name.clone(), inferred_expression_type(value, context));
                    }
                    IrInlineFunctionInput::Selector { value, .. } => {
                        self.expression(value, context)
                    }
                }
            }
            for branch in branches {
                self.function_branch(branch, &inner);
            }
            return;
        }
        if expr.path.is_empty() {
            return;
        }
        if let Some(ty) = context.get(&expr.path[0]) {
            if expr.path.len() > 1 || expr.kind == IrExpressionKind::Call {
                let resolved = self.resolve_type(ty);
                if self.locale.forms().iter().any(|item| item.name == resolved) {
                    self.dependencies
                        .insert((SemanticSymbolKind::LocaleForm, resolved));
                }
            }
            return;
        }
        let full = expr.path.join(".");
        if self.locale.functions().iter().any(|item| item.name == full) {
            self.dependencies
                .insert((SemanticSymbolKind::LocaleFunction, full));
            return;
        }
        if self.locale.forms().iter().any(|item| item.name == full) {
            self.dependencies
                .insert((SemanticSymbolKind::LocaleForm, full));
            return;
        }
        if expr.kind == IrExpressionKind::Reference
            && expr.path.len() == 1
            && self
                .locale
                .variables()
                .iter()
                .any(|item| item.name == expr.path[0])
        {
            self.dependencies
                .insert((SemanticSymbolKind::LocaleVariable, expr.path[0].clone()));
        }
    }
    fn resolve_type(&self, ty: &str) -> String {
        crate::resolve_type_name(self.schema, ty).to_owned()
    }
    fn function_branch(&mut self, branch: &IrFunctionBranch, context: &BTreeMap<String, String>) {
        match &branch.value {
            IrFunctionBranchValue::Text(text) => self.text(text, context),
            IrFunctionBranchValue::Dispatch(items) => {
                for item in items {
                    self.function_branch(item, context);
                }
            }
        }
    }
    fn form_entries(&mut self, entries: &[IrFormEntry], inherited: &BTreeMap<String, String>) {
        for entry in entries {
            match entry {
                IrFormEntry::Attribute {
                    parameters, value, ..
                } => {
                    for parameter in parameters {
                        self.record_type(&parameter.ty);
                    }
                    let mut context = inherited.clone();
                    context.extend(parameter_context(parameters));
                    self.value(value, &context);
                }
                IrFormEntry::Branch(branch) => self.text(&branch.value, inherited),
            }
        }
    }
    fn value(&mut self, value: &IrValue, context: &BTreeMap<String, String>) {
        match value {
            IrValue::Text(text) => self.text(text, context),
            IrValue::Map(branches) => {
                for branch in branches {
                    self.text(&branch.value, context)
                }
            }
            IrValue::Object(entries) => self.form_entries(entries, context),
        }
    }
}

fn inferred_expression_type(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
) -> String {
    match &expression.kind {
        IrExpressionKind::Reference if expression.path.len() == 1 => context
            .get(&expression.path[0])
            .cloned()
            .unwrap_or_else(|| "String".to_owned()),
        IrExpressionKind::Call
            if expression.path.len() == 1 && is_plural_intrinsic(&expression.path[0]) =>
        {
            "Plural".to_owned()
        }
        _ => "String".to_owned(),
    }
}

#[cfg(test)]
mod tests;
