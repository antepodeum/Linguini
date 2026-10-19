use std::collections::BTreeMap;

use linguini_ir::{
    is_plural_intrinsic, IrBranch, IrExpression, IrExpressionKind, IrFormEntry, IrFormatter,
    IrFormatterKind, IrFunctionBranch, IrFunctionBranchValue, IrFunctionParameter, IrText,
    IrTextPart, IrValue,
};

use super::names::{
    form_binding_name, path_expression, property_access, property_key, safe_identifier,
    string_literal,
};
use super::{render_jsdoc_type, EcmaOptions, TypeModel};

pub(super) fn form_object(entries: &[IrFormEntry], options: &EcmaOptions) -> String {
    let fields = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute {
                name,
                parameters,
                value,
            } => Some(minijinja::context! { key => property_key(name), value => value_expression_with_parameters(value, parameters, options) }),
            IrFormEntry::Branch(_) => None,
        })
        .collect::<Vec<_>>();
    let branches = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute { .. } => None,
            IrFormEntry::Branch(branch) => Some(branch.clone()),
        })
        .collect::<Vec<_>>();

    let dispatcher = (!branches.is_empty()).then(|| {
        map_expression(
            &branches,
            &[IrFunctionParameter {
                name: None,
                ty: "Plural".to_owned(),
            }],
            options,
        )
    });
    crate::templates::render(
        "expression",
        minijinja::context! { kind => "object", fields, dispatcher },
    )
}

fn value_expression_with_parameters(
    value: &IrValue,
    parameters: &[IrFunctionParameter],
    options: &EcmaOptions,
) -> String {
    match value {
        IrValue::Text(text) => text_expression(text, options),
        IrValue::Map(branches) => map_expression(branches, parameters, options),
        IrValue::Object(entries) => form_object(entries, options),
    }
}

