use std::collections::BTreeSet;

use linguini_core::TypeKind;
use linguini_ir::{
    IrExpression, IrExpressionKind, IrFormEntry, IrFormatter, IrFormatterKind, IrFunctionBranch,
    IrFunctionBranchValue, IrInlineFunctionInput, IrMessage, IrModule, IrText, IrTextPart, IrValue,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FormatterRequirements {
    pub number: bool,
    pub currency: bool,
    pub date: bool,
}

impl FormatterRequirements {
    pub fn any(self) -> bool {
        self.number || self.currency || self.date
    }

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

pub fn module_uses_inline_functions(module: &IrModule) -> bool {
    module
        .variables
        .iter()
        .any(|item| text_uses_inline(&item.value))
        || module
            .messages
            .iter()
            .filter_map(|item| item.body.as_ref())
            .any(text_uses_inline)
        || module.forms.iter().any(|form| {
            form.variants
                .iter()
                .any(|variant| variant.entries.iter().any(form_entry_uses_inline))
        })
        || module
            .functions
            .iter()
            .any(|function| function.branches.iter().any(function_branch_uses_inline))
}

pub fn formatter_requirements(schema: &IrModule, locale: &IrModule) -> FormatterRequirements {
    let mut requirements = FormatterRequirements::default();
    collect_module_formatters(schema, &mut requirements);
    collect_module_formatters(locale, &mut requirements);
    collect_automatic_formatters(schema, &mut requirements);
    requirements
}

fn collect_module_formatters(module: &IrModule, requirements: &mut FormatterRequirements) {
    for alias in &module.type_aliases {
        collect_formatters(&alias.formatters, requirements);
    }
    for variable in &module.variables {
        collect_text_formatters(&variable.value, requirements);
    }
    for message in &module.messages {
        collect_message_formatters(message, requirements);
    }
    for form in &module.forms {
        for variant in &form.variants {
            for entry in &variant.entries {
                collect_form_entry_formatters(entry, requirements);
            }
        }
    }
    for function in &module.functions {
        for branch in &function.branches {
            collect_function_branch_formatters(branch, requirements);
        }
    }
}

fn collect_automatic_formatters(schema: &IrModule, requirements: &mut FormatterRequirements) {
    for parameter in schema
        .messages
        .iter()
        .flat_map(|message| &message.parameters)
    {
        let mut current = parameter.ty.as_str();
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(current) {
                break;
            }
            if let Some(alias) = schema
                .type_aliases
                .iter()
                .find(|alias| alias.name == current)
            {
                if !alias.formatters.is_empty() {
                    collect_formatters(&alias.formatters, requirements);
                    break;
                }
                current = &alias.target;
                continue;
            }
            if let Some(kind) = TypeKind::from_name(current).and_then(TypeKind::default_formatter) {
                match kind {
                    linguini_core::FormatterKind::Number => requirements.number = true,
                    linguini_core::FormatterKind::Currency => requirements.currency = true,
                    linguini_core::FormatterKind::Date => requirements.date = true,
                    linguini_core::FormatterKind::Unknown(_) => {}
                }
            }
            break;
        }
    }
}

fn collect_formatters(formatters: &[IrFormatter], requirements: &mut FormatterRequirements) {
    for formatter in formatters {
        requirements.record(formatter);
    }
}

fn collect_message_formatters(message: &IrMessage, requirements: &mut FormatterRequirements) {
    if let Some(body) = &message.body {
        collect_text_formatters(body, requirements);
    }
}

fn collect_form_entry_formatters(entry: &IrFormEntry, requirements: &mut FormatterRequirements) {
    match entry {
        IrFormEntry::Attribute { value, .. } => collect_value_formatters(value, requirements),
        IrFormEntry::Branch(branch) => collect_text_formatters(&branch.value, requirements),
    }
}

fn collect_value_formatters(value: &IrValue, requirements: &mut FormatterRequirements) {
    match value {
        IrValue::Text(text) => collect_text_formatters(text, requirements),
        IrValue::Map(branches) => {
            for branch in branches {
                collect_text_formatters(&branch.value, requirements);
            }
        }
        IrValue::Object(entries) => {
            for entry in entries {
                collect_form_entry_formatters(entry, requirements);
            }
        }
    }
}

fn collect_function_branch_formatters(
    branch: &IrFunctionBranch,
    requirements: &mut FormatterRequirements,
) {
    match &branch.value {
        IrFunctionBranchValue::Text(text) => collect_text_formatters(text, requirements),
        IrFunctionBranchValue::Dispatch(branches) => {
            for branch in branches {
                collect_function_branch_formatters(branch, requirements);
            }
        }
    }
}

fn collect_text_formatters(text: &IrText, requirements: &mut FormatterRequirements) {
    for part in &text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            collect_expression_formatters(expression, requirements);
        }
    }
}

fn collect_expression_formatters(
    expression: &IrExpression,
    requirements: &mut FormatterRequirements,
) {
    collect_formatters(&expression.formatters, requirements);
    for argument in &expression.arguments {
        collect_expression_formatters(argument, requirements);
    }
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        for input in inputs {
            collect_inline_input_formatters(input, requirements);
        }
        for branch in branches {
            collect_function_branch_formatters(branch, requirements);
        }
    }
}

fn collect_inline_input_formatters(
    input: &IrInlineFunctionInput,
    requirements: &mut FormatterRequirements,
) {
    let value = match input {
        IrInlineFunctionInput::Binding { value, .. }
        | IrInlineFunctionInput::Selector { value, .. } => value,
    };
    collect_expression_formatters(value, requirements);
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
