use crate::branch_coverage::validate_branch_sequence;
use crate::{
    analyze_branch_coverage, require_other_branch, BranchCoverage, Diagnostic, NamedSpan, QuickFix,
    Replacement,
};
use linguini_core::{is_plural_intrinsic, PLURAL_TYPE_NAME};
use linguini_syntax::{
    Expression, ExpressionKind, FormEntry, FormatterKind, FunctionBranchValue, FunctionDeclaration,
    InlineFunctionInput, LocaleDeclaration, LocaleFile, LocaleValue, SchemaDeclaration, SchemaFile,
    Span, TextPart, TextPattern,
};
use std::collections::{BTreeMap, BTreeSet};

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
    pub parameter_types: Vec<String>,
    pub result_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub name: String,
    pub arity: usize,
    pub parameter_types: Vec<Option<String>>,
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
    pub variables: Vec<Variable>,
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
            parameter_types: Vec::new(),
            result_type: "String".to_owned(),
        }
    }

    pub fn plural(name: impl Into<String>, span: linguini_syntax::Span) -> Self {
        Self {
            name: name.into(),
            span,
            needs_number: true,
            parameter_types: vec!["Plural".to_owned()],
            result_type: "String".to_owned(),
        }
    }

    pub fn dispatch(
        name: impl Into<String>,
        parameter_types: impl IntoIterator<Item = impl Into<String>>,
        span: linguini_syntax::Span,
    ) -> Self {
        let parameter_types = parameter_types
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        Self {
            name: name.into(),
            span,
            needs_number: parameter_types.as_slice() == ["Plural"],
            parameter_types,
            result_type: "String".to_owned(),
        }
    }

    pub fn typed(
        name: impl Into<String>,
        result_type: impl Into<String>,
        span: linguini_syntax::Span,
    ) -> Self {
        Self {
            name: name.into(),
            span,
            needs_number: false,
            parameter_types: Vec::new(),
            result_type: result_type.into(),
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
            parameter_types: vec![None; arity],
            span,
        }
    }

    pub fn typed(
        name: impl Into<String>,
        parameter_types: impl IntoIterator<Item = impl Into<String>>,
        span: linguini_syntax::Span,
    ) -> Self {
        let parameter_types = parameter_types
            .into_iter()
            .map(|ty| Some(ty.into()))
            .collect::<Vec<_>>();
        Self {
            name: name.into(),
            arity: parameter_types.len(),
            parameter_types,
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
    analyze_expressions_with_enums(input, &BTreeMap::new(), &BTreeMap::new())
}

fn analyze_expressions_with_enums(
    input: ExpressionAnalysis,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    type_aliases: &BTreeMap<&str, &str>,
) -> Vec<Diagnostic> {
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
    let global_variables = input
        .variables
        .iter()
        .map(|variable| (variable.name.as_str(), variable))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();

    for message in input.messages {
        let all_variables = input
            .variables
            .iter()
            .chain(message.variables.iter())
            .collect::<Vec<_>>();
        let mut variables = BTreeMap::new();
        for variable in &all_variables {
            if let Some(previous) = variables.insert(variable.name.as_str(), *variable) {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "message-local variable `{}` shadows another variable",
                            variable.name
                        ),
                        variable.span,
                    )
                    .with_code("linguini.shadowed_name")
                    .with_related(previous.span, "previous variable is here"),
                );
            }
        }
        let numeric_variables = numeric_variables(&all_variables);
        analyze_text(
            &message.value,
            &variables,
            &global_variables,
            &functions,
            &forms,
            enum_variants,
            type_aliases,
            &numeric_variables,
            &mut diagnostics,
        );
    }

    diagnostics
}

