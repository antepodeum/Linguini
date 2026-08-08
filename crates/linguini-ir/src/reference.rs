use crate::model::{
    IrExpression, IrExpressionKind, IrForm, IrFormEntry, IrFunction, IrFunctionBranch,
    IrFunctionBranchValue, IrInlineFunctionInput, IrModule, IrText, IrTextPart, IrValue,
};
use linguini_core::{is_plural_intrinsic, FormatterKind, TypeKind, PLURAL_TYPE_NAME};
use linguini_syntax::Span;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

pub const BUILTIN_PLURAL: &str = PLURAL_TYPE_NAME;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrRelatedError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrReferenceError {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
    pub related: Vec<IrRelatedError>,
}

impl IrReferenceError {
    fn new(code: &'static str, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }

    fn at(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self::new(code, message, Some(span))
    }

    fn related(mut self, span: Span, message: impl Into<String>) -> Self {
        self.related.push(IrRelatedError {
            span,
            message: message.into(),
        });
        self
    }
}

/// Capability proving that both modules passed structural, reference, type, cycle and formatter
/// validation. Production emitters should accept this type instead of raw [`IrModule`] values.
#[derive(Debug)]
pub struct ValidatedIr<'a> {
    schema: &'a IrModule,
    locale: &'a IrModule,
}

impl<'a> ValidatedIr<'a> {
    pub fn schema(&self) -> &'a IrModule {
        self.schema
    }

    pub fn locale(&self) -> &'a IrModule {
        self.locale
    }
}

pub fn validate_ir<'a>(
    schema: &'a IrModule,
    locale: &'a IrModule,
) -> Result<ValidatedIr<'a>, Vec<IrReferenceError>> {
    let mut errors = Vec::new();
    validate_structure("schema", schema, &mut errors);
    validate_structure("locale", locale, &mut errors);

    let context = ReferenceContext::new(schema, locale, &mut errors);
    validate_schema_types(schema, &context, &mut errors);
    validate_locale(schema, locale, &context, &mut errors);
    validate_reference_cycles(locale, &context, &mut errors);

    if errors.is_empty() {
        Ok(ValidatedIr { schema, locale })
    } else {
        Err(errors)
    }
}

pub fn validate_typed_ir<'a>(
    schema: &'a crate::SchemaIr,
    locale: &'a crate::LocaleIr,
) -> Result<ValidatedIr<'a>, Vec<IrReferenceError>> {
    validate_ir(schema.as_module(), locale.as_module())
}

pub fn ensure_no_unresolved_references(
    schema: &IrModule,
    locale: &IrModule,
) -> Result<(), Vec<IrReferenceError>> {
    validate_ir(schema, locale).map(|_| ())
}

fn validate_structure(label: &str, module: &IrModule, errors: &mut Vec<IrReferenceError>) {
    let mut origin_names = BTreeMap::new();
    for origin in &module.origins {
        match origin_names.entry(origin.name.as_str()) {
            Entry::Vacant(entry) => {
                entry.insert((origin.kind, origin.span));
            }
            Entry::Occupied(mut first) if origin.is_override => {
                first.insert((origin.kind, origin.span));
            }
            Entry::Occupied(first) => errors.push(
                IrReferenceError::at(
                    "IR000",
                    format!(
                        "duplicate {label} path `{}` ({:?} conflicts with {:?})",
                        origin.name,
                        origin.kind,
                        first.get().0
                    ),
                    origin.span,
                )
                .related(first.get().1, "first path declaration is here"),
            ),
        }
    }

    unique_named(
        label,
        "enum",
        module.enums.iter().map(|item| item.name.as_str()),
        module,
        errors,
    );
    unique_named(
        label,
        "type alias",
        module.type_aliases.iter().map(|item| item.name.as_str()),
        module,
        errors,
    );
    unique_named(
        label,
        "variable",
        module.variables.iter().map(|item| item.name.as_str()),
        module,
        errors,
    );
    unique_named(
        label,
        "message",
        module.messages.iter().map(|item| item.name.as_str()),
        module,
        errors,
    );
    unique_named(
        label,
        "group",
        module.groups.iter().map(|item| item.name.as_str()),
        module,
        errors,
    );
    unique_named(
        label,
        "form",
        module.forms.iter().map(|item| item.name.as_str()),
        module,
        errors,
    );
    unique_named(
        label,
        "function",
        module.functions.iter().map(|item| item.name.as_str()),
        module,
        errors,
    );

    let mut all_names = BTreeMap::<&str, (&str, Option<Span>)>::new();
    for (kind, names) in [
        (
            "enum",
            module
                .enums
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
        ),
        (
            "type alias",
            module
                .type_aliases
                .iter()
                .map(|item| item.name.as_str())
                .collect(),
        ),
        (
            "variable",
            module
                .variables
                .iter()
                .map(|item| item.name.as_str())
                .collect(),
        ),
        (
            "message",
            module
                .messages
                .iter()
                .map(|item| item.name.as_str())
                .collect(),
        ),
        (
            "group",
            module
                .groups
                .iter()
                .map(|item| item.name.as_str())
                .collect(),
        ),
        (
            "form",
            module.forms.iter().map(|item| item.name.as_str()).collect(),
        ),
        (
            "function",
            module
                .functions
                .iter()
                .map(|item| item.name.as_str())
                .collect(),
        ),
    ] {
        for name in names {
            let span = origin_span(module, name);
            match all_names.entry(name) {
                Entry::Vacant(entry) => {
                    entry.insert((kind, span));
                }
                Entry::Occupied(first) if first.get().0 != kind => {
                    let mut error = IrReferenceError::new(
                        "IR002",
                        format!(
                            "{label} symbol `{name}` is both {} and {kind}",
                            first.get().0
                        ),
                        span,
                    );
                    if let Some(first_span) = first.get().1 {
                        error = error.related(first_span, "first declaration is here");
                    }
                    errors.push(error);
                }
                Entry::Occupied(_) => {}
            }
        }
    }

    for item in &module.enums {
        duplicate_strings(
            &format!("enum `{}` variant", item.name),
            &item.variants,
            origin_span(module, &item.name),
            errors,
        );
    }
    for item in &module.messages {
        duplicate_strings(
            &format!("message `{}` parameter", item.name),
            &item
                .parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect::<Vec<_>>(),
            origin_span(module, &item.name),
            errors,
        );
    }
    for form in &module.forms {
        duplicate_strings(
            &format!("form `{}` variant", form.name),
            &form
                .variants
                .iter()
                .map(|variant| variant.name.clone())
                .collect::<Vec<_>>(),
            origin_span(module, &form.name),
            errors,
        );
        for variant in &form.variants {
            validate_form_entries(
                &format!("form `{}` variant `{}`", form.name, variant.name),
                &variant.entries,
                errors,
            );
        }
    }
    for function in &module.functions {
        validate_parameter_order(
            &format!("function `{}`", function.name),
            &function.parameters,
            origin_span(module, &function.name),
            errors,
        );
        validate_function_branches(&function.name, &function.branches, errors);
    }
}

