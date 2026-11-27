use std::collections::{BTreeMap, BTreeSet};

use linguini_analyzer::{inherited_locale_globals, InheritedLocaleGlobal};
use linguini_core::{is_plural_intrinsic, TypeKind};
use linguini_syntax::{
    Expression, ExpressionKind, FormEntry, FunctionBranch, FunctionBranchValue, LocaleDeclaration,
    LocaleFile, LocaleValue, MapBranch, SchemaDeclaration, SchemaFile, Span, TextPart, TextPattern,
};

#[derive(Debug, Clone, Eq, PartialEq)]
/// Stable semantic identity shared by schema and locale tooling.
pub enum SemanticKey {
    /// Canonical message path.
    Message(String),
    /// Schema or locale type name.
    Type(String),
    /// Member owned by one enum.
    EnumVariant {
        /// Owning enum name.
        enumeration: String,
        /// Variant name.
        variant: String,
    },
    /// Nested attribute owned by one form.
    FormAttribute {
        /// Owning form name.
        form: String,
        /// Canonical attribute path.
        path: String,
    },
    /// Locale variable name.
    Variable(String),
    /// Locale function name.
    Function(String),
    /// Parameter or inline binding scoped to one owner.
    Parameter {
        /// Canonical message, function, or inline-expression owner.
        owner: String,
        /// Parameter or binding name.
        name: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// One declaration or reference discovered by the shared semantic walk.
pub struct SemanticOccurrence {
    /// Resolved semantic identity.
    pub key: SemanticKey,
    /// Exact source span.
    pub span: Span,
    /// Whether this occurrence declares the identity.
    pub declaration: bool,
}

/// Indexes declarations and references in one parsed schema file.
pub fn schema_occurrences(schema: &SchemaFile) -> Vec<SemanticOccurrence> {
    let mut output = Vec::new();
    for declaration in schema.declarations() {
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

/// Indexes declarations and references in one parsed locale file.
pub fn locale_occurrences(locale: &LocaleFile) -> Vec<SemanticOccurrence> {
    let mut names = LocaleNames::default();
    for declaration in locale.declarations() {
        collect_locale_names(declaration, &mut names);
    }
    collect_locale_occurrences(locale, &names)
}

/// One global declaration and the original source that owns it.
#[derive(Debug, Clone)]
pub struct LocaleGlobalOccurrence<'a> {
    /// Original parsed locale, retained independently of numeric source IDs.
    pub source: &'a LocaleFile,
    /// Declaration of a variable, function, locale enum/member, or form attribute.
    pub occurrence: SemanticOccurrence,
}

/// Current-source occurrences resolved against its effective global declarations.
#[derive(Debug, Clone)]
pub struct LocaleNavigation<'a> {
    /// Only occurrences physically present in the current locale.
    pub occurrences: Vec<SemanticOccurrence>,
    /// Local and inherited global declarations with their original source ownership.
    pub globals: Vec<LocaleGlobalOccurrence<'a>>,
    /// Selected whole fallback declarations, retaining their original body scopes.
    pub inherited: Vec<InheritedLocaleGlobal<'a>>,
}

/// Index one locale using the analyzer's whole-declaration fallback selection.
///
/// Fallbacks are ordered from highest to lowest priority. Inherited bodies are not added
/// to current-source occurrences; callers index those sources in their own context.
pub fn locale_navigation<'a>(
    locale: &'a LocaleFile,
    fallbacks: &[&'a LocaleFile],
) -> LocaleNavigation<'a> {
    let inherited = inherited_locale_globals(locale, fallbacks);
    let mut names = LocaleNames::default();
    for declaration in locale.declarations() {
        collect_locale_names(declaration, &mut names);
    }
    for global in &inherited {
        collect_locale_names(global.declaration, &mut names);
    }

    let output = collect_locale_occurrences(locale, &names);
    let mut globals = global_occurrences(locale, &output);
    for global in &inherited {
        let mut occurrences = Vec::new();
        collect_locale_declaration(global.declaration, &names, &mut occurrences);
        globals.extend(global_occurrences(global.source, &occurrences));
    }
    LocaleNavigation {
        occurrences: output,
        globals,
        inherited,
    }
}

fn collect_locale_occurrences(locale: &LocaleFile, names: &LocaleNames) -> Vec<SemanticOccurrence> {
    let mut output = Vec::new();
    for declaration in locale.declarations() {
        collect_locale_declaration(declaration, names, &mut output);
    }
    output
}

fn global_occurrences<'a>(
    source: &'a LocaleFile,
    occurrences: &[SemanticOccurrence],
) -> Vec<LocaleGlobalOccurrence<'a>> {
    occurrences
        .iter()
        .filter(|occurrence| {
            occurrence.declaration
                && matches!(
                    occurrence.key,
                    SemanticKey::Variable(_)
                        | SemanticKey::Function(_)
                        | SemanticKey::Type(_)
                        | SemanticKey::EnumVariant { .. }
                        | SemanticKey::FormAttribute { .. }
                )
        })
        .map(|occurrence| LocaleGlobalOccurrence {
            source,
            occurrence: occurrence.clone(),
        })
        .collect()
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
        // Calls resolve named functions and intrinsics before same-named local values.
        // References retain lexical binding precedence.
        let (key, span) = if expression.kind == ExpressionKind::Call
            && expression.path.len() == 1
            && names.functions.contains(&root.value)
        {
            (Some(SemanticKey::Function(root.value.clone())), root.span)
        } else if expression.kind == ExpressionKind::Call
            && expression.path.len() == 1
            && is_plural_intrinsic(&root.value)
        {
            (Some(SemanticKey::Type("Plural".to_owned())), root.span)
        } else if names.messages.contains(&full_path) {
            (Some(SemanticKey::Message(full_path)), last.span)
        } else if let Some(key) = bindings.get(&root.value) {
            (Some(key.clone()), root.span)
        } else if expression.kind == ExpressionKind::Reference
            && names.variables.contains(&root.value)
        {
            (Some(SemanticKey::Variable(root.value.clone())), root.span)
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
    use super::{is_builtin_type, locale_navigation, SemanticKey};
    use linguini_core::TypeKind;
    use linguini_syntax::{parse_locale_with_recovery_in, SourceId};

    #[test]
    fn builtin_type_filter_tracks_the_shared_primitive_registry() {
        for kind in TypeKind::all() {
            assert!(is_builtin_type(kind.as_str()));
        }
        assert!(!is_builtin_type("UserType"));
    }

    #[test]
    fn contextual_navigation_borrows_effective_globals_without_importing_bodies() {
        let parse = |id, text| {
            let parsed = parse_locale_with_recovery_in(text, SourceId(id));
            assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
            parsed.ast.unwrap()
        };
        let base = parse(1, "let suffix = BASE\nlet pair = VALUE\nfn pair(count: Number) { _ => {count} }\nfn hidden(count: Number) { _ => BASE }\nenum Tone { formal }\nmessage = BASE\n");
        let parent = parse(
            2,
            "fn render(count: Number) { _ => {count}|{suffix} }\nenum Tone { casual }\n",
        );
        let current = parse(
            3,
            "hidden = LOCAL\nsummary = {render(count)}|{suffix}|{pair}|{pair(count)}\n",
        );
        let original = (base.clone(), parent.clone(), current.clone());
        let navigation = locale_navigation(&current, &[&parent, &base]);
        assert!(navigation
            .occurrences
            .iter()
            .all(|item| item.span.source == SourceId(3)));
        for (key, owner) in [
            (SemanticKey::Variable("suffix".into()), &base),
            (SemanticKey::Variable("pair".into()), &base),
            (SemanticKey::Function("pair".into()), &base),
            (SemanticKey::Function("render".into()), &parent),
            (SemanticKey::Type("Tone".into()), &parent),
        ] {
            let global = navigation
                .globals
                .iter()
                .find(|global| global.occurrence.key == key)
                .unwrap();
            assert!(std::ptr::eq(global.source, owner));
            if !matches!(key, SemanticKey::Type(_)) {
                assert!(navigation
                    .occurrences
                    .iter()
                    .any(|item| !item.declaration && item.key == key));
            }
        }
        assert!(!navigation
            .globals
            .iter()
            .any(|global| matches!(&global.occurrence.key,
            SemanticKey::Function(name) if name == "hidden")));
        assert!(!navigation
            .globals
            .iter()
            .any(|global| matches!(&global.occurrence.key,
            SemanticKey::EnumVariant { variant, .. } if variant == "formal")));
        assert!(
            !navigation.occurrences.iter().any(|item| matches!(&item.key,
            SemanticKey::Message(name) if name == "message"))
        );
        assert_eq!((base, parent, current), original);
    }
}