pub fn map_expression(
    branches: &[IrBranch],
    parameters: &[IrFunctionParameter],
    options: &EcmaOptions,
) -> String {
    let context = parameters
        .iter()
        .filter_map(|parameter| {
            parameter
                .name
                .as_ref()
                .map(|name| (name.clone(), parameter.ty.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let dispatch = parameters
        .first()
        .expect("validated form maps have one dispatch parameter");
    let parameter = dispatch
        .name
        .as_deref()
        .map(safe_identifier)
        .unwrap_or_else(|| "value".to_owned());
    let selector = selector_expression(&parameter, dispatch.ty == "Plural", true, options);
    let decision = linguini_codegen::Decision::branch(
        selector,
        branches
            .iter()
            .flat_map(|branch| {
                let value = linguini_codegen::Decision::Return(text_expression_with_context(
                    &branch.value,
                    &context,
                    &BTreeMap::new(),
                    options,
                ));
                let keys = if branch.keys.is_empty() {
                    vec!["_".to_owned()]
                } else {
                    branch.keys.clone()
                };
                keys.into_iter().map(move |key| (key, value.clone()))
            })
            .collect(),
    );
    let body = crate::templates::render(
        "dispatch",
        minijinja::context! { node => decision_context(&decision, 0) },
    );
    let parameter_type = if dispatch.ty == "Plural" {
        "number | bigint | string".to_owned()
    } else {
        render_jsdoc_type(&TypeModel::from_source_name(&dispatch.ty))
    };
    crate::templates::render(
        "dispatch_function",
        minijinja::context! {
            parameter, parameter_type, body,
        },
    )
    .trim_end()
    .to_owned()
}

pub(super) fn text_expression(text: &IrText, options: &EcmaOptions) -> String {
    text_expression_with_context(text, &BTreeMap::new(), &BTreeMap::new(), options)
}

pub(super) fn text_expression_with_context(
    text: &IrText,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &EcmaOptions,
) -> String {
    let parts = text
        .parts
        .iter()
        .map(|part| match part {
            IrTextPart::Text(raw) => string_literal(raw),
            IrTextPart::Placeholder(expression) => {
                expression_string(expression, context, default_formatters, options)
            }
        })
        .collect::<Vec<_>>();

    crate::templates::render(
        "expression",
        minijinja::context! { kind => "concat", values => parts },
    )
}

fn expression_string(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &EcmaOptions,
) -> String {
    let value = expression_value(expression, context, options);
    let formatters = if expression.formatters.is_empty() {
        expression
            .path
            .first()
            .and_then(|name| default_formatters.get(name))
            .map_or(expression.formatters.as_slice(), Vec::as_slice)
    } else {
        expression.formatters.as_slice()
    };
    let formatted = apply_formatters(value, formatters);
    crate::templates::render(
        "expression",
        minijinja::context! { kind => "string", value => formatted },
    )
}

pub(super) fn expression_value(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    options: &EcmaOptions,
) -> String {
    assert!(
        !matches!(expression.kind, IrExpressionKind::InlineFunction { .. }),
        "inline dispatch must be lowered before rendering"
    );
    let called = expression.kind == IrExpressionKind::Call;
    let root = expression.path.first().map(|root| safe_identifier(root));
    let form = expression
        .path
        .first()
        .filter(|_| called || expression.path.len() > 1)
        .and_then(|root| context.get(root))
        .map(|ty| form_binding_name(ty));
    let suffix = expression
        .path
        .iter()
        .skip(1)
        .map(|part| property_access(part))
        .collect::<String>();
    let path = if called && expression.path.len() == 1 && is_plural_intrinsic(&expression.path[0]) {
        options.plural_function.clone()
    } else {
        path_expression(&expression.path)
    };
    let arguments = expression
        .arguments
        .iter()
        .map(|argument| expression_value(argument, context, options))
        .collect::<Vec<_>>();
    crate::templates::render(
        "expression",
        minijinja::context! {
            kind => "value", empty => expression.path.is_empty(), root, form, suffix, path, called, arguments,
        },
    )
}

pub(super) fn function_dispatch_body(
    parameters: &[IrFunctionParameter],
    branches: &[IrFunctionBranch],
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &EcmaOptions,
) -> String {
    let parameter_names =
        super::names::function_parameter_names(parameters, branches, &options.plural_function);
    let selectors = parameters
        .iter()
        .enumerate()
        .filter(|(_, parameter)| parameter.name.is_none())
        .map(|(index, parameter)| DispatchSelector {
            value: parameter_names[index].clone(),
            normalize_plural: parameter.ty == "Plural",
            coerce: parameter.ty != "String",
        })
        .collect::<Vec<_>>();
    let decision = dispatch_decision(
        &selectors,
        branches,
        0,
        context,
        default_formatters,
        options,
    );
    crate::templates::render(
        "dispatch",
        minijinja::context! { node => decision_context(&decision, 0) },
    )
}

#[derive(Debug)]
struct DispatchSelector {
    value: String,
    normalize_plural: bool,
    coerce: bool,
}

fn dispatch_decision(
    selectors: &[DispatchSelector],
    branches: &[IrFunctionBranch],
    depth: usize,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &EcmaOptions,
) -> linguini_codegen::Decision<String, String> {
    use linguini_codegen::Decision;
    if selectors.get(depth).is_none() {
        if let Some(branch) = branches
            .iter()
            .find(|branch| branch.key == "undefined")
            .or_else(|| branches.iter().find(|branch| branch.key == "_"))
            .or_else(|| branches.iter().find(|branch| branch.key == "other"))
        {
            return match &branch.value {
                IrFunctionBranchValue::Text(text) => Decision::Return(
                    text_expression_with_context(text, context, default_formatters, options),
                ),
                IrFunctionBranchValue::Dispatch(children) => dispatch_decision(
                    selectors,
                    children,
                    depth + 1,
                    context,
                    default_formatters,
                    options,
                ),
            };
        }
    }
    let selector = selectors.get(depth).map_or_else(
        || "undefined".to_owned(),
        |selector| {
            selector_expression(
                &selector.value,
                selector.normalize_plural,
                selector.coerce,
                options,
            )
        },
    );
    Decision::branch(
        selector,
        branches
            .iter()
            .map(|branch| {
                let result = match &branch.value {
                    IrFunctionBranchValue::Text(text) => Decision::Return(
                        text_expression_with_context(text, context, default_formatters, options),
                    ),
                    IrFunctionBranchValue::Dispatch(children) => dispatch_decision(
                        selectors,
                        children,
                        depth + 1,
                        context,
                        default_formatters,
                        options,
                    ),
                };
                (branch.key.clone(), result)
            })
            .collect(),
    )
}

fn decision_context(
    decision: &linguini_codegen::Decision<String, String>,
    depth: usize,
) -> minijinja::Value {
    use linguini_codegen::Decision;
    use minijinja::context;
    match decision {
        Decision::Return(value) => context! { kind => "return", value },
        Decision::Branch {
            selector,
            arms,
            fallback,
        } => {
            let arms: Vec<_> = arms.iter().map(|arm| context! { keys => &arm.keys, result => decision_context(&arm.result, depth + 1) }).collect();
            context! {
                kind => "branch", selector, arms,
                fallback => fallback.as_ref().map(|node| decision_context(node, depth + 1)),
                key_binding => format!("__lgl_key_{depth}"),
            }
        }
    }
}

fn selector_expression(value: &str, plural: bool, coerce: bool, options: &EcmaOptions) -> String {
    crate::templates::render(
        "selector",
        minijinja::context! {
            value, plural, coerce, plural_function => &options.plural_function,
        },
    )
    .trim_end()
    .to_owned()
}

fn apply_formatters(value: String, formatters: &[IrFormatter]) -> String {
    formatters.iter().fold(value, |current, formatter| {
        if matches!(formatter.kind, IrFormatterKind::Unknown(_)) {
            return current;
        }
        let name = super::formatters::formatter_binding(&linguini_codegen::FormatterRequest::new(
            formatter,
        ));
        crate::templates::render(
            "formatter_call",
            minijinja::context! { name, value => current },
        )
        .trim_end()
        .to_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::{expression_value, form_object, EcmaOptions};
    use linguini_ir::{lower_locale, IrExpression, IrExpressionKind};
    use linguini_syntax::{parse_locale, Span};
    use std::collections::BTreeMap;

    fn expression(kind: IrExpressionKind, path: &[&str]) -> IrExpression {
        IrExpression {
            kind,
            path: path.iter().map(|part| (*part).to_owned()).collect(),
            arguments: Vec::new(),
            formatters: Vec::new(),
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn zero_argument_global_call_differs_from_reference() {
        let context = BTreeMap::new();
        let options = EcmaOptions::default();

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["ready"]),
                &context,
                &options
            ),
            "ready()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["ready"]),
                &context,
                &options
            ),
            "ready"
        );
    }

    #[test]
    fn canonical_plural_call_uses_configured_function() {
        let context = BTreeMap::new();
        let options = EcmaOptions {
            plural_function: "selectPlural".to_owned(),
            ..EcmaOptions::default()
        };

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["Plural"]),
                &context,
                &options
            ),
            "selectPlural()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["plural"]),
                &context,
                &options
            ),
            "plural()"
        );
    }

    #[test]
    fn zero_argument_form_calls_differ_from_form_references() {
        let context = BTreeMap::from([("item".to_owned(), "Item".to_owned())]);
        let options = EcmaOptions::default();

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["item"]),
                &context,
                &options
            ),
            "__lgl_form_4974656D[item]()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["item"]),
                &context,
                &options
            ),
            "item"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["item", "label"]),
                &context,
                &options
            ),
            "__lgl_form_4974656D[item].label()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["item", "label"]),
                &context,
                &options
            ),
            "__lgl_form_4974656D[item].label"
        );
    }

    #[test]
    fn explicit_non_plural_form_selector_uses_string_dispatch() {
        let locale = lower_locale(
            &parse_locale(
                "impl Fruit {\n  apple {\n    form label(gender: Gender) {\n      male => He\n      _ => They\n    }\n  }\n}\n",
            )
            .expect("locale"),
        );
        let entries = &locale.forms()[0].variants[0].entries;

        let emitted = form_object(entries, &EcmaOptions::default());

        assert!(emitted.contains("label: /** @type {(gender: Gender) => string} */ ((gender) => {"));
        assert!(emitted.contains("if (String(gender) === \"male\") return \"He\";"));
        assert!(!emitted.contains("selectBranch("));
        assert!(!emitted.contains("pluralEn(gender)"));
    }
}
