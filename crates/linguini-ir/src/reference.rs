use crate::model::{IrExpression, IrModule, IrText, IrTextPart};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrReferenceError {
    pub message: String,
}

pub fn ensure_no_unresolved_references(
    schema: &IrModule,
    locale: &IrModule,
) -> Result<(), Vec<IrReferenceError>> {
    let context = ReferenceContext::new(schema, locale);
    let mut errors = Vec::new();

    for message in &locale.messages {
        let Some(parameters) = context.message_parameters.get(&message.name) else {
            errors.push(IrReferenceError {
                message: format!("unresolved message `{}`", message.name),
            });
            continue;
        };
        if let Some(body) = &message.body {
            check_text(body, parameters, &context, &mut errors);
        }
    }

    for function in &locale.functions {
        let variables: BTreeSet<_> = function.parameters.iter().cloned().collect();
        for branch in &function.branches {
            check_text(&branch.value, &variables, &context, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

struct ReferenceContext {
    message_parameters: BTreeMap<String, BTreeSet<String>>,
    functions: BTreeSet<String>,
    forms: BTreeSet<String>,
}

impl ReferenceContext {
    fn new(schema: &IrModule, locale: &IrModule) -> Self {
        Self {
            message_parameters: schema
                .messages
                .iter()
                .map(|message| {
                    (
                        message.name.clone(),
                        message
                            .parameters
                            .iter()
                            .map(|parameter| parameter.name.clone())
                            .collect(),
                    )
                })
                .collect(),
            functions: locale
                .functions
                .iter()
                .map(|function| function.name.clone())
                .collect(),
            forms: locale.forms.iter().map(|form| form.name.clone()).collect(),
        }
    }
}

fn check_text(
    text: &IrText,
    variables: &BTreeSet<String>,
    context: &ReferenceContext,
    errors: &mut Vec<IrReferenceError>,
) {
    for part in &text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            check_expression(expression, variables, context, errors);
        }
    }
}

fn check_expression(
    expression: &IrExpression,
    variables: &BTreeSet<String>,
    context: &ReferenceContext,
    errors: &mut Vec<IrReferenceError>,
) {
    let Some(root) = expression.path.first() else {
        errors.push(IrReferenceError {
            message: "unresolved empty expression".to_owned(),
        });
        return;
    };

    let resolved = variables.contains(root)
        || context.forms.contains(root)
        || context.functions.contains(root);

    if !resolved {
        errors.push(IrReferenceError {
            message: format!("unresolved reference `{}`", expression.path.join(".")),
        });
    }

    for argument in &expression.arguments {
        check_expression(argument, variables, context, errors);
    }
}