fn validate_parameter_order(
    subject: &str,
    parameters: &[crate::IrFunctionParameter],
    span: Option<Span>,
    errors: &mut Vec<IrReferenceError>,
) {
    let mut saw_payload = false;
    let mut names = BTreeSet::new();
    for parameter in parameters {
        match &parameter.name {
            Some(name) => {
                saw_payload = true;
                if !names.insert(name) {
                    errors.push(IrReferenceError::new(
                        "IR003",
                        format!("duplicate {subject} payload parameter `{name}`"),
                        span,
                    ));
                }
            }
            None if saw_payload => errors.push(IrReferenceError::new(
                "IR038",
                format!("{subject} dispatch parameters must precede named payload parameters"),
                span,
            )),
            None => {}
        }
    }
}

fn unique_named<'a>(
    module_label: &str,
    kind: &str,
    names: impl Iterator<Item = &'a str>,
    module: &IrModule,
    errors: &mut Vec<IrReferenceError>,
) {
    let mut seen = BTreeSet::new();
    for name in names {
        if !seen.insert(name) {
            errors.push(IrReferenceError::new(
                "IR001",
                format!("duplicate {module_label} {kind} `{name}`"),
                origin_span(module, name),
            ));
        }
    }
}

fn duplicate_strings(
    subject: &str,
    values: &[String],
    span: Option<Span>,
    errors: &mut Vec<IrReferenceError>,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            errors.push(IrReferenceError::new(
                "IR003",
                format!("duplicate {subject} `{value}`"),
                span,
            ));
        }
    }
}

fn validate_form_entries(
    subject: &str,
    entries: &[IrFormEntry],
    errors: &mut Vec<IrReferenceError>,
) {
    let mut attributes = BTreeSet::new();
    let mut branch_patterns = BTreeSet::new();
    for entry in entries {
        match entry {
            IrFormEntry::Attribute {
                name,
                parameters,
                value,
            } => {
                if !attributes.insert(name) {
                    errors.push(IrReferenceError::new(
                        "IR004",
                        format!("duplicate {subject} attribute `{name}`"),
                        value_span(value),
                    ));
                }
                validate_parameter_order(
                    &format!("{subject} attribute `{name}`"),
                    parameters,
                    value_span(value),
                    errors,
                );
                match value {
                    IrValue::Text(_) => {}
                    IrValue::Map(branches) => validate_map_branches(subject, branches, errors),
                    IrValue::Object(children) => {
                        validate_form_entries(&format!("{subject}.{name}"), children, errors);
                    }
                }
            }
            IrFormEntry::Branch(branch) => {
                let pattern = branch.keys.join("\u{1f}");
                if !branch_patterns.insert(pattern) {
                    errors.push(IrReferenceError::at(
                        "IR005",
                        format!("duplicate {subject} branch `{}`", branch.keys.join(", ")),
                        branch.span,
                    ));
                }
            }
        }
    }
}

fn validate_map_branches(
    subject: &str,
    branches: &[crate::IrBranch],
    errors: &mut Vec<IrReferenceError>,
) {
    let mut patterns = BTreeSet::new();
    for branch in branches {
        let pattern = branch.keys.join("\u{1f}");
        if !patterns.insert(pattern) {
            errors.push(IrReferenceError::at(
                "IR005",
                format!(
                    "duplicate {subject} map branch `{}`",
                    branch.keys.join(", ")
                ),
                branch.span,
            ));
        }
    }
}

fn validate_function_branches(
    function: &str,
    branches: &[IrFunctionBranch],
    errors: &mut Vec<IrReferenceError>,
) {
    let mut keys = BTreeSet::new();
    let mut wildcard = None;
    for branch in branches {
        if !keys.insert(branch.key.as_str()) {
            errors.push(IrReferenceError::at(
                "IR006",
                format!("duplicate function `{function}` branch `{}`", branch.key),
                branch.span,
            ));
        }
        if let Some(wildcard_span) = wildcard {
            errors.push(
                IrReferenceError::at(
                    "IR037",
                    format!(
                        "function `{function}` branch `{}` is unreachable after `_`",
                        branch.key
                    ),
                    branch.span,
                )
                .related(wildcard_span, "wildcard branch is here"),
            );
        }
        if branch.key == "_" && wildcard.is_none() {
            wildcard = Some(branch.span);
        }
        if let IrFunctionBranchValue::Dispatch(children) = &branch.value {
            validate_function_branches(function, children, errors);
        }
    }
}

fn value_span(value: &IrValue) -> Option<Span> {
    match value {
        IrValue::Text(text) => Some(text.span),
        IrValue::Map(branches) => branches.first().map(|branch| branch.span),
        IrValue::Object(entries) => entries.iter().find_map(|entry| match entry {
            IrFormEntry::Attribute { value, .. } => value_span(value),
            IrFormEntry::Branch(branch) => Some(branch.span),
        }),
    }
}

struct ReferenceContext<'a> {
    messages: BTreeMap<&'a str, &'a crate::IrMessage>,
    functions: BTreeMap<&'a str, &'a IrFunction>,
    forms: BTreeMap<&'a str, &'a IrForm>,
    variables: BTreeMap<&'a str, &'a crate::IrVariable>,
    enums: BTreeMap<&'a str, &'a crate::IrEnum>,
    aliases: BTreeMap<&'a str, &'a crate::IrTypeAlias>,
}

