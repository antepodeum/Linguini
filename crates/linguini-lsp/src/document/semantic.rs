use std::collections::{BTreeMap, BTreeSet};

use linguini_core::TypeKind;
use linguini_format::SourceKind;
use linguini_syntax::{
    lex_schema_with_recovery, Expression, ExpressionKind, FormEntry, FunctionBranch,
    FunctionBranchValue, LocaleDeclaration, LocaleValue, MapBranch, SchemaDeclaration, Span,
    TextPart, TextPattern, TokenKind,
};

use super::{contains, parsed_locale, parsed_schema, LinguiniDocument};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum SemanticKey {
    Message(String),
    Type(String),
    EnumVariant {
        enumeration: String,
        variant: String,
    },
    FormAttribute {
        form: String,
        path: String,
    },
    Variable(String),
    Function(String),
    Parameter {
        owner: String,
        name: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct SemanticOccurrence {
    pub key: SemanticKey,
    pub span: Span,
    pub declaration: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedOccurrence {
    pub document: LinguiniDocument,
    pub occurrence: SemanticOccurrence,
}

pub(super) fn occurrence_at(
    document: &LinguiniDocument,
    offset: usize,
) -> Option<SemanticOccurrence> {
    occurrences(document)
        .into_iter()
        .find(|occurrence| contains(occurrence.span, offset))
}

pub(super) fn resolved_occurrences(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
) -> Option<Vec<ResolvedOccurrence>> {
    let source_occurrence = occurrence_at(source, offset)?;
    let mut documents = documents.into_iter().collect::<Vec<_>>();
    if !documents.iter().any(|document| document.uri == source.uri) {
        documents.push(source.clone());
    }

    let document_local = match &source_occurrence.key {
        SemanticKey::FormAttribute { .. } | SemanticKey::Variable(_) | SemanticKey::Function(_) => {
            true
        }
        SemanticKey::Parameter { owner, .. } => {
            owner.starts_with("fn:") || owner.starts_with("inline:")
        }
        SemanticKey::Message(_) | SemanticKey::Type(_) | SemanticKey::EnumVariant { .. } => false,
    };
    let schema_anchor = if document_local || source.namespace.is_some() {
        None
    } else {
        unique_schema_anchor(&documents, source, &source_occurrence)?
    };

    let mut resolved = Vec::new();
    for document in documents {
        if document_local && document.uri != source.uri {
            continue;
        }
        if !document_local && !same_namespace(source, &document, schema_anchor.as_deref()) {
            continue;
        }
        for occurrence in occurrences(&document) {
            if occurrence.key == source_occurrence.key {
                resolved.push(ResolvedOccurrence {
                    document: document.clone(),
                    occurrence,
                });
            }
        }
    }

    (!resolved.is_empty()).then_some(resolved)
}

pub(super) fn definition_occurrence(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
) -> Option<ResolvedOccurrence> {
    let source_occurrence = occurrence_at(source, offset)?;
    let resolved = resolved_occurrences(documents, source, offset)?;
    let declarations = resolved
        .into_iter()
        .filter(|candidate| candidate.occurrence.declaration)
        .collect::<Vec<_>>();

    if source.kind == SourceKind::Schema && source_occurrence.declaration {
        return declarations.into_iter().find(|candidate| {
            candidate.document.uri == source.uri
                && candidate.occurrence.span == source_occurrence.span
        });
    }

    let schema_declarations = declarations
        .iter()
        .filter(|candidate| candidate.document.kind == SourceKind::Schema)
        .collect::<Vec<_>>();
    if schema_declarations.len() == 1 {
        return schema_declarations.into_iter().next().cloned();
    }
    if schema_declarations.len() > 1 {
        return None;
    }

    let local = declarations
        .into_iter()
        .filter(|candidate| candidate.document.uri == source.uri)
        .collect::<Vec<_>>();
    (local.len() == 1).then(|| local[0].clone())
}

pub(super) fn rename_occurrences(
    documents: impl IntoIterator<Item = LinguiniDocument>,
    source: &LinguiniDocument,
    offset: usize,
    new_name: &str,
) -> Option<Vec<ResolvedOccurrence>> {
    if !valid_identifier(new_name) {
        return None;
    }
    let resolved = resolved_occurrences(documents, source, offset)?;
    let source_occurrence = occurrence_at(source, offset)?;
    let renamed = renamed_key(&source_occurrence.key, new_name);
    if renamed == source_occurrence.key {
        return Some(Vec::new());
    }

    let affected_documents = resolved
        .iter()
        .map(|candidate| (candidate.document.uri.clone(), candidate.document.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    for candidate in affected_documents.into_values() {
        if occurrences(&candidate).into_iter().any(|occurrence| {
            occurrence.declaration
                && occurrence.key != source_occurrence.key
                && declaration_keys_conflict(&renamed, &occurrence.key)
        }) {
            return None;
        }
    }

    Some(resolved)
}

pub(super) fn valid_identifier(value: &str) -> bool {
    if value == "_"
        || matches!(
            value,
            "enum" | "type" | "impl" | "form" | "fn" | "let" | "override"
        )
    {
        return false;
    }
    let lexed = lex_schema_with_recovery(value);
    lexed.errors.is_empty()
        && matches!(
            lexed.tokens.as_slice(),
            [token]
                if token.span == Span::new(0, value.len())
                    && matches!(&token.kind, TokenKind::Ident(name) if name == value)
        )
}

pub(super) fn occurrences(document: &LinguiniDocument) -> Vec<SemanticOccurrence> {
    document
        .semantic_occurrences
        .get_or_init(|| match document.kind {
            SourceKind::Schema => schema_occurrences(document),
            SourceKind::Locale => locale_occurrences(document),
        })
        .clone()
}

fn schema_occurrences(document: &LinguiniDocument) -> Vec<SemanticOccurrence> {
    let Some(schema) = parsed_schema(document).and_then(|parsed| parsed.ast.as_ref()) else {
        return Vec::new();
    };
    let mut output = Vec::new();
    for declaration in &schema.declarations {
        match declaration {
            SchemaDeclaration::Enum(item) => {
                push(
                    &mut output,
                    SemanticKey::Type(item.name.value.clone()),
                    item.name.span,
                    true,
                );
                for variant in &item.variants {
                    push(
                        &mut output,
                        SemanticKey::EnumVariant {
                            enumeration: item.name.value.clone(),
                            variant: variant.value.clone(),
                        },
                        variant.span,
                        true,
                    );
                }
            }
            SchemaDeclaration::TypeAlias(item) => {
                push(
                    &mut output,
                    SemanticKey::Type(item.name.value.clone()),
                    item.name.span,
                    true,
                );
                if !is_builtin_type(&item.target.value) {
                    push(
                        &mut output,
                        SemanticKey::Type(item.target.value.clone()),
                        item.target.span,
                        false,
                    );
                }
            }
            SchemaDeclaration::Message(message) => {
                collect_schema_message(message, &message.name.value, &mut output);
            }
            SchemaDeclaration::Group(group) => {
                collect_schema_group(group, None, &mut output);
            }
        }
    }
    output
}

fn collect_schema_group(
    group: &linguini_syntax::MessageGroup,
    parent: Option<&str>,
    output: &mut Vec<SemanticOccurrence>,
) {
    let path = parent
        .map(|parent| format!("{parent}.{}", group.name.value))
        .unwrap_or_else(|| group.name.value.clone());
    for message in &group.messages {
        collect_schema_message(message, &format!("{path}.{}", message.name.value), output);
    }
    for child in &group.groups {
        collect_schema_group(child, Some(&path), output);
    }
}

fn collect_schema_message(
    message: &linguini_syntax::MessageSignature,
    path: &str,
    output: &mut Vec<SemanticOccurrence>,
) {
    push(
        output,
        SemanticKey::Message(path.to_owned()),
        message.name.span,
        true,
    );
    for parameter in &message.parameters {
        push(
            output,
            SemanticKey::Parameter {
                owner: path.to_owned(),
                name: parameter.name.value.clone(),
            },
            parameter.name.span,
            true,
        );
        if !is_builtin_type(&parameter.ty.value) {
            push(
                output,
                SemanticKey::Type(parameter.ty.value.clone()),
                parameter.ty.span,
                false,
            );
        }
    }
}

#[derive(Default)]
struct LocaleNames {
    messages: BTreeSet<String>,
    variables: BTreeSet<String>,
    functions: BTreeSet<String>,
}

fn locale_occurrences(document: &LinguiniDocument) -> Vec<SemanticOccurrence> {
    let Some(locale) = parsed_locale(document).and_then(|parsed| parsed.ast.as_ref()) else {
        return Vec::new();
    };
    let mut names = LocaleNames::default();
    for declaration in &locale.declarations {
        collect_locale_names(declaration, &mut names);
    }

    let mut output = Vec::new();
    for declaration in &locale.declarations {
        collect_locale_declaration(declaration, &names, &mut output);
    }
    output
}

fn collect_locale_names(declaration: &LocaleDeclaration, names: &mut LocaleNames) {
    match declaration {
        LocaleDeclaration::Variable(item) => {
            names.variables.insert(item.name.value.clone());
        }
        LocaleDeclaration::Function(item) => {
            names.functions.insert(item.name.value.clone());
        }
        LocaleDeclaration::Message(item) => {
            names.messages.insert(item.name.value.clone());
        }
        LocaleDeclaration::Group(group) => {
            collect_locale_group_names(group, None, names);
        }
        LocaleDeclaration::Override(inner) => collect_locale_names(inner, names),
        LocaleDeclaration::Enum(_) | LocaleDeclaration::Form(_) => {}
    }
}

fn collect_locale_group_names(
    group: &linguini_syntax::MessageImplementationGroup,
    parent: Option<&str>,
    names: &mut LocaleNames,
) {
    let path = parent
        .map(|parent| format!("{parent}.{}", group.name.value))
        .unwrap_or_else(|| group.name.value.clone());
    names.messages.extend(
        group
            .messages
            .iter()
            .map(|message| format!("{path}.{}", message.name.value)),
    );
    for child in &group.groups {
        collect_locale_group_names(child, Some(&path), names);
    }
}

fn collect_locale_declaration(
    declaration: &LocaleDeclaration,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    match declaration {
        LocaleDeclaration::Enum(item) => {
            push(
                output,
                SemanticKey::Type(item.name.value.clone()),
                item.name.span,
                true,
            );
            for variant in &item.variants {
                push(
                    output,
                    SemanticKey::EnumVariant {
                        enumeration: item.name.value.clone(),
                        variant: variant.value.clone(),
                    },
                    variant.span,
                    true,
                );
            }
        }
        LocaleDeclaration::Variable(item) => {
            push(
                output,
                SemanticKey::Variable(item.name.value.clone()),
                item.name.span,
                true,
            );
            collect_text(&item.value, None, names, output);
        }
        LocaleDeclaration::Form(item) => {
            push(
                output,
                SemanticKey::Type(item.name.value.clone()),
                item.name.span,
                false,
            );
            for variant in &item.variants {
                push(
                    output,
                    SemanticKey::EnumVariant {
                        enumeration: item.name.value.clone(),
                        variant: variant.name.value.clone(),
                    },
                    variant.name.span,
                    false,
                );
                for entry in &variant.entries {
                    collect_form_entry(entry, &item.name.value, None, names, output);
                }
            }
        }
        LocaleDeclaration::Function(item) => {
            let owner = format!("fn:{}", item.name.value);
            push(
                output,
                SemanticKey::Function(item.name.value.clone()),
                item.name.span,
                true,
            );
            let parameters = item
                .parameters
                .iter()
                .filter_map(|parameter| {
                    parameter.name.as_ref().map(|name| {
                        let key = SemanticKey::Parameter {
                            owner: owner.clone(),
                            name: name.value.clone(),
                        };
                        push(output, key.clone(), name.span, true);
                        (name.value.clone(), key)
                    })
                })
                .collect::<BTreeMap<_, _>>();
            for parameter in &item.parameters {
                if !is_builtin_type(&parameter.ty.value) && parameter.ty.value != "Plural" {
                    push(
                        output,
                        SemanticKey::Type(parameter.ty.value.clone()),
                        parameter.ty.span,
                        false,
                    );
                }
            }
            collect_function_branches(&item.branches, &owner, &parameters, names, output);
        }
        LocaleDeclaration::Message(item) => {
            push(
                output,
                SemanticKey::Message(item.name.value.clone()),
                item.name.span,
                true,
            );
            collect_text(&item.value, Some(&item.name.value), names, output);
        }
        LocaleDeclaration::Group(group) => {
            collect_locale_group(group, None, names, output);
        }
        LocaleDeclaration::Override(inner) => collect_locale_declaration(inner, names, output),
    }
}

fn collect_locale_group(
    group: &linguini_syntax::MessageImplementationGroup,
    parent: Option<&str>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    let path = parent
        .map(|parent| format!("{parent}.{}", group.name.value))
        .unwrap_or_else(|| group.name.value.clone());
    for message in &group.messages {
        let message_path = format!("{path}.{}", message.name.value);
        push(
            output,
            SemanticKey::Message(message_path.clone()),
            message.name.span,
            true,
        );
        collect_text(&message.value, Some(&message_path), names, output);
    }
    for child in &group.groups {
        collect_locale_group(child, Some(&path), names, output);
    }
}

fn collect_form_entry(
    entry: &FormEntry,
    form: &str,
    parent: Option<&str>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    match entry {
        FormEntry::Attribute(attribute) => {
            let path = parent
                .map(|parent| format!("{parent}.{}", attribute.name.value))
                .unwrap_or_else(|| attribute.name.value.clone());
            push(
                output,
                SemanticKey::FormAttribute {
                    form: form.to_owned(),
                    path: path.clone(),
                },
                attribute.name.span,
                true,
            );
            collect_locale_value(&attribute.value, form, Some(&path), names, output);
        }
        FormEntry::Branch(branch) => collect_map_branch(branch, names, output),
    }
}

fn collect_locale_value(
    value: &LocaleValue,
    form: &str,
    parent: Option<&str>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    match value {
        LocaleValue::Text(text) => collect_text(text, None, names, output),
        LocaleValue::Map(branches) => {
            for branch in branches {
                collect_map_branch(branch, names, output);
            }
        }
        LocaleValue::Object(entries) => {
            for entry in entries {
                collect_form_entry(entry, form, parent, names, output);
            }
        }
    }
}

fn collect_map_branch(
    branch: &MapBranch,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    collect_text(&branch.value, None, names, output);
}

fn collect_function_branches(
    branches: &[FunctionBranch],
    owner: &str,
    bindings: &BTreeMap<String, SemanticKey>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    for branch in branches {
        match &branch.value {
            FunctionBranchValue::Text(text) => {
                collect_text_with_bindings(text, owner, bindings, names, output);
            }
            FunctionBranchValue::Dispatch(children) => {
                collect_function_branches(children, owner, bindings, names, output);
            }
        }
    }
}

fn collect_text(
    text: &TextPattern,
    message_owner: Option<&str>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    for part in &text.parts {
        if let TextPart::Placeholder(placeholder) = part {
            collect_expression(
                &placeholder.expression,
                message_owner,
                &BTreeMap::new(),
                names,
                output,
            );
        }
    }
}

fn collect_text_with_bindings(
    text: &TextPattern,
    owner: &str,
    bindings: &BTreeMap<String, SemanticKey>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    for part in &text.parts {
        if let TextPart::Placeholder(placeholder) = part {
            collect_expression(
                &placeholder.expression,
                Some(owner),
                bindings,
                names,
                output,
            );
        }
    }
}

fn collect_expression(
    expression: &Expression,
    owner: Option<&str>,
    bindings: &BTreeMap<String, SemanticKey>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    if let (Some(root), Some(last)) = (expression.path.first(), expression.path.last()) {
        let full_path = expression
            .path
            .iter()
            .map(|part| part.value.as_str())
            .collect::<Vec<_>>()
            .join(".");
        let (key, span) = if names.messages.contains(&full_path) {
            (Some(SemanticKey::Message(full_path)), last.span)
        } else if let Some(key) = bindings.get(&root.value) {
            (Some(key.clone()), root.span)
        } else if expression.kind == ExpressionKind::Reference
            && names.variables.contains(&root.value)
        {
            (Some(SemanticKey::Variable(root.value.clone())), root.span)
        } else if expression.kind == ExpressionKind::Call
            && expression.path.len() == 1
            && names.functions.contains(&root.value)
        {
            (Some(SemanticKey::Function(root.value.clone())), root.span)
        } else if expression.kind == ExpressionKind::Call
            && expression.path.len() == 1
            && linguini_ir::is_plural_intrinsic(&root.value)
        {
            (Some(SemanticKey::Type("Plural".to_owned())), root.span)
        } else if let Some(owner) = owner {
            (
                Some(SemanticKey::Parameter {
                    owner: owner.to_owned(),
                    name: root.value.clone(),
                }),
                root.span,
            )
        } else if names.variables.contains(&root.value) {
            (Some(SemanticKey::Variable(root.value.clone())), root.span)
        } else {
            (None, root.span)
        };
        if let Some(key) = key {
            push(output, key, span, false);
        }
    }
    for argument in &expression.arguments {
        collect_expression(argument, owner, bindings, names, output);
    }
    if let ExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        // Inline inputs have ordinary call semantics: every RHS resolves in
        // the enclosing lexical scope before any new binding is introduced.
        for input in inputs {
            let value = match input {
                linguini_syntax::InlineFunctionInput::Binding { value, .. }
                | linguini_syntax::InlineFunctionInput::Selector { value, .. } => value,
            };
            collect_expression(value, owner, bindings, names, output);
        }

        let inline_owner = format!("inline:{}", expression.span.start);
        let mut branch_bindings = bindings.clone();
        for input in inputs {
            if let linguini_syntax::InlineFunctionInput::Binding { name, .. } = input {
                let key = SemanticKey::Parameter {
                    owner: inline_owner.clone(),
                    name: name.value.clone(),
                };
                push(output, key.clone(), name.span, true);
                branch_bindings.insert(name.value.clone(), key);
            }
        }
        collect_inline_function_branches(branches, owner, &branch_bindings, names, output);
    }
}

fn collect_inline_function_branches(
    branches: &[FunctionBranch],
    owner: Option<&str>,
    bindings: &BTreeMap<String, SemanticKey>,
    names: &LocaleNames,
    output: &mut Vec<SemanticOccurrence>,
) {
    for branch in branches {
        match &branch.value {
            FunctionBranchValue::Text(text) => {
                for part in &text.parts {
                    if let TextPart::Placeholder(placeholder) = part {
                        collect_expression(&placeholder.expression, owner, bindings, names, output);
                    }
                }
            }
            FunctionBranchValue::Dispatch(children) => {
                collect_inline_function_branches(children, owner, bindings, names, output)
            }
        }
    }
}

fn declaration_keys_conflict(left: &SemanticKey, right: &SemanticKey) -> bool {
    match (declaration_slot(left), declaration_slot(right)) {
        (Some(left), Some(right)) => left == right,
        _ => left == right,
    }
}

fn declaration_slot(key: &SemanticKey) -> Option<(String, String)> {
    match key {
        SemanticKey::Message(path) => {
            let (scope, name) = path.rsplit_once('.').unwrap_or(("", path.as_str()));
            Some((format!("symbol:{scope}"), name.to_owned()))
        }
        SemanticKey::Type(name) | SemanticKey::Variable(name) | SemanticKey::Function(name) => {
            Some(("symbol:".to_owned(), name.clone()))
        }
        SemanticKey::EnumVariant {
            enumeration,
            variant,
        } => Some((format!("enum:{enumeration}"), variant.clone())),
        SemanticKey::FormAttribute { form, path } => {
            let (scope, name) = path.rsplit_once('.').unwrap_or(("", path.as_str()));
            Some((format!("form:{form}:{scope}"), name.to_owned()))
        }
        SemanticKey::Parameter { owner, name } => {
            Some((format!("parameter:{owner}"), name.clone()))
        }
    }
}

fn unique_schema_anchor(
    documents: &[LinguiniDocument],
    source: &LinguiniDocument,
    source_occurrence: &SemanticOccurrence,
) -> Option<Option<String>> {
    let schema_uris = documents
        .iter()
        .filter(|document| document.kind == SourceKind::Schema)
        .filter(|document| {
            occurrences(document)
                .into_iter()
                .any(|occurrence| occurrence.declaration && occurrence.key == source_occurrence.key)
        })
        .map(|document| document.uri.clone())
        .collect::<BTreeSet<_>>();

    if source.kind == SourceKind::Schema && source_occurrence.declaration {
        return Some(Some(source.uri.clone()));
    }
    match schema_uris.len() {
        0 => Some(None),
        1 => schema_uris.into_iter().next().map(Some),
        _ => None,
    }
}

fn same_namespace(
    source: &LinguiniDocument,
    candidate: &LinguiniDocument,
    schema_anchor: Option<&str>,
) -> bool {
    match (&source.namespace, &candidate.namespace) {
        (Some(left), Some(right)) => left == right,
        (Some(_), None) | (None, Some(_)) => false,
        (None, None) => {
            candidate.kind != SourceKind::Schema
                || schema_anchor
                    .map(|anchor| candidate.uri == anchor)
                    .unwrap_or(true)
        }
    }
}

fn renamed_key(key: &SemanticKey, new_name: &str) -> SemanticKey {
    match key {
        SemanticKey::Message(path) => {
            let prefix = path.rsplit_once('.').map(|(prefix, _)| prefix);
            SemanticKey::Message(
                prefix
                    .map(|prefix| format!("{prefix}.{new_name}"))
                    .unwrap_or_else(|| new_name.to_owned()),
            )
        }
        SemanticKey::Type(_) => SemanticKey::Type(new_name.to_owned()),
        SemanticKey::EnumVariant { enumeration, .. } => SemanticKey::EnumVariant {
            enumeration: enumeration.clone(),
            variant: new_name.to_owned(),
        },
        SemanticKey::FormAttribute { form, path } => {
            let prefix = path.rsplit_once('.').map(|(prefix, _)| prefix);
            SemanticKey::FormAttribute {
                form: form.clone(),
                path: prefix
                    .map(|prefix| format!("{prefix}.{new_name}"))
                    .unwrap_or_else(|| new_name.to_owned()),
            }
        }
        SemanticKey::Variable(_) => SemanticKey::Variable(new_name.to_owned()),
        SemanticKey::Function(_) => SemanticKey::Function(new_name.to_owned()),
        SemanticKey::Parameter { owner, .. } => SemanticKey::Parameter {
            owner: owner.clone(),
            name: new_name.to_owned(),
        },
    }
}

fn is_builtin_type(value: &str) -> bool {
    TypeKind::from_name(value).is_some()
}

fn push(output: &mut Vec<SemanticOccurrence>, key: SemanticKey, span: Span, declaration: bool) {
    output.push(SemanticOccurrence {
        key,
        span,
        declaration,
    });
}

#[cfg(test)]
mod tests {
    use super::is_builtin_type;
    use linguini_core::TypeKind;

    #[test]
    fn builtin_type_filter_tracks_the_shared_primitive_registry() {
        for kind in TypeKind::all() {
            assert!(is_builtin_type(kind.as_str()));
        }
        assert!(!is_builtin_type("UserType"));
    }
}