pub fn analyze_project_expressions(schema: &SchemaFile, locale: &LocaleFile) -> Vec<Diagnostic> {
    let mut schema_messages = BTreeMap::new();
    for declaration in &schema.declarations {
        collect_schema_messages(declaration, None, &mut schema_messages);
    }
    let type_aliases = schema
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            SchemaDeclaration::TypeAlias(item) => {
                Some((item.name.value.as_str(), item.target.value.as_str()))
            }
            SchemaDeclaration::Enum(_)
            | SchemaDeclaration::Message(_)
            | SchemaDeclaration::Group(_) => None,
        })
        .collect::<BTreeMap<_, _>>();
    let schema_enum_names = schema
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            SchemaDeclaration::Enum(item) => Some(item.name.value.as_str()),
            SchemaDeclaration::TypeAlias(_)
            | SchemaDeclaration::Message(_)
            | SchemaDeclaration::Group(_) => None,
        })
        .collect::<BTreeSet<_>>();
    let mut enum_variants = schema
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            SchemaDeclaration::Enum(item) => Some((
                item.name.value.clone(),
                item.variants
                    .iter()
                    .map(|variant| NamedSpan::new(&variant.value, variant.span))
                    .collect(),
            )),
            SchemaDeclaration::TypeAlias(_)
            | SchemaDeclaration::Message(_)
            | SchemaDeclaration::Group(_) => None,
        })
        .collect::<BTreeMap<_, _>>();
    for alias in type_aliases.keys() {
        let resolved = resolve_schema_type(alias, &type_aliases);
        if let Some(variants) = enum_variants.get(&resolved).cloned() {
            enum_variants.insert((*alias).to_owned(), variants);
        }
    }
    for declaration in &locale.declarations {
        collect_expression_enum_variants(declaration, &mut enum_variants);
    }
    let enum_names = schema_enum_names
        .iter()
        .map(|name| (*name).to_owned())
        .chain(
            type_aliases
                .keys()
                .filter(|name| {
                    schema_enum_names.contains(resolve_schema_type(name, &type_aliases).as_str())
                })
                .map(|name| (*name).to_owned()),
        )
        .chain(locale.declarations.iter().filter_map(|declaration| {
            let LocaleDeclaration::Enum(item) = declaration else {
                return None;
            };
            Some(item.name.value.clone())
        }))
        .collect::<BTreeSet<_>>();

    let mut functions = Vec::new();
    let mut forms = Vec::new();
    let mut variables = Vec::new();
    let mut messages = Vec::new();
    for declaration in &locale.declarations {
        collect_locale_expression_inputs(
            declaration,
            None,
            &schema_messages,
            &enum_names,
            &type_aliases,
            &mut functions,
            &mut forms,
            &mut variables,
            &mut messages,
        );
    }
    analyze_expressions_with_enums(
        ExpressionAnalysis {
            variables,
            messages,
            functions,
            forms,
        },
        &enum_variants,
        &type_aliases,
    )
}

fn collect_expression_enum_variants(
    declaration: &LocaleDeclaration,
    enum_variants: &mut BTreeMap<String, Vec<NamedSpan>>,
) {
    match declaration {
        LocaleDeclaration::Enum(item) => {
            enum_variants
                .entry(item.name.value.clone())
                .or_insert_with(|| {
                    item.variants
                        .iter()
                        .map(|variant| NamedSpan::new(&variant.value, variant.span))
                        .collect()
                });
        }
        LocaleDeclaration::Override(inner) => {
            collect_expression_enum_variants(inner, enum_variants);
        }
        LocaleDeclaration::Form(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Function(_)
        | LocaleDeclaration::Message(_)
        | LocaleDeclaration::Group(_) => {}
    }
}

fn collect_schema_messages<'a>(
    declaration: &'a SchemaDeclaration,
    namespace: Option<&str>,
    messages: &mut BTreeMap<String, &'a linguini_syntax::MessageSignature>,
) {
    match declaration {
        SchemaDeclaration::Message(message) => {
            messages.insert(qualified_name(namespace, &message.name.value), message);
        }
        SchemaDeclaration::Group(group) => {
            collect_schema_group(group, namespace, messages);
        }
        SchemaDeclaration::Enum(_) | SchemaDeclaration::TypeAlias(_) => {}
    }
}

