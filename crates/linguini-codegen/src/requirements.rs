//! IR analysis for demand-selected formatter, plural, and inline-function support.

use std::collections::BTreeMap;

use linguini_ir::{
    is_plural_intrinsic, IrExpression, IrExpressionKind, IrFormEntry, IrFormatter, IrFormatterKind,
    IrFunctionBranch, IrFunctionBranchValue, IrFunctionParameter, IrInlineFunctionInput, IrModule,
    IrText, IrTextPart, IrValue,
};

/// Semantic capabilities required by a schema and its effective locale implementation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FormatterRequirements {
    /// Decimal-number formatting is used.
    pub number: bool,
    /// Currency formatting is used.
    pub currency: bool,
    /// Date formatting is used.
    pub date: bool,
}

impl FormatterRequirements {
    /// Whether any formatter is required.
    pub fn any(self) -> bool {
        self.number || self.currency || self.date
    }

    /// Whether decimal symbols and patterns are required for number or currency formatting.
    pub fn needs_number_data(self) -> bool {
        self.number || self.currency
    }

    fn record(&mut self, formatter: &IrFormatter) {
        match formatter.kind {
            IrFormatterKind::Number => self.number = true,
            IrFormatterKind::Currency => self.currency = true,
            IrFormatterKind::Date => self.date = true,
            IrFormatterKind::Unknown(_) => {}
        }
    }
}

/// Whether any expression in this module contains an inline function.
pub fn module_uses_inline_functions(module: &IrModule) -> bool {
    module
        .variables()
        .iter()
        .any(|item| text_uses_inline(&item.value))
        || module
            .messages()
            .iter()
            .filter_map(|item| item.body.as_ref())
            .any(text_uses_inline)
        || module.forms().iter().any(|form| {
            form.variants
                .iter()
                .any(|variant| variant.entries.iter().any(form_entry_uses_inline))
        })
        || module
            .functions()
            .iter()
            .any(|function| function.branches.iter().any(function_branch_uses_inline))
}

/// Collects explicit formatters and automatic parameter formatters, following type aliases.
pub fn formatter_requirements(schema: &IrModule, locale: &IrModule) -> FormatterRequirements {
    let mut requirements = FormatterRequirements::default();
    for request in crate::formatter_requests(schema, locale) {
        requirements.record(&request.formatter);
    }
    requirements
}

