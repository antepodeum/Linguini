use crate::{
    analyze_branch_coverage, require_other_branch, BranchCoverage, Diagnostic, NamedSpan, QuickFix,
    Replacement,
};
use linguini_syntax::{
    FormAttribute, FormDeclaration, FormEntry, FunctionBranch, FunctionBranchValue, FunctionKind,
    LocaleDeclaration, LocaleFile, LocaleValue, MapBranch, SchemaDeclaration, SchemaFile, Span,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn analyze_locale_branch_coverage(
    schema: Option<&SchemaFile>,
    locale: &LocaleFile,
) -> Vec<Diagnostic> {
    let schemas = schema.map(std::slice::from_ref).unwrap_or_default();
    analyze_locale_branch_coverage_from_files(schemas, locale)
}

pub(super) fn analyze_locale_branch_coverage_from_files(
    schemas: &[SchemaFile],
    locale: &LocaleFile,
) -> Vec<Diagnostic> {
    let mut enum_variants = schema_enum_variants(schemas);
    let mut diagnostics = Vec::new();
    for declaration in locale.declarations() {
        collect_locale_enum_variants(declaration, &mut enum_variants, &mut diagnostics);
    }
    for declaration in locale.declarations() {
        collect_branch_coverage_diagnostics(declaration, &enum_variants, &mut diagnostics);
    }
    diagnostics
}

fn schema_enum_variants(schemas: &[SchemaFile]) -> BTreeMap<String, Vec<NamedSpan>> {
    schemas
        .iter()
        .flat_map(|schema| schema.declarations())
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
        .collect()
}

fn collect_locale_enum_variants(
    declaration: &LocaleDeclaration,
    enum_variants: &mut BTreeMap<String, Vec<NamedSpan>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match declaration {
        LocaleDeclaration::Enum(item) => {
            if enum_variants.contains_key(&item.name.value) {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "locale enum `{}` conflicts with an existing enum",
                            item.name.value
                        ),
                        item.name.span,
                    )
                    .with_code("linguini.duplicate_enum"),
                );
            } else {
                enum_variants.insert(
                    item.name.value.clone(),
                    item.variants
                        .iter()
                        .map(|variant| NamedSpan::new(&variant.value, variant.span))
                        .collect(),
                );
            }
        }
        LocaleDeclaration::Override(inner) => {
            collect_locale_enum_variants(inner, enum_variants, diagnostics);
        }
        LocaleDeclaration::Form(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Function(_)
        | LocaleDeclaration::Message(_)
        | LocaleDeclaration::Group(_) => {}
    }
}

fn collect_branch_coverage_diagnostics(
    declaration: &LocaleDeclaration,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match declaration {
        LocaleDeclaration::Form(form) => validate_impl_variants(form, enum_variants, diagnostics),
        LocaleDeclaration::Function(function) => {
            if function.kind == FunctionKind::Function {
                validate_named_function_parameter_order(
                    &function.name.value,
                    &function.parameters,
                    diagnostics,
                );
            }
            let dispatch_types = function
                .parameters
                .iter()
                .filter_map(|parameter| {
                    (parameter.name.is_none()
                        && !matches!(parameter.ty.value.as_str(), "Date" | "Boolean" | "String"))
                    .then_some(parameter.ty.value.as_str())
                })
                .collect::<Vec<_>>();
            for parameter in &function.parameters {
                if parameter.name.is_none()
                    && matches!(parameter.ty.value.as_str(), "Date" | "Boolean" | "String")
                {
                    diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "type `{}` cannot be used as a dispatch dimension",
                                parameter.ty.value
                            ),
                            parameter.ty.span,
                        )
                        .with_code("linguini.invalid_dispatch_type"),
                    );
                }
            }
            validate_param_order(
                &function.name.value,
                &dispatch_types,
                enum_variants,
                function.span,
                diagnostics,
            );
            if function.kind == FunctionKind::Function
                && !function
                    .parameters
                    .iter()
                    .any(|parameter| parameter.name.is_some() && parameter.ty.value == "String")
            {
                diagnostics.push(
                    Diagnostic::warning(
                        format!(
                            "fn `{}` has no named String parameter; use `form`",
                            function.name.value
                        ),
                        function.name.span,
                    )
                    .as_lint("fn_without_strings"),
                );
            }
            validate_dispatch_branches(
                &function.name.value,
                &function.branches,
                &dispatch_types,
                0,
                enum_variants,
                diagnostics,
            );
        }
        LocaleDeclaration::Override(inner) => {
            collect_branch_coverage_diagnostics(inner, enum_variants, diagnostics);
        }
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Message(_)
        | LocaleDeclaration::Group(_) => {}
    }
}