fn collect_schema_group<'a>(
    group: &'a linguini_syntax::MessageGroup,
    namespace: Option<&str>,
    messages: &mut BTreeMap<String, &'a linguini_syntax::MessageSignature>,
) {
    let group_name = qualified_name(namespace, &group.name.value);
    for message in &group.messages {
        messages.insert(
            qualified_name(Some(&group_name), &message.name.value),
            message,
        );
    }
    for child in &group.groups {
        collect_schema_group(child, Some(&group_name), messages);
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_locale_expression_inputs(
    declaration: &LocaleDeclaration,
    namespace: Option<&str>,
    schema_messages: &BTreeMap<String, &linguini_syntax::MessageSignature>,
    enum_names: &BTreeSet<String>,
    type_aliases: &BTreeMap<&str, &str>,
    functions: &mut Vec<FunctionSignature>,
    forms: &mut Vec<FormSignature>,
    variables: &mut Vec<Variable>,
    messages: &mut Vec<MessageToAnalyze>,
) {
    match declaration {
        LocaleDeclaration::Variable(variable) => {
            variables.push(Variable::new(
                &variable.name.value,
                "String",
                variable.name.span,
            ));
            messages.push(MessageToAnalyze::new(
                format!("variable `{}`", variable.name.value),
                variable.value.clone(),
                Vec::new(),
            ));
        }
        LocaleDeclaration::Function(function) => {
            functions.push(FunctionSignature::typed(
                &function.name.value,
                function
                    .parameters
                    .iter()
                    .map(|parameter| resolve_schema_type(&parameter.ty.value, type_aliases)),
                function.span,
            ));
            let parameters = named_function_parameters(&function.parameters, type_aliases);
            collect_function_branch_expression_inputs(
                &format!("function `{}`", function.name.value),
                &function.branches,
                &parameters,
                messages,
            );
        }
        LocaleDeclaration::Form(form) => {
            let mut properties = BTreeMap::new();
            for variant in &form.variants {
                collect_form_properties(
                    &variant.entries,
                    enum_names,
                    type_aliases,
                    &mut properties,
                );
                collect_form_entry_expression_inputs(
                    &format!(
                        "form `{}` variant `{}`",
                        form.name.value, variant.name.value
                    ),
                    &variant.entries,
                    &[],
                    type_aliases,
                    messages,
                );
            }
            forms.push(FormSignature::new(
                resolve_schema_type(&form.name.value, type_aliases),
                properties.into_values().collect(),
                form.span,
            ));
        }
        LocaleDeclaration::Message(message) => {
            let name = qualified_name(namespace, &message.name.value);
            let parameters = schema_messages
                .get(&name)
                .map(|signature| {
                    signature
                        .parameters
                        .iter()
                        .map(|parameter| {
                            Variable::new(
                                &parameter.name.value,
                                resolve_schema_type(&parameter.ty.value, type_aliases),
                                parameter.span,
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            messages.push(MessageToAnalyze::new(
                name,
                message.value.clone(),
                parameters,
            ));
        }
        LocaleDeclaration::Group(group) => {
            let group_name = qualified_name(namespace, &group.name.value);
            for message in &group.messages {
                let declaration = LocaleDeclaration::Message(message.clone());
                collect_locale_expression_inputs(
                    &declaration,
                    Some(&group_name),
                    schema_messages,
                    enum_names,
                    type_aliases,
                    functions,
                    forms,
                    variables,
                    messages,
                );
            }
            for child in &group.groups {
                let declaration = LocaleDeclaration::Group(child.clone());
                collect_locale_expression_inputs(
                    &declaration,
                    Some(&group_name),
                    schema_messages,
                    enum_names,
                    type_aliases,
                    functions,
                    forms,
                    variables,
                    messages,
                );
            }
        }
        LocaleDeclaration::Override(inner) => collect_locale_expression_inputs(
            inner,
            namespace,
            schema_messages,
            enum_names,
            type_aliases,
            functions,
            forms,
            variables,
            messages,
        ),
        LocaleDeclaration::Enum(_) => {}
    }
}

fn named_function_parameters(
    parameters: &[linguini_syntax::FunctionParameter],
    type_aliases: &BTreeMap<&str, &str>,
) -> Vec<Variable> {
    parameters
        .iter()
        .filter_map(|parameter| {
            parameter.name.as_ref().map(|name| {
                Variable::new(
                    &name.value,
                    resolve_schema_type(&parameter.ty.value, type_aliases),
                    name.span,
                )
            })
        })
        .collect()
}

fn collect_function_branch_expression_inputs(
    owner: &str,
    branches: &[linguini_syntax::FunctionBranch],
    variables: &[Variable],
    messages: &mut Vec<MessageToAnalyze>,
) {
    for branch in branches {
        match &branch.value {
            FunctionBranchValue::Text(text) => messages.push(MessageToAnalyze::new(
                owner,
                text.clone(),
                variables.to_vec(),
            )),
            FunctionBranchValue::Dispatch(children) => {
                collect_function_branch_expression_inputs(owner, children, variables, messages);
            }
        }
    }
}

fn collect_form_entry_expression_inputs(
    owner: &str,
    entries: &[FormEntry],
    inherited_variables: &[Variable],
    type_aliases: &BTreeMap<&str, &str>,
    messages: &mut Vec<MessageToAnalyze>,
) {
    for entry in entries {
        match entry {
            FormEntry::Branch(branch) => messages.push(MessageToAnalyze::new(
                owner,
                branch.value.clone(),
                inherited_variables.to_vec(),
            )),
            FormEntry::Attribute(attribute) => {
                let mut variables = inherited_variables.to_vec();
                variables.extend(named_function_parameters(
                    &attribute.parameters,
                    type_aliases,
                ));
                collect_locale_value_expression_inputs(
                    owner,
                    &attribute.value,
                    &variables,
                    type_aliases,
                    messages,
                );
            }
        }
    }
}

fn collect_locale_value_expression_inputs(
    owner: &str,
    value: &LocaleValue,
    variables: &[Variable],
    type_aliases: &BTreeMap<&str, &str>,
    messages: &mut Vec<MessageToAnalyze>,
) {
    match value {
        LocaleValue::Text(text) => messages.push(MessageToAnalyze::new(
            owner,
            text.clone(),
            variables.to_vec(),
        )),
        LocaleValue::Map(branches) => {
            for branch in branches {
                messages.push(MessageToAnalyze::new(
                    owner,
                    branch.value.clone(),
                    variables.to_vec(),
                ));
            }
        }
        LocaleValue::Object(entries) => {
            collect_form_entry_expression_inputs(owner, entries, variables, type_aliases, messages)
        }
    }
}

fn collect_form_properties(
    entries: &[FormEntry],
    enum_names: &BTreeSet<String>,
    type_aliases: &BTreeMap<&str, &str>,
    properties: &mut BTreeMap<String, FormProperty>,
) {
    for entry in entries {
        let FormEntry::Attribute(attribute) = entry else {
            continue;
        };
        let property = match &attribute.value {
            LocaleValue::Map(_) if attribute.parameters.is_empty() => {
                FormProperty::plural(&attribute.name.value, attribute.span)
            }
            LocaleValue::Map(_) => FormProperty::dispatch(
                &attribute.name.value,
                attribute
                    .parameters
                    .iter()
                    .map(|parameter| resolve_schema_type(&parameter.ty.value, type_aliases)),
                attribute.span,
            ),
            LocaleValue::Text(_) if enum_names.contains(attribute.name.value.as_str()) => {
                FormProperty::typed(&attribute.name.value, &attribute.name.value, attribute.span)
            }
            LocaleValue::Text(_) | LocaleValue::Object(_) => {
                FormProperty::new(&attribute.name.value, attribute.span)
            }
        };
        properties.entry(property.name.clone()).or_insert(property);
    }
}

fn resolve_schema_type(ty: &str, aliases: &BTreeMap<&str, &str>) -> String {
    let mut current = ty;
    let mut visited = BTreeSet::new();
    while let Some(target) = aliases.get(current).copied() {
        if !visited.insert(current) {
            break;
        }
        current = target;
    }
    current.to_owned()
}

fn qualified_name(namespace: Option<&str>, name: &str) -> String {
    match namespace {
        Some(namespace) => format!("{namespace}.{name}"),
        None => name.to_owned(),
    }
}

pub fn analyze_function_patterns(file: &LocaleFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for declaration in &file.declarations {
        collect_function_pattern_diagnostics(declaration, &mut diagnostics);
    }
    diagnostics
}

#[allow(clippy::too_many_arguments)]
fn analyze_text(
    text: &TextPattern,
    variables: &BTreeMap<&str, &Variable>,
    global_variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    type_aliases: &BTreeMap<&str, &str>,
    numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for part in &text.parts {
        if let TextPart::Placeholder(placeholder) = part {
            analyze_expression(
                &placeholder.expression,
                variables,
                global_variables,
                functions,
                forms,
                enum_variants,
                type_aliases,
                numeric_variables,
                diagnostics,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn analyze_expression(
    expression: &Expression,
    variables: &BTreeMap<&str, &Variable>,
    global_variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    type_aliases: &BTreeMap<&str, &str>,
    numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for argument in &expression.arguments {
        analyze_expression(
            argument,
            variables,
            global_variables,
            functions,
            forms,
            enum_variants,
            type_aliases,
            numeric_variables,
            diagnostics,
        );
    }

    match &expression.kind {
        ExpressionKind::InlineFunction { inputs, branches } => {
            analyze_inline_function(
                inputs,
                branches,
                variables,
                global_variables,
                functions,
                forms,
                enum_variants,
                type_aliases,
                numeric_variables,
                diagnostics,
            );
            analyze_formatters(expression, Some("String"), diagnostics);
        }
        ExpressionKind::Reference => {
            if expression.path.is_empty() {
                return;
            }
            analyze_path(expression, variables, forms, numeric_variables, diagnostics);
        }
        ExpressionKind::Call => {
            if expression.path.is_empty() {
                return;
            }
            analyze_call(expression, variables, functions, forms, diagnostics);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn analyze_inline_function(
    inputs: &[InlineFunctionInput],
    branches: &[linguini_syntax::FunctionBranch],
    variables: &BTreeMap<&str, &Variable>,
    global_variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    type_aliases: &BTreeMap<&str, &str>,
    outer_numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut binding_variables = Vec::new();
    let mut dispatch_types = Vec::new();
    let mut seen_bindings = BTreeMap::new();
    let mut first_binding = None;

    for input in inputs {
        let value = match input {
            InlineFunctionInput::Binding { value, .. }
            | InlineFunctionInput::Selector { value, .. } => value,
        };
        analyze_expression(
            value,
            variables,
            global_variables,
            functions,
            forms,
            enum_variants,
            type_aliases,
            outer_numeric_variables,
            diagnostics,
        );

        match input {
            InlineFunctionInput::Selector { value, span } => {
                if let Some(binding_span) = first_binding {
                    diagnostics.push(
                        Diagnostic::error(
                            "inline fn selectors must precede named payload bindings",
                            *span,
                        )
                        .with_code("linguini.parameter_order")
                        .with_related(binding_span, "first named payload binding is here"),
                    );
                }

                let selector_type = expression_type(value, variables, forms)
                    .map(|ty| resolve_schema_type(&ty, type_aliases));
                let valid_type = selector_type.as_ref().filter(|ty| {
                    ty.as_str() == PLURAL_TYPE_NAME || enum_variants.contains_key(ty.as_str())
                });
                if let Some(selector_type) = selector_type.as_deref() {
                    if valid_type.is_none() {
                        let guidance = if matches!(selector_type, "Number" | "Decimal") {
                            "; wrap numeric values in `Plural(...)`"
                        } else {
                            ""
                        };
                        diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "inline fn selector must resolve to an enum or `Plural`, got `{selector_type}`{guidance}"
                                ),
                                value.span,
                            )
                            .with_code("linguini.invalid_dispatch_type"),
                        );
                    }
                }
                dispatch_types.push(valid_type.cloned());
            }
            InlineFunctionInput::Binding { name, value, span } => {
                first_binding.get_or_insert(*span);
                if let Some(previous) = seen_bindings.insert(name.value.as_str(), name.span) {
                    diagnostics.push(
                        Diagnostic::error(
                            format!("duplicate inline fn binding `{}`", name.value),
                            name.span,
                        )
                        .with_code("linguini.duplicate_parameter")
                        .with_related(previous, "first binding is here"),
                    );
                }
                let ty = expression_type(value, variables, forms)
                    .map(|ty| resolve_schema_type(&ty, type_aliases))
                    .unwrap_or_else(|| "String".to_owned());
                binding_variables.push(Variable::new(&name.value, ty, name.span));
            }
        }
    }

    let mut branch_variables = variables.clone();
    for binding in &binding_variables {
        branch_variables.insert(binding.name.as_str(), binding);
    }
    let branch_variable_refs = branch_variables.values().copied().collect::<Vec<_>>();
    let branch_numeric_variables = numeric_variables(&branch_variable_refs);
    analyze_inline_branch_level(
        branches,
        &dispatch_types,
        0,
        &branch_variables,
        global_variables,
        functions,
        forms,
        enum_variants,
        type_aliases,
        &branch_numeric_variables,
        diagnostics,
    );
}

#[allow(clippy::too_many_arguments)]
fn analyze_inline_branch_level(
    branches: &[linguini_syntax::FunctionBranch],
    dispatch_types: &[Option<String>],
    depth: usize,
    variables: &BTreeMap<&str, &Variable>,
    global_variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    type_aliases: &BTreeMap<&str, &str>,
    numeric_variables: &[&Variable],
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_inline_selector_coverage(
        branches,
        dispatch_types.get(depth).and_then(Option::as_deref),
        enum_variants,
        diagnostics,
    );
    let dispatch_count = dispatch_types.len();
    for branch in branches {
        match &branch.value {
            FunctionBranchValue::Text(text) => {
                if dispatch_count == 0 && branch.key.value != "_" {
                    diagnostics.push(
                        Diagnostic::error(
                            "inline fn without dispatch parameters only accepts a `_` branch",
                            branch.key.span,
                        )
                        .with_code("linguini.inline_fn_depth"),
                    );
                } else if depth + 1 < dispatch_count && branch.key.value != "_" {
                    diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "inline fn branch pattern expects {dispatch_count} dispatch value(s), got {}",
                                depth + 1
                            ),
                            branch.span,
                        )
                        .with_code("linguini.inline_fn_depth"),
                    );
                }
                analyze_text(
                    text,
                    variables,
                    global_variables,
                    functions,
                    forms,
                    enum_variants,
                    type_aliases,
                    numeric_variables,
                    diagnostics,
                );
            }
            FunctionBranchValue::Dispatch(children) => {
                if depth + 1 >= dispatch_count {
                    diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "inline fn branch pattern exceeds its {dispatch_count} dispatch value(s)"
                            ),
                            branch.span,
                        )
                        .with_code("linguini.inline_fn_depth"),
                    );
                } else {
                    analyze_inline_branch_level(
                        children,
                        dispatch_types,
                        depth + 1,
                        variables,
                        global_variables,
                        functions,
                        forms,
                        enum_variants,
                        type_aliases,
                        numeric_variables,
                        diagnostics,
                    );
                }
            }
        }
    }
}

fn validate_inline_selector_coverage(
    branches: &[linguini_syntax::FunctionBranch],
    selector_type: Option<&str>,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let branch_spans = branches
        .iter()
        .map(|branch| NamedSpan::new(&branch.key.value, branch.span))
        .collect::<Vec<_>>();
    let span = branches
        .first()
        .map_or_else(|| Span::new(0, 0), |branch| branch.span);
    let Some(selector_type) = selector_type else {
        diagnostics.extend(validate_branch_sequence(&branch_spans));
        return;
    };
    let keys = branches
        .iter()
        .map(|branch| branch.key.value.as_str())
        .collect::<BTreeSet<_>>();
    let has_wildcard = keys.contains("_");

    if selector_type == PLURAL_TYPE_NAME {
        let known = ["zero", "one", "two", "few", "many", "other", "_"];
        for branch in branches {
            if !known.contains(&branch.key.value.as_str()) {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "inline fn uses unknown plural category `{}`",
                            branch.key.value
                        ),
                        branch.key.span,
                    )
                    .with_code("linguini.unknown_plural_category"),
                );
            }
        }
        diagnostics.extend(require_other_branch("inline fn", &branch_spans, span));
        if has_wildcard
            && known[..known.len() - 1]
                .iter()
                .all(|category| keys.contains(category))
        {
            if let Some(wildcard) = branches.iter().find(|branch| branch.key.value == "_") {
                diagnostics.push(
                    Diagnostic::warning(
                        "inline fn has a redundant wildcard after covering every `Plural` branch",
                        wildcard.span,
                    )
                    .as_lint("redundant_wildcard"),
                );
            }
        }
        return;
    }

    let Some(variants) = enum_variants.get(selector_type) else {
        diagnostics.extend(validate_branch_sequence(&branch_spans));
        return;
    };
    diagnostics.extend(analyze_branch_coverage(BranchCoverage {
        subject: "inline fn",
        enum_name: selector_type,
        variants: variants.clone(),
        branches: branch_spans,
        span,
    }));
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
        analyze_formatters(expression, Some(&variable.ty), diagnostics);
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

    if expression.path.len() > 2 {
        diagnostics.push(Diagnostic::error(
            format!(
                "type `{}` does not define nested property `{}`",
                variable.ty,
                expression.path[1..]
                    .iter()
                    .map(|part| part.value.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            ),
            expression.path[2].span,
        ));
        return;
    }

    if property_signature.needs_number && numeric_variables.is_empty() {
        diagnostics.push(
            Diagnostic::error(
                format!(
                    "implicit plural use for `{}` requires a numeric variable or an explicit argument",
                    expression_path(expression)
                ),
                expression.span,
            )
            .with_code("linguini.missing_plural_argument"),
        );
    } else if property_signature.needs_number && numeric_variables.len() > 1 {
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
    analyze_formatters(
        expression,
        Some(&property_signature.result_type),
        diagnostics,
    );
}

fn analyze_call(
    expression: &Expression,
    variables: &BTreeMap<&str, &Variable>,
    functions: &BTreeMap<&str, &FunctionSignature>,
    forms: &BTreeMap<&str, &FormSignature>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if expression.path.len() == 1 {
        let name = &expression.path[0];
        if is_plural_intrinsic(&name.value) {
            if expression.arguments.len() != 1 {
                diagnostics.push(Diagnostic::error(
                    format!(
                        "intrinsic `{PLURAL_TYPE_NAME}` expects 1 argument(s), got {}",
                        expression.arguments.len()
                    ),
                    expression.span,
                ));
            }
            if let Some(argument) = expression.arguments.first() {
                let argument_type = expression_type(argument, variables, forms);
                if argument_type
                    .as_deref()
                    .is_some_and(|ty| !matches!(ty, "Number" | "Decimal"))
                {
                    diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "intrinsic `{PLURAL_TYPE_NAME}` expects Number or Decimal, got `{}`",
                                argument_type.unwrap_or_default()
                            ),
                            argument.span,
                        )
                        .with_code("linguini.type_mismatch"),
                    );
                }
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
        for (index, (expected, argument)) in function
            .parameter_types
            .iter()
            .zip(&expression.arguments)
            .enumerate()
        {
            let (Some(expected), Some(actual)) = (
                expected.as_deref(),
                expression_type(argument, variables, forms),
            ) else {
                continue;
            };
            let compatible = types_compatible(expected, &actual);
            if !compatible {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "argument {} to `{}` expects `{expected}`, got `{actual}`",
                            index + 1,
                            name.value
                        ),
                        argument.span,
                    )
                    .with_code("linguini.type_mismatch")
                    .with_related(function.span, "function is declared here"),
                );
            }
        }
        return;
    }

    analyze_form_call(expression, variables, forms, diagnostics);
}