impl<'a> ReferenceContext<'a> {
    fn new(schema: &'a IrModule, locale: &'a IrModule, errors: &mut Vec<IrReferenceError>) -> Self {
        let mut enums = first_wins(schema.enums.iter().map(|item| (item.name.as_str(), item)));
        for item in &locale.enums {
            if let Some(schema_enum) = enums.get(item.name.as_str()) {
                errors.push(
                    IrReferenceError::new(
                        "IR007",
                        format!(
                            "locale enum `{}` conflicts with a schema enum of the same name",
                            item.name
                        ),
                        origin_span(locale, &item.name),
                    )
                    .related(
                        origin_span(schema, &schema_enum.name).unwrap_or_else(|| Span::new(0, 0)),
                        "schema enum is here",
                    ),
                );
            } else {
                enums.insert(item.name.as_str(), item);
            }
        }

        Self {
            messages: first_wins(
                schema
                    .messages
                    .iter()
                    .map(|item| (item.name.as_str(), item)),
            ),
            functions: first_wins(
                locale
                    .functions
                    .iter()
                    .map(|item| (item.name.as_str(), item)),
            ),
            forms: first_wins(locale.forms.iter().map(|item| (item.name.as_str(), item))),
            variables: first_wins(
                locale
                    .variables
                    .iter()
                    .map(|item| (item.name.as_str(), item)),
            ),
            enums,
            aliases: first_wins(
                schema
                    .type_aliases
                    .iter()
                    .map(|item| (item.name.as_str(), item)),
            ),
        }
    }

    fn resolve_alias<'b>(&'b self, ty: &'b str) -> Result<&'b str, Vec<&'b str>> {
        let mut current = ty;
        let mut path = Vec::new();
        while let Some(alias) = self.aliases.get(current) {
            if let Some(offset) = path.iter().position(|item| *item == current) {
                path.push(current);
                return Err(path[offset..].to_vec());
            }
            path.push(current);
            current = &alias.target;
        }
        Ok(current)
    }

    fn known_type(&self, ty: &str) -> bool {
        self.resolve_alias(ty).is_ok_and(|resolved| {
            TypeKind::from_name(resolved).is_some() || self.enums.contains_key(resolved)
        })
    }
}

fn first_wins<K: Ord, V>(items: impl Iterator<Item = (K, V)>) -> BTreeMap<K, V> {
    let mut output = BTreeMap::new();
    for (key, value) in items {
        output.entry(key).or_insert(value);
    }
    output
}

fn validate_schema_types(
    schema: &IrModule,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    let mut reported_alias_cycles = BTreeSet::new();
    for alias in &schema.type_aliases {
        match context.resolve_alias(&alias.name) {
            Err(cycle) => {
                let mut identity = cycle[..cycle.len().saturating_sub(1)].to_vec();
                identity.sort_unstable();
                identity.dedup();
                if reported_alias_cycles.insert(identity.join("\u{1f}")) {
                    errors.push(IrReferenceError::new(
                        "IR008",
                        format!("cyclic type alias `{}`", cycle.join(" -> ")),
                        origin_span(schema, &alias.name),
                    ));
                }
            }
            Ok(resolved)
                if TypeKind::from_name(resolved).is_none()
                    && !context.enums.contains_key(resolved) =>
            {
                errors.push(IrReferenceError::new(
                    "IR009",
                    format!("unknown type `{resolved}` in alias `{}`", alias.name),
                    origin_span(schema, &alias.name),
                ));
            }
            Ok(_) => {}
        }
        validate_formatters(
            &alias.formatters,
            Some(&alias.target),
            None,
            context,
            errors,
        );
    }

    for message in &schema.messages {
        for parameter in &message.parameters {
            if !context.known_type(&parameter.ty) {
                errors.push(IrReferenceError::new(
                    "IR009",
                    format!(
                        "unknown type `{}` for message `{}.{}`",
                        parameter.ty, message.name, parameter.name
                    ),
                    origin_span(schema, &message.name),
                ));
            }
        }
    }
}

fn validate_locale(
    schema: &IrModule,
    locale: &IrModule,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    let global_variables = global_variables(context);
    let no_capture_candidates = BTreeMap::new();

    for message in &locale.messages {
        let Some(signature) = context.messages.get(message.name.as_str()) else {
            errors.push(IrReferenceError::new(
                "IR010",
                format!("unresolved message `{}`", message.name),
                origin_span(locale, &message.name),
            ));
            continue;
        };
        let mut variables = global_variables.clone();
        let mut capture_candidates = BTreeMap::new();
        for parameter in &signature.parameters {
            if variables
                .insert(parameter.name.clone(), parameter.ty.clone())
                .is_some()
            {
                errors.push(IrReferenceError::new(
                    "IR011",
                    format!(
                        "message parameter `{}` shadows a global variable",
                        parameter.name
                    ),
                    origin_span(schema, &signature.name),
                ));
            }
            capture_candidates.insert(parameter.name.clone(), parameter.ty.clone());
        }
        if let Some(body) = &message.body {
            check_text(body, &variables, &capture_candidates, context, errors);
        }
    }

    for variable in &locale.variables {
        check_text(
            &variable.value,
            &global_variables,
            &no_capture_candidates,
            context,
            errors,
        );
    }

    for function in &locale.functions {
        let mut variables = global_variables.clone();
        let mut capture_candidates = BTreeMap::new();
        for parameter in &function.parameters {
            if !context.known_type(&parameter.ty) && parameter.ty != "Plural" {
                errors.push(IrReferenceError::new(
                    "IR009",
                    format!(
                        "unknown parameter type `{}` in function `{}`",
                        parameter.ty, function.name
                    ),
                    origin_span(locale, &function.name),
                ));
            }
            if let Some(name) = &parameter.name {
                if variables
                    .insert(name.clone(), parameter.ty.clone())
                    .is_some()
                {
                    errors.push(IrReferenceError::new(
                        "IR011",
                        format!("function parameter `{name}` shadows a global variable"),
                        origin_span(locale, &function.name),
                    ));
                }
                capture_candidates.insert(name.clone(), parameter.ty.clone());
            }
        }
        validate_function_dispatch_coverage(
            function,
            &function.branches,
            0,
            context,
            origin_span(locale, &function.name),
            errors,
        );
        for branch in &function.branches {
            check_function_branch(branch, &variables, &capture_candidates, context, errors);
        }
    }

    for form in &locale.forms {
        for variant in &form.variants {
            check_form_entries(
                &form.name,
                &variant.entries,
                &global_variables,
                &no_capture_candidates,
                context,
                errors,
            );
        }
    }
}