fn validate_named_function_parameter_order(
    function_name: &str,
    parameters: &[linguini_syntax::FunctionParameter],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut first_payload = None;
    for parameter in parameters {
        if parameter.name.is_some() {
            first_payload.get_or_insert(parameter.span);
        } else if let Some(payload_span) = first_payload {
            diagnostics.push(
                Diagnostic::error(
                    format!(
                        "function `{function_name}` must place unnamed selectors before named payload parameters"
                    ),
                    parameter.span,
                )
                .with_code("linguini.parameter_order")
                .with_related(payload_span, "first named payload parameter is here"),
            );
            return;
        }
    }
}

fn validate_impl_variants(
    form: &FormDeclaration,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(variants) = enum_variants.get(&form.name.value) else {
        diagnostics.push(
            Diagnostic::error(
                format!("impl targets unknown enum `{}`", form.name.value),
                form.name.span,
            )
            .with_code("linguini.unknown_impl_target"),
        );
        return;
    };
    let branches = form
        .variants
        .iter()
        .map(|variant| NamedSpan::new(&variant.name.value, variant.name.span))
        .collect::<Vec<_>>();

    diagnostics.extend(analyze_impl_coverage(
        &form.name.value,
        variants,
        &branches,
        form,
    ));

    for variant in &form.variants {
        validate_form_entries(
            &format!(
                "impl `{}` variant `{}`",
                form.name.value, variant.name.value
            ),
            &variant.entries,
            diagnostics,
        );
    }
}

fn analyze_impl_coverage(
    enum_name: &str,
    variants: &[NamedSpan],
    branches: &[NamedSpan],
    form: &FormDeclaration,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let variant_names = variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeMap::new();
    let wildcard = branches.iter().position(|branch| branch.name == "_");
    for (index, branch) in branches.iter().enumerate() {
        if let Some(first) = seen.insert(branch.name.as_str(), branch.span) {
            diagnostics.push(
                Diagnostic::error(
                    format!("duplicate impl variant `{}`", branch.name),
                    branch.span,
                )
                .with_code("linguini.duplicate_branch")
                .with_related(first, "first variant implementation is here"),
            );
        }
        if branch.name != "_" && !variant_names.contains(branch.name.as_str()) {
            diagnostics.push(
                Diagnostic::error(
                    format!("impl `{enum_name}` uses unknown variant `{}`", branch.name),
                    branch.span,
                )
                .with_code("linguini.unknown_enum_variant"),
            );
        }
        if wildcard.is_some_and(|wildcard| index > wildcard) {
            diagnostics.push(
                Diagnostic::warning(
                    format!("impl variant `{}` is unreachable after `_`", branch.name),
                    branch.span,
                )
                .as_lint("unreachable_arm"),
            );
        }
    }
    if let Some(wildcard) = wildcard {
        if wildcard + 1 != branches.len() {
            diagnostics.push(
                Diagnostic::error("wildcard `_` must be last", branches[wildcard].span)
                    .with_code("linguini.wildcard_order"),
            );
        }
        return diagnostics;
    }
    let branch_names = branches
        .iter()
        .map(|branch| branch.name.as_str())
        .collect::<BTreeSet<_>>();
    diagnostics.extend(
        variants
            .iter()
            .filter(|variant| !branch_names.contains(variant.name.as_str()))
            .map(|variant| {
                let diagnostic = Diagnostic::error(
                    format!(
                        "impl `{enum_name}` for enum `{enum_name}` is missing variant `{}`",
                        variant.name
                    ),
                    form.span,
                )
                .as_lint("incomplete_impl")
                .with_related(variant.span, "enum variant is declared here");

                match impl_variant_insertion(form, &variant.name) {
                    Some(replacement) => diagnostic.with_quick_fix(QuickFix::replacement(
                        format!("add variant `{}`", variant.name),
                        replacement,
                    )),
                    None => diagnostic,
                }
            })
            .collect::<Vec<_>>(),
    );
    diagnostics
}