fn analyze_form_call(
    expression: &Expression,
    variables: &BTreeMap<&str, &Variable>,
    forms: &BTreeMap<&str, &FormSignature>,
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
    let Some(property_name) = expression.path.get(1) else {
        diagnostics.push(Diagnostic::error(
            format!("type `{}` has no unnamed form", variable.ty),
            expression.span,
        ));
        return;
    };
    let Some(form) = forms.get(variable.ty.as_str()) else {
        diagnostics.push(Diagnostic::error(
            format!("unknown form target type `{}`", variable.ty),
            property_name.span,
        ));
        return;
    };
    let Some(property) = form
        .properties
        .iter()
        .find(|candidate| candidate.name == property_name.value)
    else {
        diagnostics.push(
            Diagnostic::error(
                format!(
                    "unknown form property `{}` on type `{}`",
                    property_name.value, variable.ty
                ),
                property_name.span,
            )
            .with_related(form.span, "form is declared here"),
        );
        return;
    };
    if expression.path.len() > 2 {
        diagnostics.push(Diagnostic::error(
            format!(
                "unknown nested form property `{}`",
                expression.path[1..]
                    .iter()
                    .map(|part| part.value.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            ),
            expression.path[2].span,
        ));
    }
    let expected = property.parameter_types.len();
    if expression.arguments.len() != expected {
        diagnostics.push(
            Diagnostic::error(
                format!(
                    "form property `{}.{}` expects {expected} argument(s), got {}",
                    variable.ty,
                    property.name,
                    expression.arguments.len()
                ),
                expression.span,
            )
            .with_code("linguini.call_arity"),
        );
    }
    for (index, (expected, argument)) in property
        .parameter_types
        .iter()
        .zip(&expression.arguments)
        .enumerate()
    {
        if let Some(actual) = expression_type(argument, variables, forms) {
            let compatible = types_compatible(expected, &actual);
            if !compatible {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "argument {} to form property `{}.{}` expects `{expected}`, got `{actual}`",
                            index + 1,
                            variable.ty,
                            property.name,
                        ),
                        argument.span,
                    )
                    .with_code("linguini.type_mismatch"),
                );
            }
        }
    }
    analyze_formatters(expression, Some(&property.result_type), diagnostics);
}

