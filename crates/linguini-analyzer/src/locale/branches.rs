use crate::{
    analyze_branch_coverage, require_other_branch, BranchCoverage, Diagnostic, NamedSpan, QuickFix,
    Replacement,
};
use linguini_syntax::{
    FormAttribute, FormDeclaration, FormEntry, FunctionBranch, FunctionBranchValue,
    LocaleDeclaration, LocaleFile, LocaleValue, MapBranch, SchemaDeclaration, SchemaFile, Span,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn analyze_locale_branch_coverage(
    schema: Option<&SchemaFile>,
    locale: &LocaleFile,
) -> Vec<Diagnostic> {
    let mut enum_variants = schema.map(schema_enum_variants).unwrap_or_default();
    for declaration in &locale.declarations {
        collect_locale_enum_variants(declaration, &mut enum_variants);
    }

    let mut diagnostics = Vec::new();
    for declaration in &locale.declarations {
        collect_branch_coverage_diagnostics(declaration, &enum_variants, &mut diagnostics);
    }
    diagnostics
}

fn schema_enum_variants(schema: &SchemaFile) -> BTreeMap<String, Vec<NamedSpan>> {
    schema
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
        .collect()
}

fn collect_locale_enum_variants(
    declaration: &LocaleDeclaration,
    enum_variants: &mut BTreeMap<String, Vec<NamedSpan>>,
) {
    match declaration {
        LocaleDeclaration::Enum(item) => {
            enum_variants.insert(
                item.name.value.clone(),
                item.variants
                    .iter()
                    .map(|variant| NamedSpan::new(&variant.value, variant.span))
                    .collect(),
            );
        }
        LocaleDeclaration::Override(inner) => collect_locale_enum_variants(inner, enum_variants),
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
            let dispatch_types = function
                .parameters
                .iter()
                .filter_map(|parameter| {
                    (parameter.ty.value != "String").then_some(parameter.ty.value.as_str())
                })
                .collect::<Vec<_>>();
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

fn validate_impl_variants(
    form: &FormDeclaration,
    enum_variants: &BTreeMap<String, Vec<NamedSpan>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(variants) = enum_variants.get(&form.name.value) else {
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
    if branches.iter().any(|branch| branch.name == "_") {
        return Vec::new();
    }

    let branch_names = branches
        .iter()
        .map(|branch| branch.name.as_str())
        .collect::<BTreeSet<_>>();
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
            .with_related(variant.span, "enum variant is declared here");

            match impl_variant_insertion(form, &variant.name) {
                Some(replacement) => diagnostic.with_quick_fix(QuickFix::replacement(
                    format!("add variant `{}`", variant.name),
                    replacement,
                )),
                None => diagnostic,
            }
        })
        .collect()
}

fn impl_variant_insertion(form: &FormDeclaration, variant_name: &str) -> Option<Replacement> {
    let last_variant = form.variants.last()?;
    Some(Replacement {
        span: Span::new(last_variant.span.end, last_variant.span.end),
        text: format!("\n\n  {variant_name} {{\n  }}"),
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