fn global_variables(context: &ReferenceContext<'_>) -> BTreeMap<String, String> {
    context
        .variables
        .keys()
        .map(|name| ((*name).to_owned(), "String".to_owned()))
        .collect()
}

fn check_function_branch(
    branch: &IrFunctionBranch,
    variables: &BTreeMap<String, String>,
    capture_candidates: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    match &branch.value {
        IrFunctionBranchValue::Text(text) => {
            check_text(text, variables, capture_candidates, context, errors);
        }
        IrFunctionBranchValue::Dispatch(branches) => {
            for branch in branches {
                check_function_branch(branch, variables, capture_candidates, context, errors);
            }
        }
    }
}

fn check_form_entries(
    owner: &str,
    entries: &[IrFormEntry],
    variables: &BTreeMap<String, String>,
    capture_candidates: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    let direct_branches = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Branch(branch) => Some(branch),
            IrFormEntry::Attribute { .. } => None,
        })
        .collect::<Vec<_>>();
    if !direct_branches.is_empty() {
        validate_dispatch_coverage(
            &format!("form `{owner}`"),
            "Plural",
            direct_branches
                .iter()
                .flat_map(|branch| branch.keys.iter().map(String::as_str)),
            direct_branches.first().map(|branch| branch.span),
            context,
            errors,
        );
    }

    for entry in entries {
        match entry {
            IrFormEntry::Attribute {
                name,
                parameters,
                value,
            } => {
                let mut attribute_variables = variables.clone();
                let mut attribute_capture_candidates = BTreeMap::new();
                for parameter in parameters {
                    if !context.known_type(&parameter.ty) && parameter.ty != "Plural" {
                        errors.push(IrReferenceError::new(
                            "IR009",
                            format!(
                                "unknown parameter type `{}` in form `{}.{name}`",
                                parameter.ty, owner
                            ),
                            value_span(value),
                        ));
                    }
                    if let Some(parameter_name) = &parameter.name {
                        if attribute_variables
                            .insert(parameter_name.clone(), parameter.ty.clone())
                            .is_some()
                        {
                            errors.push(IrReferenceError::new(
                                "IR011",
                                format!(
                                    "form parameter `{parameter_name}` shadows a global variable"
                                ),
                                value_span(value),
                            ));
                        }
                        attribute_capture_candidates
                            .insert(parameter_name.clone(), parameter.ty.clone());
                    }
                }
                validate_value_dispatch_coverage(
                    &format!("form `{}.{name}`", owner),
                    parameters,
                    value,
                    context,
                    errors,
                );
                check_value(
                    value,
                    &attribute_variables,
                    &attribute_capture_candidates,
                    context,
                    errors,
                );
            }
            IrFormEntry::Branch(branch) => {
                check_text(
                    &branch.value,
                    variables,
                    capture_candidates,
                    context,
                    errors,
                );
            }
        }
    }
}

fn validate_value_dispatch_coverage(
    subject: &str,
    parameters: &[crate::IrFunctionParameter],
    value: &IrValue,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    if let IrValue::Map(branches) = value {
        let Some(parameter) = parameters.first().filter(|_| parameters.len() == 1) else {
            errors.push(IrReferenceError::new(
                "IR039",
                format!(
                    "{subject} branch map requires exactly one dispatch parameter, got {}",
                    parameters.len()
                ),
                value_span(value),
            ));
            return;
        };
        validate_dispatch_coverage(
            subject,
            &parameter.ty,
            branches
                .iter()
                .flat_map(|branch| branch.keys.iter().map(String::as_str)),
            value_span(value),
            context,
            errors,
        );
    }
}

fn validate_function_dispatch_coverage(
    function: &IrFunction,
    branches: &[IrFunctionBranch],
    depth: usize,
    context: &ReferenceContext<'_>,
    fallback_span: Option<Span>,
    errors: &mut Vec<IrReferenceError>,
) {
    validate_parameter_dispatch_coverage(
        &format!("function `{}`", function.name),
        &function.parameters,
        branches,
        depth,
        context,
        fallback_span,
        errors,
    );
}

#[allow(clippy::too_many_arguments)]
fn validate_parameter_dispatch_coverage(
    subject: &str,
    parameters: &[crate::IrFunctionParameter],
    branches: &[IrFunctionBranch],
    depth: usize,
    context: &ReferenceContext<'_>,
    fallback_span: Option<Span>,
    errors: &mut Vec<IrReferenceError>,
) {
    let dispatch_types = parameters
        .iter()
        .filter(|parameter| parameter.name.is_none())
        .map(|parameter| parameter.ty.clone())
        .collect::<Vec<_>>();
    validate_typed_dispatch_coverage(
        subject,
        &dispatch_types,
        branches,
        depth,
        context,
        fallback_span,
        errors,
    );
}

fn validate_dispatch_coverage<'a>(
    subject: &str,
    ty: &str,
    keys: impl Iterator<Item = &'a str>,
    span: Option<Span>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    let keys = keys.collect::<BTreeSet<_>>();
    let Ok(resolved) = context.resolve_alias(ty) else {
        return;
    };
    if resolved == "Plural" {
        let known = ["zero", "one", "two", "few", "many", "other", "_"];
        for key in &keys {
            if !known.contains(key) {
                errors.push(IrReferenceError::new(
                    "IR036",
                    format!("{subject} contains unknown `Plural` branch `{key}`"),
                    span,
                ));
            }
        }
        if keys.contains("_") {
            return;
        }
        if !keys.contains("other") {
            errors.push(IrReferenceError::new(
                "IR034",
                format!("{subject} is not exhaustive for `Plural`; add an `other` or `_` branch"),
                span,
            ));
        }
        return;
    }

    if let Some(declaration) = context.enums.get(resolved) {
        for key in &keys {
            if *key != "_" && !declaration.variants.iter().any(|variant| variant == *key) {
                errors.push(IrReferenceError::new(
                    "IR036",
                    format!("{subject} contains unknown enum `{resolved}` branch `{key}`"),
                    span,
                ));
            }
        }
        if keys.contains("_") {
            return;
        }
        for variant in &declaration.variants {
            if !keys.contains(variant.as_str()) {
                errors.push(IrReferenceError::new(
                    "IR034",
                    format!(
                        "{subject} is not exhaustive for enum `{resolved}`; missing branch `{variant}`"
                    ),
                    span,
                ));
            }
        }
        return;
    }

    if keys.contains("_") {
        return;
    }

    errors.push(IrReferenceError::new(
        "IR034",
        format!("{subject} cannot dispatch exhaustively on `{ty}` without a `_` branch"),
        span,
    ));
}

