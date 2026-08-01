use super::{contains, parsed_locale, LinguiniDocument};
use linguini_format::SourceKind;
use linguini_syntax::{
    Expression, ExpressionKind, FormEntry, FunctionBranch, FunctionBranchValue,
    FunctionDeclaration, FunctionParameter, InlineFunctionInput, LocaleDeclaration, LocaleValue,
    MessageImplementationGroup, TextPart, TextPattern,
};

pub(super) fn plural_branch_hover(document: &LinguiniDocument, offset: usize) -> Option<String> {
    let SourceKind::Locale = document.kind else {
        return None;
    };
    let locale = document
        .locale
        .clone()
        .or_else(|| locale_from_uri(&document.uri))?;
    let rules = linguini_cldr::compiled_plural_rules(&locale)?;
    let file = parsed_locale(document)?.ast.as_ref()?;

    for declaration in &file.declarations {
        if let Some(hover) = declaration_plural_branch_hover(declaration, offset, &locale, &rules) {
            return Some(hover);
        }
    }
    None
}

fn declaration_plural_branch_hover(
    declaration: &LocaleDeclaration,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    match declaration {
        LocaleDeclaration::Function(function) => dispatch_plural_branch_hover(
            &function.name.value,
            &dispatch_types(function),
            &function.branches,
            0,
            offset,
            locale,
            rules,
        )
        .or_else(|| {
            function
                .branches
                .iter()
                .find_map(|branch| function_branch_plural_hover(branch, offset, locale, rules))
        }),
        LocaleDeclaration::Variable(variable) => {
            text_plural_branch_hover(&variable.value, offset, locale, rules)
        }
        LocaleDeclaration::Message(message) => {
            text_plural_branch_hover(&message.value, offset, locale, rules)
        }
        LocaleDeclaration::Group(group) => group_plural_branch_hover(group, offset, locale, rules),
        LocaleDeclaration::Form(form) => form.variants.iter().find_map(|variant| {
            form_entries_plural_branch_hover(&variant.entries, offset, locale, rules)
        }),
        LocaleDeclaration::Override(inner) => {
            declaration_plural_branch_hover(inner, offset, locale, rules)
        }
        LocaleDeclaration::Enum(_) => None,
    }
}

fn group_plural_branch_hover(
    group: &MessageImplementationGroup,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    group
        .messages
        .iter()
        .find_map(|message| text_plural_branch_hover(&message.value, offset, locale, rules))
        .or_else(|| {
            group
                .groups
                .iter()
                .find_map(|child| group_plural_branch_hover(child, offset, locale, rules))
        })
}

fn form_entries_plural_branch_hover(
    entries: &[FormEntry],
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    entries.iter().find_map(|entry| match entry {
        FormEntry::Branch(branch) => text_plural_branch_hover(&branch.value, offset, locale, rules),
        FormEntry::Attribute(attribute) => {
            locale_value_plural_branch_hover(&attribute.value, offset, locale, rules)
        }
    })
}

fn locale_value_plural_branch_hover(
    value: &LocaleValue,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    match value {
        LocaleValue::Text(text) => text_plural_branch_hover(text, offset, locale, rules),
        LocaleValue::Map(branches) => branches
            .iter()
            .find_map(|branch| text_plural_branch_hover(&branch.value, offset, locale, rules)),
        LocaleValue::Object(entries) => {
            form_entries_plural_branch_hover(entries, offset, locale, rules)
        }
    }
}

fn function_branch_plural_hover(
    branch: &FunctionBranch,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    match &branch.value {
        FunctionBranchValue::Text(text) => text_plural_branch_hover(text, offset, locale, rules),
        FunctionBranchValue::Dispatch(children) => children
            .iter()
            .find_map(|child| function_branch_plural_hover(child, offset, locale, rules)),
    }
}

fn text_plural_branch_hover(
    text: &TextPattern,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    text.parts.iter().find_map(|part| match part {
        TextPart::Text(_) => None,
        TextPart::Placeholder(placeholder) => {
            expression_plural_branch_hover(&placeholder.expression, offset, locale, rules)
        }
    })
}

fn expression_plural_branch_hover(
    expression: &Expression,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    if let ExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        let dispatch_types = inline_dispatch_types(inputs);
        if let Some(hover) = dispatch_plural_branch_hover(
            "inline fn",
            &dispatch_types,
            branches,
            0,
            offset,
            locale,
            rules,
        ) {
            return Some(hover);
        }
        for input in inputs {
            let value = match input {
                InlineFunctionInput::Binding { value, .. }
                | InlineFunctionInput::Selector { value, .. } => value,
            };
            if let Some(hover) = expression_plural_branch_hover(value, offset, locale, rules) {
                return Some(hover);
            }
        }
    }
    expression
        .arguments
        .iter()
        .find_map(|argument| expression_plural_branch_hover(argument, offset, locale, rules))
}

fn dispatch_plural_branch_hover(
    function_name: &str,
    dispatch_types: &[Option<&str>],
    branches: &[FunctionBranch],
    depth: usize,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    let dispatch_type = dispatch_types.get(depth).copied().flatten();
    for branch in branches {
        if dispatch_type == Some("Plural") && contains(branch.key.span, offset) {
            return Some(plural_samples_hover(
                function_name,
                &branch.key.value,
                locale,
                rules,
            ));
        }
        if let FunctionBranchValue::Dispatch(children) = &branch.value {
            if let Some(hover) = dispatch_plural_branch_hover(
                function_name,
                dispatch_types,
                children,
                depth + 1,
                offset,
                locale,
                rules,
            ) {
                return Some(hover);
            }
        }
    }
    None
}

fn dispatch_types(function: &FunctionDeclaration) -> Vec<Option<&str>> {
    parameter_dispatch_types(&function.parameters)
}

fn parameter_dispatch_types(parameters: &[FunctionParameter]) -> Vec<Option<&str>> {
    parameters
        .iter()
        .filter(|parameter| parameter.name.is_none())
        .map(|parameter| Some(parameter.ty.value.as_str()))
        .collect()
}

fn inline_dispatch_types(inputs: &[InlineFunctionInput]) -> Vec<Option<&str>> {
    inputs
        .iter()
        .filter_map(|input| match input {
            InlineFunctionInput::Binding { .. } => None,
            InlineFunctionInput::Selector { value, .. } => Some(
                (value.kind == ExpressionKind::Call
                    && value.path.len() == 1
                    && linguini_ir::is_plural_intrinsic(&value.path[0].value))
                .then_some("Plural"),
            ),
        })
        .collect()
}

fn plural_samples_hover(
    function_name: &str,
    branch: &str,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> String {
    let category = if branch == "_" { "other" } else { branch };
    let samples = plural_samples_for_category(rules, category);
    let sample_text = if samples.is_empty() {
        "no integer samples in 0..200".to_owned()
    } else {
        samples.join(", ")
    };
    format!(
        "plural branch `{branch}` in `{function_name}`\n\nLocale `{locale}` category `{category}`\n\nSample numbers: {sample_text}"
    )
}

fn plural_samples_for_category(
    rules: &linguini_cldr::CompiledPluralRules,
    category: &str,
) -> Vec<String> {
    (0..=200)
        .map(|number| number.to_string())
        .filter(|sample| {
            rules
                .category_for(sample)
                .is_ok_and(|candidate| candidate == category)
        })
        .take(12)
        .collect()
}

fn locale_from_uri(uri: &str) -> Option<String> {
    let file_name = uri
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())?;
    file_name
        .strip_suffix(".lgl")
        .or_else(|| file_name.strip_suffix(".linguini"))
        .map(|locale| locale.to_owned())
}