fn impl_variant_insertion(form: &FormDeclaration, variant_name: &str) -> Option<Replacement> {
    let last_variant = form.variants.last()?;
    Some(Replacement {
        span: Span::new(last_variant.span.end, last_variant.span.end),
        text: format!("\n\n  {variant_name} {{\n    TODO = TODO\n  }}"),
    })
}

fn validate_dispatch_branches(
    function_name: &str,
    branches: &[FunctionBranch],
    dispatch_types: &[&str],
    depth: usize,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(dispatch_type) = dispatch_types.get(depth) else {
        return;
    };
    let branch_spans = branches
        .iter()
        .map(|branch| NamedSpan::new(&branch.key.value, branch.span))
        .collect::<Vec<_>>();
    let span = branch_list_span(branches);
    let subject = format!("function `{function_name}`");
    validate_collapsible_arms(function_name, branches, diagnostics);

    if *dispatch_type == "Plural" {
        diagnostics.extend(require_other_branch(&subject, &branch_spans, span));
    } else if let Some(variants) = enum_variants.get(*dispatch_type) {
        diagnostics.extend(analyze_branch_coverage(BranchCoverage {
            subject: &subject,
            enum_name: dispatch_type,
            variants: variants.clone(),
            branches: branch_spans,
            span,
        }));
    } else {
        diagnostics.push(
            Diagnostic::error(
                format!(
                    "function `{function_name}` dispatches on unknown enum or category `{dispatch_type}`"
                ),
                span,
            )
            .with_code("linguini.unknown_dispatch_type"),
        );
    }

    for branch in branches {
        if let FunctionBranchValue::Dispatch(children) = &branch.value {
            validate_dispatch_branches(
                function_name,
                children,
                dispatch_types,
                depth + 1,
                enum_variants,
                diagnostics,
            );
        }
    }
}

fn validate_param_order(
    function_name: &str,
    dispatch_types: &[&str],
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let cardinality = |ty: &str| {
        if ty == "Plural" {
            Some(6)
        } else {
            enum_variants.get(ty).map(Vec::len)
        }
    };
    if dispatch_types.windows(2).any(|pair| {
        matches!(
            (cardinality(pair[0]), cardinality(pair[1])),
            (Some(left), Some(right)) if left > right
        )
    }) {
        diagnostics.push(
            Diagnostic::warning(
                format!(
                    "function `{function_name}` dispatch parameters should be ordered from fewest variants to most"
                ),
                span,
            )
            .as_lint("param_order"),
        );
    }
}

fn validate_collapsible_arms(
    function_name: &str,
    branches: &[FunctionBranch],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (index, branch) in branches.iter().enumerate() {
        if let Some(first) = branches[..index]
            .iter()
            .find(|candidate| branch_values_equivalent(&candidate.value, &branch.value))
        {
            diagnostics.push(
                Diagnostic::warning(
                    format!(
                        "function `{function_name}` branches `{}` and `{}` have identical output",
                        first.key.value, branch.key.value
                    ),
                    branch.span,
                )
                .as_lint("collapsible_arms")
                .with_related(first.span, "identical branch is here"),
            );
        }
    }
}

fn branch_values_equivalent(left: &FunctionBranchValue, right: &FunctionBranchValue) -> bool {
    match (left, right) {
        (FunctionBranchValue::Text(left), FunctionBranchValue::Text(right)) => {
            text_patterns_equivalent(left, right)
        }
        (FunctionBranchValue::Dispatch(left), FunctionBranchValue::Dispatch(right)) => {
            left.len() == right.len()
                && left.iter().zip(right).all(|(left, right)| {
                    left.key.value == right.key.value
                        && branch_values_equivalent(&left.value, &right.value)
                })
        }
        (FunctionBranchValue::Text(_), FunctionBranchValue::Dispatch(_))
        | (FunctionBranchValue::Dispatch(_), FunctionBranchValue::Text(_)) => false,
    }
}

