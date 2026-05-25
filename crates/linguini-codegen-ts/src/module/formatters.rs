use linguini_ir::{
    IrExpression, IrFormEntry, IrFunctionBranch, IrFunctionBranchValue, IrMessage, IrModule,
    IrText, IrTextPart, IrValue,
};

pub fn module_uses_formatters(module: &IrModule) -> bool {
    module
        .type_aliases
        .iter()
        .any(|alias| !alias.formatters.is_empty())
        || module
            .variables
            .iter()
            .any(|variable| text_uses_formatters(&variable.value))
        || module.messages.iter().any(message_uses_formatters)
        || module.forms.iter().any(|form| {
            form.variants
                .iter()
                .any(|variant| variant.entries.iter().any(form_entry_uses_formatters))
        })
        || module.functions.iter().any(|function| {
            function
                .branches
                .iter()
                .any(function_branch_uses_formatters)
        })
}

fn message_uses_formatters(message: &IrMessage) -> bool {
    message.body.as_ref().is_some_and(text_uses_formatters)
}

fn form_entry_uses_formatters(entry: &IrFormEntry) -> bool {
    match entry {
        IrFormEntry::Attribute { value, .. } => value_uses_formatters(value),
        IrFormEntry::Branch(branch) => text_uses_formatters(&branch.value),
    }
}

fn value_uses_formatters(value: &IrValue) -> bool {
    match value {
        IrValue::Text(text) => text_uses_formatters(text),
        IrValue::Map(branches) => branches
            .iter()
            .any(|branch| text_uses_formatters(&branch.value)),
        IrValue::Object(entries) => entries.iter().any(form_entry_uses_formatters),
    }
}

fn function_branch_uses_formatters(branch: &IrFunctionBranch) -> bool {
    match &branch.value {
        IrFunctionBranchValue::Text(text) => text_uses_formatters(text),
        IrFunctionBranchValue::Dispatch(branches) => {
            branches.iter().any(function_branch_uses_formatters)
        }
    }
}

fn text_uses_formatters(text: &IrText) -> bool {
    text.parts.iter().any(|part| match part {
        IrTextPart::Text(_) => false,
        IrTextPart::Placeholder(expression) => expression_uses_formatters(expression),
    })
}

fn expression_uses_formatters(expression: &IrExpression) -> bool {
    !expression.formatters.is_empty() || expression.arguments.iter().any(expression_uses_formatters)
}