fn types_compatible(expected: &str, actual: &str) -> bool {
    expected == actual
        || (expected == "Plural" && matches!(actual, "Number" | "Decimal"))
        || (matches!(expected, "Number" | "Decimal") && matches!(actual, "Number" | "Decimal"))
}

fn expression_type(
    expression: &Expression,
    variables: &BTreeMap<&str, &Variable>,
    forms: &BTreeMap<&str, &FormSignature>,
) -> Option<String> {
    if matches!(&expression.kind, ExpressionKind::InlineFunction { .. }) {
        return Some("String".to_owned());
    }
    let root = expression.path.first()?;
    if expression.kind == ExpressionKind::Call {
        if expression.path.len() == 1 && is_plural_intrinsic(&root.value) {
            return Some(PLURAL_TYPE_NAME.to_owned());
        }
        return Some("String".to_owned());
    }
    let variable = variables.get(root.value.as_str())?;
    if expression.path.len() == 1 {
        return Some(variable.ty.clone());
    }
    let property = forms
        .get(variable.ty.as_str())?
        .properties
        .iter()
        .find(|property| property.name == expression.path[1].value)?;
    Some(property.result_type.clone())
}

fn analyze_formatters(
    expression: &Expression,
    value_type: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for formatter in &expression.annotations {
        match &formatter.kind {
            FormatterKind::Unknown(name) => diagnostics.push(
                Diagnostic::error(format!("unknown formatter `{name}`"), formatter.span)
                    .with_code("linguini.unknown_formatter"),
            ),
            FormatterKind::Number | FormatterKind::Currency
                if value_type.is_some_and(|ty| !matches!(ty, "Number" | "Decimal")) =>
            {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "formatter `{}` requires Number or Decimal, got `{}`",
                            formatter.kind.as_str(),
                            value_type.unwrap_or("unknown")
                        ),
                        formatter.span,
                    )
                    .with_code("linguini.formatter_type"),
                );
            }
            FormatterKind::Date if value_type.is_some_and(|ty| ty != "Date") => {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "formatter `date` requires Date, got `{}`",
                            value_type.unwrap_or("unknown")
                        ),
                        formatter.span,
                    )
                    .with_code("linguini.formatter_type"),
                );
            }
            FormatterKind::Number | FormatterKind::Currency | FormatterKind::Date => {}
        }
    }
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
        .filter(|parameter| {
            parameter.name.is_none()
                && !matches!(parameter.ty.value.as_str(), "Date" | "Boolean" | "String")
        })
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
            FunctionBranchValue::Text(_)
                if depth + 1 > dispatch_parameter_count
                    || (depth + 1 < dispatch_parameter_count && branch.key.value != "_") =>
            {
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

fn numeric_variables<'a>(variables: &[&'a Variable]) -> Vec<&'a Variable> {
    variables
        .iter()
        .filter(|variable| matches!(variable.ty.as_str(), "Number" | "Decimal"))
        .copied()
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
