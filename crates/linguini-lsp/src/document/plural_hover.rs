use super::{contains, LinguiniDocument};
use linguini_format::SourceKind;
use linguini_syntax::{
    parse_locale_with_recovery, FunctionBranch, FunctionBranchValue, FunctionDeclaration,
    LocaleDeclaration,
};

pub(super) fn plural_branch_hover(document: &LinguiniDocument, offset: usize) -> Option<String> {
    let SourceKind::Locale = document.kind else {
        return None;
    };
    let locale = locale_from_uri(&document.uri)?;
    let rules = linguini_cldr::compiled_plural_rules(&locale)?;
    let file = parse_locale_with_recovery(&document.text).ast?;

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
        LocaleDeclaration::Function(function) => function_plural_branch_hover(
            function,
            &dispatch_types(function),
            &function.branches,
            0,
            offset,
            locale,
            rules,
        ),
        LocaleDeclaration::Override(inner) => {
            declaration_plural_branch_hover(inner, offset, locale, rules)
        }
        LocaleDeclaration::Enum(_)
        | LocaleDeclaration::Variable(_)
        | LocaleDeclaration::Form(_)
        | LocaleDeclaration::Message(_)
        | LocaleDeclaration::Group(_) => None,
    }
}

fn function_plural_branch_hover(
    function: &FunctionDeclaration,
    dispatch_types: &[&str],
    branches: &[FunctionBranch],
    depth: usize,
    offset: usize,
    locale: &str,
    rules: &linguini_cldr::CompiledPluralRules,
) -> Option<String> {
    let dispatch_type = dispatch_types.get(depth).copied();
    for branch in branches {
        if dispatch_type == Some("Plural") && contains(branch.key.span, offset) {
            return Some(plural_samples_hover(
                &function.name.value,
                &branch.key.value,
                locale,
                rules,
            ));
        }
        if let FunctionBranchValue::Dispatch(children) = &branch.value {
            if let Some(hover) = function_plural_branch_hover(
                function,
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

fn dispatch_types(function: &FunctionDeclaration) -> Vec<&str> {
    function
        .parameters
        .iter()
        .filter_map(|parameter| {
            (parameter.ty.value != "String").then_some(parameter.ty.value.as_str())
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