/// Whether emitting this schema/locale pair requires locale plural normalization.
pub fn plural_required(schema: &IrModule, locale: &IrModule) -> bool {
    locale
        .variables()
        .iter()
        .any(|variable| text_requires_plural(&variable.value, &BTreeMap::new()))
        || locale.messages().iter().any(|message| {
            let context = schema
                .messages()
                .iter()
                .find(|signature| signature.name == message.name)
                .map(|signature| {
                    signature
                        .parameters
                        .iter()
                        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                        .collect::<BTreeMap<_, _>>()
                })
                .unwrap_or_default();
            message
                .body
                .as_ref()
                .is_some_and(|body| text_requires_plural(body, &context))
        })
        || locale.forms().iter().any(|form| {
            form.variants
                .iter()
                .any(|variant| variant.entries.iter().any(form_entry_requires_plural))
        })
        || locale.functions().iter().any(|function| {
            function
                .parameters
                .iter()
                .any(|parameter| parameter.name.is_none() && parameter.ty == "Plural")
                || {
                    let context = parameter_context(&function.parameters);
                    function
                        .branches
                        .iter()
                        .any(|branch| function_branch_requires_plural(branch, &context))
                }
        })
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

fn form_entry_requires_plural(entry: &IrFormEntry) -> bool {
    match entry {
        IrFormEntry::Attribute {
            parameters, value, ..
        } => {
            parameters.first().is_some_and(|parameter| {
                matches!(value, IrValue::Map(_)) && parameter.ty == "Plural"
            }) || value_requires_plural(value, &parameter_context(parameters))
        }
        // Form-level branches are always dispatched through the locale plural function.
        IrFormEntry::Branch(_) => true,
    }
}

fn value_requires_plural(value: &IrValue, context: &BTreeMap<String, String>) -> bool {
    match value {
        IrValue::Text(text) => text_requires_plural(text, context),
        IrValue::Map(branches) => branches
            .iter()
            .any(|branch| text_requires_plural(&branch.value, context)),
        IrValue::Object(entries) => entries.iter().any(form_entry_requires_plural),
    }
}

fn function_branch_requires_plural(
    branch: &IrFunctionBranch,
    context: &BTreeMap<String, String>,
) -> bool {
    match &branch.value {
        IrFunctionBranchValue::Text(text) => text_requires_plural(text, context),
        IrFunctionBranchValue::Dispatch(branches) => branches
            .iter()
            .any(|branch| function_branch_requires_plural(branch, context)),
    }
}

fn text_requires_plural(text: &IrText, context: &BTreeMap<String, String>) -> bool {
    text.parts.iter().any(|part| match part {
        IrTextPart::Text(_) => false,
        IrTextPart::Placeholder(expression) => expression_requires_plural(expression, context),
    })
}

fn expression_requires_plural(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
) -> bool {
    if expression
        .arguments
        .iter()
        .any(|argument| expression_requires_plural(argument, context))
    {
        return true;
    }
    if expression.kind == IrExpressionKind::Call
        && expression.path.len() == 1
        && is_plural_intrinsic(&expression.path[0])
    {
        return true;
    }
    let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind else {
        return false;
    };
    let mut branch_context = context.clone();
    for input in inputs {
        let value = match input {
            IrInlineFunctionInput::Binding { name, value, .. } => {
                if let Some(ty) = expression_type(value, context) {
                    branch_context.insert(name.clone(), ty);
                }
                value
            }
            IrInlineFunctionInput::Selector { value, .. } => {
                if expression_type(value, context).as_deref() == Some("Plural")
                    && !is_plural_intrinsic_call(value)
                {
                    return true;
                }
                value
            }
        };
        if expression_requires_plural(value, context) {
            return true;
        }
    }
    branches
        .iter()
        .any(|branch| function_branch_requires_plural(branch, &branch_context))
}

fn is_plural_intrinsic_call(expression: &IrExpression) -> bool {
    expression.kind == IrExpressionKind::Call
        && expression.path.len() == 1
        && is_plural_intrinsic(&expression.path[0])
}

fn expression_type(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
) -> Option<String> {
    match &expression.kind {
        IrExpressionKind::InlineFunction { .. } => Some("String".to_owned()),
        IrExpressionKind::Call if is_plural_intrinsic_call(expression) => Some("Plural".to_owned()),
        IrExpressionKind::Call => Some("String".to_owned()),
        IrExpressionKind::Reference if expression.path.len() == 1 => {
            context.get(&expression.path[0]).cloned()
        }
        IrExpressionKind::Reference => None,
    }
}

fn form_entry_uses_inline(entry: &IrFormEntry) -> bool {
    match entry {
        IrFormEntry::Attribute { value, .. } => value_uses_inline(value),
        IrFormEntry::Branch(branch) => text_uses_inline(&branch.value),
    }
}

fn value_uses_inline(value: &IrValue) -> bool {
    match value {
        IrValue::Text(text) => text_uses_inline(text),
        IrValue::Map(branches) => branches
            .iter()
            .any(|branch| text_uses_inline(&branch.value)),
        IrValue::Object(entries) => entries.iter().any(form_entry_uses_inline),
    }
}

fn function_branch_uses_inline(branch: &IrFunctionBranch) -> bool {
    match &branch.value {
        IrFunctionBranchValue::Text(text) => text_uses_inline(text),
        IrFunctionBranchValue::Dispatch(branches) => {
            branches.iter().any(function_branch_uses_inline)
        }
    }
}

fn text_uses_inline(text: &IrText) -> bool {
    text.parts.iter().any(|part| match part {
        IrTextPart::Text(_) => false,
        IrTextPart::Placeholder(expression) => expression_uses_inline(expression),
    })
}

fn expression_uses_inline(expression: &IrExpression) -> bool {
    matches!(&expression.kind, IrExpressionKind::InlineFunction { .. })
        || expression.arguments.iter().any(expression_uses_inline)
        || match &expression.kind {
            IrExpressionKind::InlineFunction { inputs, branches } => {
                inputs.iter().any(|input| {
                    let value = match input {
                        IrInlineFunctionInput::Binding { value, .. }
                        | IrInlineFunctionInput::Selector { value, .. } => value,
                    };
                    expression_uses_inline(value)
                }) || branches.iter().any(function_branch_uses_inline)
            }
            IrExpressionKind::Reference | IrExpressionKind::Call => false,
        }
}
