use crate::{
    analyze_branch_coverage, require_other_branch, BranchCoverage, Diagnostic, DiagnosticSeverity,
    NamedSpan, QuickFix, Replacement,
};
use linguini_syntax::{
    DocComment, FunctionBranch, FunctionBranchValue, LocaleDeclaration, LocaleFile,
    SchemaDeclaration, SchemaFile, Span,
};
use std::collections::BTreeMap;

mod messages;

use self::messages::{
    format_name_list, locale_message_map, missing_message_stub_text, pluralize,
    schema_message_map,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredLocaleMessage {
    pub name: String,
    pub span: Span,
    pub docs: Vec<String>,
}

impl RequiredLocaleMessage {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
            docs: Vec::new(),
        }
    }

    pub fn with_docs(mut self, docs: &[DocComment]) -> Self {
        self.docs = docs.iter().map(|doc| doc.text.trim().to_owned()).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementedLocaleMessage {
    pub name: String,
    pub span: Span,
    pub docs: Vec<String>,
}

impl ImplementedLocaleMessage {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
            docs: Vec::new(),
        }
    }

    pub fn with_docs(mut self, docs: &[DocComment]) -> Self {
        self.docs = docs.iter().map(|doc| doc.text.trim().to_owned()).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleCoverageOptions {
    pub missing_message_severity: DiagnosticSeverity,
    pub subject: String,
    pub quick_fix_id: Option<String>,
}

impl Default for LocaleCoverageOptions {
    fn default() -> Self {
        Self {
            missing_message_severity: DiagnosticSeverity::Error,
            subject: "locale".to_owned(),
            quick_fix_id: None,
        }
    }
}

pub fn analyze_locale_file(locale: &LocaleFile) -> Vec<Diagnostic> {
    analyze_locale_branch_coverage(&BTreeMap::new(), locale)
}

pub fn analyze_locale_coverage(schema: &SchemaFile, locale: &LocaleFile) -> Vec<Diagnostic> {
    analyze_locale_coverage_with_options(schema, locale, LocaleCoverageOptions::default())
}

pub fn analyze_locale_coverage_with_options(
    schema: &SchemaFile,
    locale: &LocaleFile,
    options: LocaleCoverageOptions,
) -> Vec<Diagnostic> {
    let mut diagnostics = analyze_locale_message_coverage_with_options(
        &schema_public_messages(schema),
        &locale_public_messages(locale),
        locale.span,
        options,
    );
    diagnostics.extend(analyze_locale_branch_coverage(
        &schema_enum_variants(schema),
        locale,
    ));
    diagnostics
}

pub fn analyze_locale_message_coverage(
    schema_messages: &[RequiredLocaleMessage],
    locale_messages: &[ImplementedLocaleMessage],
    locale_span: Span,
) -> Vec<Diagnostic> {
    analyze_locale_message_coverage_with_options(
        schema_messages,
        locale_messages,
        locale_span,
        LocaleCoverageOptions::default(),
    )
}

pub fn analyze_locale_message_coverage_with_options(
    schema_messages: &[RequiredLocaleMessage],
    locale_messages: &[ImplementedLocaleMessage],
    locale_span: Span,
    options: LocaleCoverageOptions,
) -> Vec<Diagnostic> {
    let schema = schema_message_map(schema_messages);
    let locale = locale_message_map(locale_messages);
    let missing = schema_messages
        .iter()
        .filter(|schema_message| !locale.contains_key(schema_message.name.as_str()))
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();

    if !missing.is_empty() {
        diagnostics.push(missing_messages_diagnostic(
            &missing,
            locale_span,
            options.missing_message_severity,
            &options.subject,
            options.quick_fix_id.as_deref(),
        ));
    }

    let unknown = locale_messages
        .iter()
        .filter(|locale_message| !schema.contains_key(locale_message.name.as_str()))
        .collect::<Vec<_>>();

    if !unknown.is_empty() {
        diagnostics.push(unknown_messages_diagnostic(&unknown));
    }

    diagnostics.extend(missing_doc_comment_diagnostics(&schema, &locale));

    diagnostics
}

pub fn schema_public_messages(schema: &SchemaFile) -> Vec<RequiredLocaleMessage> {
    let mut messages = Vec::new();
    for declaration in &schema.declarations {
        collect_schema_messages(declaration, None, &mut messages);
    }
    messages
}

pub fn locale_public_messages(locale: &LocaleFile) -> Vec<ImplementedLocaleMessage> {
    let mut messages = Vec::new();
    for declaration in &locale.declarations {
        collect_locale_messages(declaration, None, &mut messages);
    }
    messages
}

fn missing_messages_diagnostic(
    missing: &[&RequiredLocaleMessage],
    locale_span: Span,
    severity: DiagnosticSeverity,
    subject: &str,
    quick_fix_id: Option<&str>,
) -> Diagnostic {
    let names = missing
        .iter()
        .map(|message| message.name.as_str())
        .collect::<Vec<_>>();
    let message = format!(
        "{subject} is missing {} schema {}: {}",
        names.len(),
        pluralize(names.len(), "message", "messages"),
        format_name_list(&names),
    );
    let diagnostic = match severity {
        DiagnosticSeverity::Error => Diagnostic::error(message, Span::new(0, 0)),
        DiagnosticSeverity::Warning => Diagnostic::warning(message, Span::new(0, 0)),
        DiagnosticSeverity::Advice => Diagnostic::advice(message, Span::new(0, 0)),
    }
    .without_source()
    .with_note("add implementations for the missing schema messages");

    let quick_fix = QuickFix::replacement(
        "add missing locale message stubs",
        Replacement {
            span: Span::new(locale_span.end, locale_span.end),
            text: missing_message_stub_text(&names),
        },
    );

    let mut diagnostic = match quick_fix_id {
        Some(id) => diagnostic.with_quick_fix(quick_fix.with_id(id)),
        None => diagnostic.with_quick_fix(quick_fix),
    };

    for name in names {
        diagnostic = diagnostic.with_quick_fix(QuickFix::replacement(
            format!("add locale message stub `{name}`"),
            Replacement {
                span: Span::new(locale_span.end, locale_span.end),
                text: missing_message_stub_text(&[name]),
            },
        ));
    }

    diagnostic
}

fn unknown_messages_diagnostic(unknown: &[&ImplementedLocaleMessage]) -> Diagnostic {
    let names = unknown
        .iter()
        .map(|message| message.name.as_str())
        .collect::<Vec<_>>();
    let mut diagnostic = Diagnostic::error(
        format!(
            "locale implements {} unknown public {}: {}",
            names.len(),
            pluralize(names.len(), "message", "messages"),
            format_name_list(&names),
        ),
        unknown[0].span,
    )
    .with_note("remove these messages or add matching declarations to the schema");

    for message in unknown.iter().skip(1) {
        diagnostic = diagnostic.with_related(
            message.span,
            format!("unknown implementation `{}`", message.name),
        );
    }

    diagnostic
}

fn collect_schema_messages(
    declaration: &SchemaDeclaration,
    group: Option<&str>,
    messages: &mut Vec<RequiredLocaleMessage>,
) {
    match declaration {
        SchemaDeclaration::Message(message) => messages.push(
            RequiredLocaleMessage::new(
                qualified_name(group, &message.name.value),
                message.name.span,
            )
            .with_docs(&message.docs),
        ),
        SchemaDeclaration::Group(group_declaration) => {
            for message in &group_declaration.messages {
                messages.push(
                    RequiredLocaleMessage::new(
                        qualified_name(Some(&group_declaration.name.value), &message.name.value),
                        message.name.span,
                    )
                    .with_docs(&message.docs),
                );
            }
        }
        SchemaDeclaration::Enum(_) | SchemaDeclaration::TypeAlias(_) => {}
    }
}

fn collect_locale_messages(
    declaration: &LocaleDeclaration,
    group: Option<&str>,
    messages: &mut Vec<ImplementedLocaleMessage>,
) {
    match declaration {
        LocaleDeclaration::Message(message) => messages.push(
            ImplementedLocaleMessage::new(
                qualified_name(group, &message.name.value),
                message.name.span,
            )
            .with_docs(&message.docs),
        ),
        LocaleDeclaration::Group(group_declaration) => {
            for message in &group_declaration.messages {
                messages.push(
                    ImplementedLocaleMessage::new(
                        qualified_name(Some(&group_declaration.name.value), &message.name.value),
                        message.name.span,
                    )
                    .with_docs(&message.docs),
                );
            }
        }
        LocaleDeclaration::Override(inner) => collect_locale_messages(inner, group, messages),
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Function(_) => {}
    }
}

fn missing_doc_comment_diagnostics(
    schema: &BTreeMap<&str, &RequiredLocaleMessage>,
    locale: &BTreeMap<&str, &ImplementedLocaleMessage>,
) -> Vec<Diagnostic> {
    schema
        .iter()
        .filter_map(|(name, schema_message)| {
            if schema_message.docs.is_empty() {
                return None;
            }
            let locale_message = locale.get(name)?;
            if !locale_message.docs.is_empty() {
                return None;
            }
            Some(
                Diagnostic::warning(
                    format!("locale message `{name}` is missing schema doc comment"),
                    locale_message.span,
                )
                .with_note("copy or adapt the schema documentation for translator context"),
            )
        })
        .collect()
}

fn analyze_locale_branch_coverage(
    schema_enums: &BTreeMap<String, Vec<NamedSpan>>,
    locale: &LocaleFile,
) -> Vec<Diagnostic> {
    let mut enum_variants = schema_enums.clone();
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
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Message(_)
        | LocaleDeclaration::Group(_) => {}
    }
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
        .map(|branch| NamedSpan::new(&branch.key.value, branch.key.span))
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

fn qualified_name(group: Option<&str>, name: &str) -> String {
    match group {
        Some(group) => format!("{group}.{name}"),
        None => name.to_owned(),
    }
}