fn check_value(
    value: &IrValue,
    variables: &BTreeMap<String, String>,
    capture_candidates: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    match value {
        IrValue::Text(text) => check_text(text, variables, capture_candidates, context, errors),
        IrValue::Map(branches) => {
            for branch in branches {
                check_text(
                    &branch.value,
                    variables,
                    capture_candidates,
                    context,
                    errors,
                );
            }
        }
        IrValue::Object(entries) => {
            check_form_entries(
                "nested",
                entries,
                variables,
                capture_candidates,
                context,
                errors,
            );
        }
    }
}

fn check_text(
    text: &IrText,
    variables: &BTreeMap<String, String>,
    capture_candidates: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    for part in &text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            let ty = infer_expression(expression, variables, capture_candidates, context, errors);
            validate_formatters(
                &expression.formatters,
                ty.as_deref(),
                Some(expression.span),
                context,
                errors,
            );
        }
    }
}

fn infer_expression(
    expression: &IrExpression,
    variables: &BTreeMap<String, String>,
    capture_candidates: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) -> Option<String> {
    let argument_types = expression
        .arguments
        .iter()
        .map(|argument| infer_expression(argument, variables, capture_candidates, context, errors))
        .collect::<Vec<_>>();

    match &expression.kind {
        IrExpressionKind::Reference if !expression.arguments.is_empty() => {
            errors.push(IrReferenceError::at(
                "IR013",
                "reference expression contains call arguments",
                expression.span,
            ));
            None
        }
        IrExpressionKind::Reference if expression.path.is_empty() => {
            errors.push(IrReferenceError::at(
                "IR012",
                "unresolved empty expression",
                expression.span,
            ));
            None
        }
        IrExpressionKind::Reference => infer_reference(expression, variables, context, errors),
        IrExpressionKind::Call if expression.path.is_empty() => {
            errors.push(IrReferenceError::at(
                "IR012",
                "unresolved empty call expression",
                expression.span,
            ));
            None
        }
        IrExpressionKind::Call => {
            infer_call(expression, &argument_types, variables, context, errors)
        }
        IrExpressionKind::InlineFunction { inputs, branches } => {
            if !expression.path.is_empty() {
                errors.push(IrReferenceError::at(
                    "IR035",
                    "inline fn expression cannot contain a reference path",
                    expression.span,
                ));
            }
            if !expression.arguments.is_empty() {
                errors.push(IrReferenceError::at(
                    "IR035",
                    "inline fn expression cannot contain call arguments",
                    expression.span,
                ));
            }
            if branches.is_empty() {
                errors.push(IrReferenceError::at(
                    "IR035",
                    "inline fn requires at least one branch",
                    expression.span,
                ));
            }

            let (selector_types, branch_variables, branch_capture_candidates) =
                infer_inline_inputs(inputs, variables, capture_candidates, context, errors);
            validate_function_branches("inline fn", branches, errors);
            validate_typed_dispatch_coverage(
                "inline fn",
                &selector_types,
                branches,
                0,
                context,
                Some(expression.span),
                errors,
            );
            for branch in branches {
                check_function_branch(
                    branch,
                    &branch_variables,
                    &branch_capture_candidates,
                    context,
                    errors,
                );
            }
            Some("String".to_owned())
        }
    }
}

fn infer_inline_inputs(
    inputs: &[IrInlineFunctionInput],
    variables: &BTreeMap<String, String>,
    capture_candidates: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) -> (
    Vec<String>,
    BTreeMap<String, String>,
    BTreeMap<String, String>,
) {
    let mut selector_types = Vec::new();
    let mut branch_variables = variables.clone();
    let mut branch_capture_candidates = capture_candidates.clone();
    let mut names = BTreeSet::new();
    let mut saw_binding = false;

    for input in inputs {
        let (value, span) = match input {
            IrInlineFunctionInput::Binding { value, span, .. }
            | IrInlineFunctionInput::Selector { value, span } => (value, *span),
        };
        let inferred = infer_expression(value, variables, capture_candidates, context, errors);
        match input {
            IrInlineFunctionInput::Selector { .. } => {
                if saw_binding {
                    errors.push(IrReferenceError::at(
                        "IR038",
                        "inline fn selectors must precede named payload bindings",
                        span,
                    ));
                }
                selector_types.push(inferred.unwrap_or_else(|| "unknown".to_owned()));
            }
            IrInlineFunctionInput::Binding { name, .. } => {
                saw_binding = true;
                if !names.insert(name.as_str()) {
                    errors.push(IrReferenceError::at(
                        "IR003",
                        format!("duplicate inline fn payload binding `{name}`"),
                        span,
                    ));
                }
                let Some(ty) = inferred else {
                    continue;
                };
                branch_variables.insert(name.clone(), ty.clone());
                branch_capture_candidates.insert(name.clone(), ty);
            }
        }
    }

    (selector_types, branch_variables, branch_capture_candidates)
}

fn validate_typed_dispatch_coverage(
    subject: &str,
    dispatch_types: &[String],
    branches: &[IrFunctionBranch],
    depth: usize,
    context: &ReferenceContext<'_>,
    fallback_span: Option<Span>,
    errors: &mut Vec<IrReferenceError>,
) {
    if dispatch_types.is_empty() {
        for branch in branches {
            if branch.key != "_" {
                errors.push(IrReferenceError::at(
                    "IR038",
                    format!("{subject} without selectors only accepts a `_` branch"),
                    branch.span,
                ));
            }
            if matches!(branch.value, IrFunctionBranchValue::Dispatch(_)) {
                errors.push(IrReferenceError::at(
                    "IR038",
                    format!("{subject} without selectors cannot contain nested dispatch"),
                    branch.span,
                ));
            }
        }
        return;
    }

    if depth >= dispatch_types.len() {
        errors.push(IrReferenceError::new(
            "IR038",
            format!(
                "{subject} branch pattern exceeds its {} selector(s)",
                dispatch_types.len()
            ),
            fallback_span,
        ));
        return;
    }

    let ty = dispatch_types.get(depth).map_or("unknown", String::as_str);
    validate_dispatch_coverage(
        subject,
        ty,
        branches.iter().map(|branch| branch.key.as_str()),
        branches.first().map(|branch| branch.span).or(fallback_span),
        context,
        errors,
    );

    for branch in branches {
        match &branch.value {
            IrFunctionBranchValue::Text(_)
                if depth + 1 < dispatch_types.len() && branch.key != "_" =>
            {
                errors.push(IrReferenceError::at(
                    "IR038",
                    format!(
                        "{subject} branch pattern expects {} selector(s), got {}",
                        dispatch_types.len(),
                        depth + 1
                    ),
                    branch.span,
                ));
            }
            IrFunctionBranchValue::Dispatch(children) => validate_typed_dispatch_coverage(
                subject,
                dispatch_types,
                children,
                depth + 1,
                context,
                Some(branch.span),
                errors,
            ),
            IrFunctionBranchValue::Text(_) => {}
        }
    }
}