fn text_patterns_equivalent(
    left: &linguini_syntax::TextPattern,
    right: &linguini_syntax::TextPattern,
) -> bool {
    left.mode == right.mode
        && left.parts.len() == right.parts.len()
        && left.parts.iter().zip(&right.parts).all(|(left, right)| {
            match (left, right) {
                (
                    linguini_syntax::TextPart::Text(left),
                    linguini_syntax::TextPart::Text(right),
                ) => left.value == right.value,
                (
                    linguini_syntax::TextPart::Placeholder(left),
                    linguini_syntax::TextPart::Placeholder(right),
                ) => expressions_equivalent(&left.expression, &right.expression),
                (
                    linguini_syntax::TextPart::Text(_),
                    linguini_syntax::TextPart::Placeholder(_),
                )
                | (
                    linguini_syntax::TextPart::Placeholder(_),
                    linguini_syntax::TextPart::Text(_),
                ) => false,
            }
        })
}

fn expressions_equivalent(
    left: &linguini_syntax::Expression,
    right: &linguini_syntax::Expression,
) -> bool {
    left.kind == right.kind
        && left
            .path
            .iter()
            .map(|name| &name.value)
            .eq(right.path.iter().map(|name| &name.value))
        && left.arguments.len() == right.arguments.len()
        && left
            .arguments
            .iter()
            .zip(&right.arguments)
            .all(|(left, right)| expressions_equivalent(left, right))
        && left.annotations.len() == right.annotations.len()
        && left
            .annotations
            .iter()
            .zip(&right.annotations)
            .all(|(left, right)| {
                left.kind == right.kind
                    && left
                        .arguments
                        .iter()
                        .map(|argument| (&argument.name.value, &argument.value.value))
                        .eq(right
                            .arguments
                            .iter()
                            .map(|argument| (&argument.name.value, &argument.value.value)))
            })
}

fn validate_form_entries(subject: &str, entries: &[FormEntry], diagnostics: &mut Vec<Diagnostic>) {
    let branches = entries
        .iter()
        .filter_map(|entry| match entry {
            FormEntry::Branch(branch) => Some(branch.clone()),
            FormEntry::Attribute(_) => None,
        })
        .collect::<Vec<_>>();
    if !branches.is_empty() {
        analyze_map_branches(subject, &branches, diagnostics);
    }

    for entry in entries {
        if let FormEntry::Attribute(attribute) = entry {
            validate_form_attribute(subject, attribute, diagnostics);
        }
    }
}

fn validate_form_attribute(
    subject: &str,
    attribute: &FormAttribute,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match &attribute.value {
        LocaleValue::Text(_) => {}
        LocaleValue::Map(branches) => {
            analyze_map_branches(
                &format!("{subject} form `{}`", attribute.name.value),
                branches,
                diagnostics,
            );
        }
        LocaleValue::Object(entries) => {
            validate_form_entries(
                &format!("{subject} attribute `{}`", attribute.name.value),
                entries,
                diagnostics,
            );
        }
    }
}

fn analyze_map_branches(subject: &str, branches: &[MapBranch], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = BTreeMap::<&str, Span>::new();
    for branch in branches {
        for key in &branch.keys {
            if let Some(first) = seen.insert(key.value.as_str(), key.span) {
                diagnostics.push(
                    Diagnostic::error(
                        format!("duplicate map key `{}` in {subject}", key.value),
                        key.span,
                    )
                    .with_code("linguini.duplicate_branch")
                    .with_related(first, "first map key is here"),
                );
            }
        }
    }
    let branch_spans = branches
        .iter()
        .filter_map(|branch| {
            let key = branch.keys.first()?;
            Some(NamedSpan::new(&key.value, branch.span))
        })
        .collect::<Vec<_>>();
    if branch_spans.is_empty() {
        return;
    }
    diagnostics.extend(require_other_branch(
        subject,
        &branch_spans,
        branch_list_span_for_map(branches),
    ));
}

fn branch_list_span_for_map(branches: &[MapBranch]) -> Span {
    let Some(first) = branches.first() else {
        return Span::new(0, 0);
    };
    let end = branches
        .last()
        .map(|branch| branch.span.end)
        .unwrap_or(first.span.end);
    Span::new(first.span.start, end)
}

fn branch_list_span(branches: &[FunctionBranch]) -> Span {
    let Some(first) = branches.first() else {
        return Span::new(0, 0);
    };
    let end = branches
        .last()
        .map(|branch| branch.span.end)
        .unwrap_or(first.span.end);
    Span::new(first.span.start, end)
}
