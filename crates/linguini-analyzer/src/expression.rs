use crate::{Diagnostic, QuickFix, Replacement};
use linguini_syntax::{
    Expression, FunctionBranchValue, FunctionDeclaration, LocaleDeclaration, LocaleFile, TextPart,
    TextPattern,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub ty: String,
    pub span: linguini_syntax::Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSignature {
    pub type_name: String,
    pub properties: Vec<FormProperty>,
    pub span: linguini_syntax::Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormProperty {
    pub name: String,
    pub span: linguini_syntax::Span,
    pub needs_number: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub name: String,
    pub arity: usize,
    pub span: linguini_syntax::Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageToAnalyze {
    pub name: String,
    pub value: TextPattern,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionAnalysis {
    pub messages: Vec<MessageToAnalyze>,
    pub functions: Vec<FunctionSignature>,
    pub forms: Vec<FormSignature>,
}

impl Variable {
    pub fn new(
        name: impl Into<String>,
        ty: impl Into<String>,
        span: linguini_syntax::Span,
    ) -> Self {
        Self {
            name: name.into(),
            ty: ty.into(),
            span,
        }
    }
}

impl FormProperty {
    pub fn new(name: impl Into<String>, span: linguini_syntax::Span) -> Self {
        Self {
            name: name.into(),
            span,
            needs_number: false,
        }
    }

    pub fn plural(name: impl Into<String>, span: linguini_syntax::Span) -> Self {
        Self {
            name: name.into(),
            span,
            needs_number: true,
        }
    }
}

impl FormSignature {
    pub fn new(
        type_name: impl Into<String>,
        properties: Vec<FormProperty>,
        span: linguini_syntax::Span,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            properties,
            span,
        }
    }
}

impl FunctionSignature {
    pub fn new(name: impl Into<String>, arity: usize, span: linguini_syntax::Span) -> Self {
        Self {
            name: name.into(),
            arity,
            span,
        }
    }
}

impl MessageToAnalyze {
    pub fn new(name: impl Into<String>, value: TextPattern, variables: Vec<Variable>) -> Self {
        Self {
            name: name.into(),
            value,
            variables,
        }
    }
}

pub fn analyze_expressions(input: ExpressionAnalysis) -> Vec<Diagnostic> {
    let functions: BTreeMap<_, _> = input
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect();
    let forms: BTreeMap<_, _> = input
        .forms
        .iter()
        .map(|form| (form.type_name.as_str(), form))
        .collect();
    let mut diagnostics = Vec::new();

    for message in input.messages {
        let variables: BTreeMap<_, _> = message
            .variables
            .iter()
            .map(|variable| (variable.name.as_str(), variable))
            .collect();
        let numeric_variables = numeric_variables(&message.variables);
        analyze_text(
            &message.value,
            &variables,
            &functions,
            &forms,
            &numeric_variables,
            &mut diagnostics,
        );
    }

    diagnostics
}

pub fn analyze_function_patterns(file: &LocaleFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for declaration in &file.declarations {
        collect_function_pattern_diagnostics(declaration, &mut diagnostics);
    }
    diagnostics
}

fn analyze_text(
    text: &TextPattern,
    variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for part in &text.parts {
        if let TextPart::Placeholder(placeholder) = part {
            analyze_expression(
                &placeholder.expression,
                variables,
                functions,
                forms,
                numeric_variables,
                diagnostics,
            );
        }
    }
}

fn analyze_expression(
    expression: &Expression,
    variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for argument in &expression.arguments {
        analyze_expression(
            argument,
            variables,
            functions,
            forms,
            numeric_variables,
            diagnostics,
        );
    }

    if expression.path.is_empty() {
        return;
    }

    if expression.arguments.is_empty() {
        analyze_path(expression, variables, forms, numeric_variables, diagnostics);
    } else {
        analyze_call(
            expression,
            variables,
            functions,
            forms,
            numeric_variables,
            diagnostics,
        );
    }
}

fn analyze_path(
    expression: &Expression,
    variables: &BTreeMap<&str, &Variable>,
    forms: &BTreeMap<&str, &FormSignature>,
    numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let root = &expression.path[0];
    let Some(variable) = variables.get(root.value.as_str()) else {
        diagnostics.push(Diagnostic::error(
            format!("unknown variable `{}`", root.value),
            root.span,
        ));
        return;
    };

    if expression.path.len() == 1 {
        return;
    }

    let property = &expression.path[1];
    let Some(form) = forms.get(variable.ty.as_str()) else {
        diagnostics.push(Diagnostic::error(
            format!("type `{}` has no form properties", variable.ty),
            property.span,
        ));
        return;
    };
    let Some(property_signature) = form
        .properties
        .iter()
        .find(|candidate| candidate.name == property.value)
    else {
        diagnostics.push(
            Diagnostic::error(
                format!(
                    "unknown form property `{}` on type `{}`",
                    property.value, variable.ty
                ),
                property.span,
            )
            .with_related(form.span, "form is declared here"),
        );
        return;
    };

    if property_signature.needs_number && numeric_variables.len() > 1 {
        let expression_path = expression_path(expression);
        let mut diagnostic = Diagnostic::error(
            format!(
                "ambiguous implicit plural argument for `{expression_path}`; pass a numeric argument explicitly",
            ),
            expression.span,
        );
        for variable in numeric_variables {
            diagnostic = diagnostic.with_quick_fix(QuickFix::replacement(
                format!("pass `{}` explicitly", variable.name),
                Replacement {
                    span: expression.span,
                    text: format!("{expression_path}({})", variable.name),
                },
            ));
        }
        diagnostics.push(diagnostic);
    }
}

fn analyze_call(
    expression: &Expression,
    variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if expression.path.len() == 1 {
        let name = &expression.path[0];
        if name.value == "plural" {
            if expression.arguments.len() != 1 {
                diagnostics.push(Diagnostic::error(
                    format!(
                        "function `plural` expects 1 argument(s), got {}",
                        expression.arguments.len()
                    ),
                    expression.span,
                ));
            }
            return;
        }

        let Some(function) = functions.get(name.value.as_str()) else {
            diagnostics.push(Diagnostic::error(
                format!("unknown function `{}`", name.value),
                name.span,
            ));
            return;
        };

        if function.arity != expression.arguments.len() {
            diagnostics.push(
                Diagnostic::error(
                    format!(
                        "function `{}` expects {} argument(s), got {}",
                        name.value,
                        function.arity,
                        expression.arguments.len()
                    ),
                    expression.span,
                )
                .with_related(function.span, "function is declared here"),
            );
        }
        return;
    }

    analyze_path(expression, variables, forms, numeric_variables, diagnostics);
}

fn collect_function_pattern_diagnostics(
    declaration: &LocaleDeclaration,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match declaration {
        LocaleDeclaration::Function(function) => {
            validate_function_branch_patterns(function, diagnostics);
        }
        LocaleDeclaration::Override(declaration) => {
            collect_function_pattern_diagnostics(declaration, diagnostics);
        }
        _ => {}
    }
}

fn validate_function_branch_patterns(
    function: &FunctionDeclaration,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let dispatch_parameter_count = function
        .parameters
        .iter()
        .filter(|parameter| parameter.ty.value != "String")
        .count();
    validate_branch_depth(
        &function.branches,
        function,
        dispatch_parameter_count,
        0,
        diagnostics,
    );
}

fn validate_branch_depth(
    branches: &[linguini_syntax::FunctionBranch],
    function: &FunctionDeclaration,
    dispatch_parameter_count: usize,
    depth: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for branch in branches {
        match &branch.value {
            FunctionBranchValue::Text(_) if depth + 1 != dispatch_parameter_count => {
                diagnostics.push(Diagnostic::error(
                    format!(
                        "function `{}` branch pattern expects {} value(s), got {}",
                        function.name.value,
                        dispatch_parameter_count,
                        depth + 1
                    ),
                    branch.span,
                ));
            }
            FunctionBranchValue::Dispatch(branches) => {
                validate_branch_depth(
                    branches,
                    function,
                    dispatch_parameter_count,
                    depth + 1,
                    diagnostics,
                );
            }
            FunctionBranchValue::Text(_) => {}
        }
    }
}

fn numeric_variables(variables: &[Variable]) -> Vec<&Variable> {
    variables
        .iter()
        .filter(|variable| matches!(variable.ty.as_str(), "Number" | "Decimal"))
        .collect()
}

fn expression_path(expression: &Expression) -> String {
    expression
        .path
        .iter()
        .map(|name| name.value.as_str())
        .collect::<Vec<_>>()
        .join(".")
}