fn infer_reference(
    expression: &IrExpression,
    variables: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) -> Option<String> {
    let full_path = expression.path.join(".");
    if let Some(variable_type) = variables.get(&full_path) {
        return Some(variable_type.clone());
    }
    if context.functions.contains_key(full_path.as_str()) || is_plural_intrinsic(&full_path) {
        errors.push(IrReferenceError::at(
            "IR014",
            format!("callable `{full_path}` must be called with parentheses"),
            expression.span,
        ));
        return None;
    }

    let root = &expression.path[0];
    let Some(root_type) = variables.get(root) else {
        errors.push(IrReferenceError::at(
            "IR015",
            format!("unresolved reference `{full_path}`"),
            expression.span,
        ));
        return None;
    };

    if expression.path.len() == 1 {
        return Some(root_type.clone());
    }
    resolve_form_path(
        root_type,
        &expression.path[1..],
        false,
        &[],
        variables,
        context,
        expression.span,
        errors,
    )
}

fn infer_call(
    expression: &IrExpression,
    argument_types: &[Option<String>],
    variables: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) -> Option<String> {
    let full_path = expression.path.join(".");
    if is_plural_intrinsic(&full_path) {
        validate_arity(
            BUILTIN_PLURAL,
            1,
            argument_types.len(),
            expression.span,
            errors,
        );
        if let Some(Some(ty)) = argument_types.first() {
            require_numeric(BUILTIN_PLURAL, ty, expression.span, context, errors);
        }
        return Some("Plural".to_owned());
    }

    if let Some(function) = context.functions.get(full_path.as_str()) {
        validate_arity(
            &function.name,
            function.parameters.len(),
            argument_types.len(),
            expression.span,
            errors,
        );
        for (index, (parameter, actual)) in
            function.parameters.iter().zip(argument_types).enumerate()
        {
            if let Some(actual) = actual {
                require_assignable(
                    &format!("argument {} to `{}`", index + 1, function.name),
                    &parameter.ty,
                    actual,
                    expression.span,
                    context,
                    errors,
                );
            }
        }
        return Some("String".to_owned());
    }

    if expression.path.len() == 1 {
        let name = &expression.path[0];
        if let Some(root_type) = variables.get(name) {
            return resolve_form_path(
                root_type,
                &[],
                true,
                argument_types,
                variables,
                context,
                expression.span,
                errors,
            );
        }

        errors.push(IrReferenceError::at(
            "IR016",
            format!("unknown function `{name}`"),
            expression.span,
        ));
        return None;
    }

    let root = &expression.path[0];
    let Some(root_type) = variables.get(root) else {
        errors.push(IrReferenceError::at(
            "IR015",
            format!("unresolved call target `{}`", expression.path.join(".")),
            expression.span,
        ));
        return None;
    };
    resolve_form_path(
        root_type,
        &expression.path[1..],
        true,
        argument_types,
        variables,
        context,
        expression.span,
        errors,
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_form_path(
    root_type: &str,
    path: &[String],
    called: bool,
    argument_types: &[Option<String>],
    variables: &BTreeMap<String, String>,
    context: &ReferenceContext<'_>,
    span: Span,
    errors: &mut Vec<IrReferenceError>,
) -> Option<String> {
    let resolved_type = context.resolve_alias(root_type).unwrap_or(root_type);
    let Some(form) = context.forms.get(resolved_type) else {
        errors.push(IrReferenceError::at(
            "IR017",
            format!("type `{root_type}` has no form implementation"),
            span,
        ));
        return None;
    };

    if path.is_empty() {
        if called {
            errors.push(IrReferenceError::at(
                "IR018",
                format!("type `{root_type}` has no unnamed callable form"),
                span,
            ));
        }
        return Some(root_type.to_owned());
    }

    let mut kinds = Vec::new();
    for variant in &form.variants {
        if let Some(kind) = form_path_kind(&variant.entries, path, resolved_type, context) {
            kinds.push(kind);
        }
    }
    if kinds.len() != form.variants.len() {
        errors.push(IrReferenceError::at(
            "IR019",
            format!(
                "form property `{}` is not defined for every variant of `{root_type}`",
                path.join(".")
            ),
            span,
        ));
        return None;
    }
    let Some(kind) = kinds.first() else {
        errors.push(IrReferenceError::at(
            "IR019",
            format!(
                "unknown form property `{}` on `{root_type}`",
                path.join(".")
            ),
            span,
        ));
        return None;
    };
    if kinds.iter().any(|candidate| candidate != kind) {
        errors.push(IrReferenceError::at(
            "IR020",
            format!(
                "form property `{}` has inconsistent shapes across `{root_type}` variants",
                path.join(".")
            ),
            span,
        ));
        return None;
    }

    match kind {
        FormPathKind::Text(ty) if called => {
            errors.push(IrReferenceError::at(
                "IR021",
                format!("form property `{}` is not callable", path.join(".")),
                span,
            ));
            Some(ty.clone())
        }
        FormPathKind::Text(ty) => Some(ty.clone()),
        FormPathKind::Map(selectors) if called => {
            validate_arity(
                &path.join("."),
                selectors.len(),
                argument_types.len(),
                span,
                errors,
            );
            for (index, (expected, actual)) in selectors.iter().zip(argument_types).enumerate() {
                if let Some(actual) = actual {
                    require_assignable(
                        &format!("argument {} to form `{}`", index + 1, path.join(".")),
                        expected,
                        actual,
                        span,
                        context,
                        errors,
                    );
                }
            }
            Some("String".to_owned())
        }
        FormPathKind::Map(selectors) => {
            let candidates = variables
                .values()
                .filter(|actual| {
                    selectors.iter().any(|expected| {
                        if expected == "Plural" {
                            is_numeric(actual, context)
                        } else {
                            context.resolve_alias(expected).unwrap_or(expected)
                                == context.resolve_alias(actual).unwrap_or(actual)
                        }
                    })
                })
                .count();
            if candidates == 0 {
                errors.push(IrReferenceError::at(
                    "IR022",
                    format!(
                        "form property `{}` needs explicit dispatch arguments",
                        path.join(".")
                    ),
                    span,
                ));
            } else if candidates > 1 || selectors.len() > 1 {
                errors.push(IrReferenceError::at(
                    "IR023",
                    format!(
                        "form property `{}` has ambiguous implicit dispatch arguments",
                        path.join(".")
                    ),
                    span,
                ));
            }
            Some("String".to_owned())
        }
        FormPathKind::Object => {
            errors.push(IrReferenceError::at(
                "IR024",
                format!("form object `{}` is not a value", path.join(".")),
                span,
            ));
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FormPathKind {
    Text(String),
    Map(Vec<String>),
    Object,
}

fn form_path_kind(
    entries: &[IrFormEntry],
    path: &[String],
    owner_type: &str,
    context: &ReferenceContext<'_>,
) -> Option<FormPathKind> {
    let (segment, rest) = path.split_first()?;
    let (parameters, value) = entries.iter().find_map(|entry| match entry {
        IrFormEntry::Attribute {
            name,
            parameters,
            value,
        } if name == segment => Some((parameters, value)),
        IrFormEntry::Attribute { .. } | IrFormEntry::Branch(_) => None,
    })?;
    if rest.is_empty() {
        return Some(match value {
            IrValue::Text(_) => {
                if let Some(ty) = relative_enum_type(segment, owner_type, context) {
                    FormPathKind::Text(ty)
                } else {
                    FormPathKind::Text("String".to_owned())
                }
            }
            IrValue::Map(_) => FormPathKind::Map(
                parameters
                    .iter()
                    .map(|parameter| parameter.ty.clone())
                    .collect(),
            ),
            IrValue::Object(_) => FormPathKind::Object,
        });
    }
    match value {
        IrValue::Object(children) => form_path_kind(children, rest, owner_type, context),
        IrValue::Text(_) | IrValue::Map(_) => None,
    }
}

fn relative_enum_type(
    name: &str,
    owner_type: &str,
    context: &ReferenceContext<'_>,
) -> Option<String> {
    let candidates = std::iter::once(name.to_owned()).chain(
        owner_type
            .rsplit_once('.')
            .map(|(namespace, _)| format!("{namespace}.{name}")),
    );
    candidates.into_iter().find(|candidate| {
        context
            .resolve_alias(candidate)
            .is_ok_and(|resolved| context.enums.contains_key(resolved))
    })
}

fn validate_arity(
    name: &str,
    expected: usize,
    actual: usize,
    span: Span,
    errors: &mut Vec<IrReferenceError>,
) {
    if expected != actual {
        errors.push(IrReferenceError::at(
            "IR025",
            format!("`{name}` expects {expected} argument(s), got {actual}"),
            span,
        ));
    }
}

fn require_numeric(
    subject: &str,
    actual: &str,
    span: Span,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    if !is_numeric(actual, context) {
        errors.push(IrReferenceError::at(
            "IR026",
            format!("`{subject}` expects Number or Decimal, got `{actual}`"),
            span,
        ));
    }
}

fn require_assignable(
    subject: &str,
    expected: &str,
    actual: &str,
    span: Span,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    if expected == "Plural" {
        require_numeric(subject, actual, span, context, errors);
        return;
    }
    let expected = context.resolve_alias(expected).unwrap_or(expected);
    let actual = context.resolve_alias(actual).unwrap_or(actual);
    if expected != actual
        && !(matches!(expected, "Number" | "Decimal") && matches!(actual, "Number" | "Decimal"))
    {
        errors.push(IrReferenceError::at(
            "IR027",
            format!("type mismatch for {subject}: expected `{expected}`, got `{actual}`"),
            span,
        ));
    }
}

fn is_numeric(ty: &str, context: &ReferenceContext<'_>) -> bool {
    matches!(
        context.resolve_alias(ty).unwrap_or(ty),
        "Number" | "Decimal"
    )
}

fn validate_formatters(
    formatters: &[crate::IrFormatter],
    value_type: Option<&str>,
    span: Option<Span>,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    for formatter in formatters {
        let (allowed, expected_type): (&[&str], Option<&str>) = match &formatter.kind {
            FormatterKind::Number => (&[], Some("numeric")),
            FormatterKind::Currency => (&["code", "accounting"], Some("numeric")),
            FormatterKind::Date => (&["style"], Some("Date")),
            FormatterKind::Unknown(name) => {
                errors.push(IrReferenceError::new(
                    "IR028",
                    format!("unknown formatter `{name}`"),
                    span,
                ));
                continue;
            }
        };

        let mut names = BTreeSet::new();
        for argument in &formatter.arguments {
            if !names.insert(argument.name.as_str()) {
                errors.push(IrReferenceError::new(
                    "IR029",
                    format!("duplicate formatter option `{}`", argument.name),
                    span,
                ));
            }
            if !allowed.contains(&argument.name.as_str()) {
                errors.push(IrReferenceError::new(
                    "IR030",
                    format!(
                        "formatter `{}` does not support option `{}`",
                        formatter.kind.as_str(),
                        argument.name
                    ),
                    span,
                ));
            }
            match (formatter.kind.as_str(), argument.name.as_str()) {
                ("currency", "code")
                    if argument.value.len() != 3
                        || !argument
                            .value
                            .chars()
                            .all(|value| value.is_ascii_alphabetic()) =>
                {
                    errors.push(IrReferenceError::new(
                        "IR031",
                        "currency formatter option `code` must be a three-letter currency code",
                        span,
                    ));
                }
                ("currency", "accounting")
                    if !matches!(argument.value.as_str(), "true" | "false") =>
                {
                    errors.push(IrReferenceError::new(
                        "IR031",
                        "currency formatter option `accounting` must be `true` or `false`",
                        span,
                    ));
                }
                ("date", "style")
                    if !matches!(
                        argument.value.as_str(),
                        "full" | "long" | "medium" | "short"
                    ) =>
                {
                    errors.push(IrReferenceError::new(
                        "IR031",
                        "date formatter option `style` must be full, long, medium, or short",
                        span,
                    ));
                }
                _ => {}
            }
        }

        if let (Some(expected), Some(actual)) = (expected_type, value_type) {
            let valid = if expected == "numeric" {
                is_numeric(actual, context)
            } else {
                context.resolve_alias(actual).unwrap_or(actual) == expected
            };
            if !valid {
                errors.push(IrReferenceError::new(
                    "IR032",
                    format!(
                        "formatter `{}` cannot format value of type `{actual}`",
                        formatter.kind.as_str()
                    ),
                    span,
                ));
            }
        }
    }
}

fn validate_reference_cycles(
    locale: &IrModule,
    context: &ReferenceContext<'_>,
    errors: &mut Vec<IrReferenceError>,
) {
    let mut graph = BTreeMap::<String, Vec<(String, Span)>>::new();
    for variable in &locale.variables {
        let edges = graph.entry(variable.name.clone()).or_default();
        collect_text_edges(&variable.value, context, edges);
    }
    for function in &locale.functions {
        let edges = graph.entry(function.name.clone()).or_default();
        let local_names = function
            .parameters
            .iter()
            .filter_map(|parameter| parameter.name.clone())
            .collect::<BTreeSet<_>>();
        for branch in &function.branches {
            collect_function_edges_scoped(branch, context, &local_names, edges);
        }
    }

    let mut visited = BTreeSet::new();
    let mut order = Vec::new();
    for node in graph.keys() {
        finish_order(node, &graph, &mut visited, &mut order);
    }
    let reverse = reverse_graph(&graph);
    visited.clear();
    while let Some(node) = order.pop() {
        if visited.contains(&node) {
            continue;
        }
        let mut component = Vec::new();
        collect_component(&node, &reverse, &mut visited, &mut component);
        component.sort();
        let component_set = component
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let self_cycle = component.len() == 1
            && graph
                .get(&component[0])
                .is_some_and(|edges| edges.iter().any(|(target, _)| target == &component[0]));
        if component.len() <= 1 && !self_cycle {
            continue;
        }

        let primary = component.first().and_then(|name| origin_span(locale, name));
        let mut error = IrReferenceError::new(
            "IR033",
            format!("cyclic reference component `{}`", component.join(" -> ")),
            primary,
        );
        for source in &component {
            if let Some(edges) = graph.get(source) {
                for (target, span) in edges {
                    if component_set.contains(target.as_str()) {
                        error = error.related(*span, format!("`{source}` references `{target}`"));
                    }
                }
            }
        }
        errors.push(error);
    }
}

fn collect_function_edges_scoped(
    branch: &IrFunctionBranch,
    context: &ReferenceContext<'_>,
    local_names: &BTreeSet<String>,
    edges: &mut Vec<(String, Span)>,
) {
    match &branch.value {
        IrFunctionBranchValue::Text(text) => {
            collect_text_edges_scoped(text, context, local_names, edges);
        }
        IrFunctionBranchValue::Dispatch(children) => {
            for child in children {
                collect_function_edges_scoped(child, context, local_names, edges);
            }
        }
    }
}

fn collect_text_edges(
    text: &IrText,
    context: &ReferenceContext<'_>,
    edges: &mut Vec<(String, Span)>,
) {
    collect_text_edges_scoped(text, context, &BTreeSet::new(), edges);
}

fn collect_text_edges_scoped(
    text: &IrText,
    context: &ReferenceContext<'_>,
    local_names: &BTreeSet<String>,
    edges: &mut Vec<(String, Span)>,
) {
    for part in &text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            collect_expression_edges(expression, context, local_names, edges);
        }
    }
}

fn collect_expression_edges(
    expression: &IrExpression,
    context: &ReferenceContext<'_>,
    local_names: &BTreeSet<String>,
    edges: &mut Vec<(String, Span)>,
) {
    if let Some(root) = expression.path.first() {
        if !local_names.contains(root)
            && (context.variables.contains_key(root.as_str())
                || context.functions.contains_key(root.as_str()))
        {
            edges.push((root.clone(), expression.span));
        }
    }
    for argument in &expression.arguments {
        collect_expression_edges(argument, context, local_names, edges);
    }
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        for input in inputs {
            let value = match input {
                IrInlineFunctionInput::Binding { value, .. }
                | IrInlineFunctionInput::Selector { value, .. } => value,
            };
            collect_expression_edges(value, context, local_names, edges);
        }
        let mut inline_names = local_names.clone();
        inline_names.extend(inputs.iter().filter_map(|input| match input {
            IrInlineFunctionInput::Binding { name, .. } => Some(name.clone()),
            IrInlineFunctionInput::Selector { .. } => None,
        }));
        for branch in branches {
            collect_function_edges_scoped(branch, context, &inline_names, edges);
        }
    }
}

fn finish_order(
    node: &str,
    graph: &BTreeMap<String, Vec<(String, Span)>>,
    visited: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) {
    if !visited.insert(node.to_owned()) {
        return;
    }
    if let Some(edges) = graph.get(node) {
        for (target, _) in edges {
            if graph.contains_key(target) {
                finish_order(target, graph, visited, order);
            }
        }
    }
    order.push(node.to_owned());
}

fn reverse_graph(graph: &BTreeMap<String, Vec<(String, Span)>>) -> BTreeMap<String, Vec<String>> {
    let mut reverse = graph
        .keys()
        .map(|node| (node.clone(), Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for (source, edges) in graph {
        for (target, _) in edges {
            if let Some(incoming) = reverse.get_mut(target) {
                incoming.push(source.clone());
            }
        }
    }
    reverse
}

fn collect_component(
    node: &str,
    graph: &BTreeMap<String, Vec<String>>,
    visited: &mut BTreeSet<String>,
    output: &mut Vec<String>,
) {
    if !visited.insert(node.to_owned()) {
        return;
    }
    output.push(node.to_owned());
    if let Some(edges) = graph.get(node) {
        for target in edges {
            collect_component(target, graph, visited, output);
        }
    }
}

fn origin_span(module: &IrModule, name: &str) -> Option<Span> {
    module
        .origins
        .iter()
        .rev()
        .find(|origin| origin.name == name)
        .map(|origin| origin.span)
}
