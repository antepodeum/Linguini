use std::collections::BTreeMap;

use linguini_cldr::compiled_currency_fraction;
use linguini_ir::{
    is_plural_intrinsic, IrBranch, IrExpression, IrExpressionKind, IrFormEntry, IrFormatter,
    IrFormatterKind, IrFunctionBranch, IrFunctionBranchValue, IrFunctionParameter,
    IrInlineFunctionInput, IrText, IrTextPart, IrValue,
};

use super::names::{
    escape_string, form_binding_name, path_expression, property_access, property_key,
    safe_identifier, string_literal, ts_type,
};
use super::{render_jsdoc_type, EcmaScriptTarget, TypeModel, TypeScriptOptions};

#[cfg(test)]
pub fn form_object(entries: &[IrFormEntry], options: &TypeScriptOptions) -> String {
    form_object_for_target(entries, options, EcmaScriptTarget::TypeScript)
}

pub(super) fn form_object_for_target(
    entries: &[IrFormEntry],
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    let fields = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute {
                name,
                parameters,
                value,
            } => Some(format!(
                "{}: {}",
                property_key(name),
                value_expression_with_parameters(value, parameters, options, target)
            )),
            IrFormEntry::Branch(_) => None,
        })
        .collect::<Vec<_>>()
        .join(", ");
    let object = format!("{{ {fields} }}");
    let branches = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute { .. } => None,
            IrFormEntry::Branch(branch) => Some(branch.clone()),
        })
        .collect::<Vec<_>>();

    if branches.is_empty() {
        object
    } else {
        let dispatcher = map_expression(
            &branches,
            &[IrFunctionParameter {
                name: None,
                ty: "Plural".to_owned(),
            }],
            options,
            target,
        );
        if fields.is_empty() {
            dispatcher
        } else {
            format!("Object.assign({dispatcher}, {object})")
        }
    }
}

fn value_expression_with_parameters(
    value: &IrValue,
    parameters: &[IrFunctionParameter],
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    match value {
        IrValue::Text(text) => text_expression_for_target(text, options, target),
        IrValue::Map(branches) => map_expression(branches, parameters, options, target),
        IrValue::Object(entries) => form_object_for_target(entries, options, target),
    }
}

pub fn map_expression(
    branches: &[IrBranch],
    parameters: &[IrFunctionParameter],
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
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
    let items = branch_items(branches, &context, options, target);
    let dispatch = parameters
        .first()
        .expect("validated form maps have one dispatch parameter");
    let parameter = dispatch
        .name
        .as_deref()
        .map(safe_identifier)
        .unwrap_or_else(|| "value".to_owned());
    let selector = if dispatch.ty == "Plural" {
        format!("{}({parameter})", options.plural_function)
    } else {
        format!("String({parameter})")
    };
    let parameter_type = if dispatch.ty == "Plural" {
        "number | bigint | string".to_owned()
    } else if target.is_typescript() {
        ts_type(&dispatch.ty)
    } else {
        render_jsdoc_type(&TypeModel::from_source_name(&dispatch.ty))
    };
    if target.is_typescript() {
        format!("({parameter}: {parameter_type}) => selectBranch({selector}, {{ {items} }})")
    } else {
        format!(
            "/** @type {{({parameter}: {parameter_type}) => string}} */ (({parameter}) => selectBranch({selector}, {{ {items} }}))"
        )
    }
}

pub(super) fn text_expression_for_target(
    text: &IrText,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    text_expression_with_context_for_target(
        text,
        &BTreeMap::new(),
        &BTreeMap::new(),
        options,
        target,
    )
}

pub(super) fn text_expression_with_context_for_target(
    text: &IrText,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    let parts = text
        .parts
        .iter()
        .map(|part| match part {
            IrTextPart::Text(raw) => string_literal(raw),
            IrTextPart::Placeholder(expression) => {
                expression_string(expression, context, default_formatters, options, target)
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "\"\"".to_owned()
    } else {
        parts.join(" + ")
    }
}

fn branch_items(
    branches: &[IrBranch],
    context: &BTreeMap<String, String>,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    branches
        .iter()
        .flat_map(|branch| {
            let value = text_expression_with_context_for_target(
                &branch.value,
                context,
                &BTreeMap::new(),
                options,
                target,
            );
            if branch.keys.is_empty() {
                return vec![format!("{}: {value}", property_key("_"))];
            }
            branch
                .keys
                .iter()
                .map(|key| format!("{}: {value}", property_key(key)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn expression_string(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    let value =
        expression_value_for_target(expression, context, default_formatters, options, target);
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
    format!("String({formatted})")
}

#[cfg(test)]
fn expression_value(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
) -> String {
    expression_value_for_target(
        expression,
        context,
        default_formatters,
        options,
        EcmaScriptTarget::TypeScript,
    )
}

fn expression_value_for_target(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        return inline_function_expression(
            inputs,
            branches,
            context,
            default_formatters,
            options,
            target,
        );
    }
    if expression.path.is_empty() {
        return "\"\"".to_owned();
    }

    if expression.kind == IrExpressionKind::Call {
        if let [root] = expression.path.as_slice() {
            if is_plural_intrinsic(root) {
                return format!(
                    "{}({})",
                    options.plural_function,
                    expression
                        .arguments
                        .iter()
                        .map(|argument| {
                            expression_value_for_target(
                                argument,
                                context,
                                default_formatters,
                                options,
                                target,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{}[{}]({})",
                    form_binding_name(ty),
                    safe_identifier(root),
                    expression
                        .arguments
                        .iter()
                        .map(|argument| {
                            expression_value_for_target(
                                argument,
                                context,
                                default_formatters,
                                options,
                                target,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root, property] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{}[{}]{}({})",
                    form_binding_name(ty),
                    safe_identifier(root),
                    property_access(property),
                    expression
                        .arguments
                        .iter()
                        .map(|argument| {
                            expression_value_for_target(
                                argument,
                                context,
                                default_formatters,
                                options,
                                target,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        return format!(
            "{}({})",
            path_expression(&expression.path),
            expression
                .arguments
                .iter()
                .map(|argument| {
                    expression_value_for_target(
                        argument,
                        context,
                        default_formatters,
                        options,
                        target,
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    match expression.path.as_slice() {
        [root, property] => context.get(root).map_or_else(
            || path_expression(&expression.path),
            |ty| {
                format!(
                    "{}[{}]{}",
                    form_binding_name(ty),
                    safe_identifier(root),
                    property_access(property)
                )
            },
        ),
        [root, property, rest @ ..] => {
            let suffix = rest
                .iter()
                .map(|part| property_access(part))
                .collect::<String>();
            context.get(root).map_or_else(
                || path_expression(&expression.path),
                |ty| {
                    format!(
                        "{}[{}]{}{suffix}",
                        form_binding_name(ty),
                        safe_identifier(root),
                        property_access(property)
                    )
                },
            )
        }
        _ => path_expression(&expression.path),
    }
}

pub(super) fn function_dispatch_expression_for_target(
    parameters: &[IrFunctionParameter],
    branches: &[IrFunctionBranch],
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    let selectors = parameters
        .iter()
        .enumerate()
        .filter(|(_, parameter)| parameter.name.is_none())
        .map(|(index, parameter)| DispatchSelector {
            value: format!("__lgl_p{index}"),
            normalize_plural: parameter.ty == "Plural",
        })
        .collect::<Vec<_>>();
    dispatch_expression_level(
        &selectors,
        branches,
        0,
        context,
        default_formatters,
        options,
        target,
    )
}

#[derive(Debug)]
struct DispatchSelector {
    value: String,
    normalize_plural: bool,
}

fn inline_function_expression(
    inputs: &[IrInlineFunctionInput],
    branches: &[IrFunctionBranch],
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    let mut parameters = Vec::with_capacity(inputs.len());
    let mut arguments = Vec::with_capacity(inputs.len());
    let mut selectors = Vec::new();
    let mut branch_context = context.clone();
    let mut branch_formatters = default_formatters.clone();

    for (index, input) in inputs.iter().enumerate() {
        match input {
            IrInlineFunctionInput::Selector { value, .. } => {
                let parameter = format!("__lgl_inline_selector_{index}");
                parameters.push(parameter.clone());
                arguments.push(expression_value_for_target(
                    value,
                    context,
                    default_formatters,
                    options,
                    target,
                ));
                // Explicit `Plural(value)` has already produced its category.
                // A directly referenced `Plural` value still accepts either a
                // numeric operand or a pre-classified category, matching named
                // function dispatch.
                selectors.push(DispatchSelector {
                    value: parameter,
                    normalize_plural: inferred_expression_type(value, context).as_deref()
                        == Some("Plural")
                        && !is_plural_intrinsic_call(value),
                });
            }
            IrInlineFunctionInput::Binding { name, value, .. } => {
                parameters.push(safe_identifier(name));
                arguments.push(expression_value_for_target(
                    value,
                    context,
                    default_formatters,
                    options,
                    target,
                ));
                branch_context.insert(
                    name.clone(),
                    inferred_expression_type(value, context).unwrap_or_else(|| "String".to_owned()),
                );
                if value.formatters.is_empty()
                    && value.kind == IrExpressionKind::Reference
                    && value.path.len() == 1
                {
                    if let Some(formatters) = default_formatters.get(&value.path[0]) {
                        branch_formatters.insert(name.clone(), formatters.clone());
                    }
                }
            }
        }
    }

    let body = dispatch_expression_level(
        &selectors,
        branches,
        0,
        &branch_context,
        &branch_formatters,
        options,
        target,
    );
    format!(
        "(({}) => {body})({})",
        parameters.join(", "),
        arguments.join(", ")
    )
}

fn is_plural_intrinsic_call(expression: &IrExpression) -> bool {
    expression.kind == IrExpressionKind::Call
        && expression.path.len() == 1
        && is_plural_intrinsic(&expression.path[0])
}

fn inferred_expression_type(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
) -> Option<String> {
    match &expression.kind {
        IrExpressionKind::InlineFunction { .. } => Some("String".to_owned()),
        IrExpressionKind::Call => expression
            .path
            .first()
            .filter(|_| expression.path.len() == 1)
            .filter(|name| is_plural_intrinsic(name))
            .map(|_| "Plural".to_owned())
            .or_else(|| Some("String".to_owned())),
        IrExpressionKind::Reference if expression.path.len() == 1 => {
            context.get(&expression.path[0]).cloned()
        }
        IrExpressionKind::Reference => None,
    }
}

fn dispatch_expression_level(
    selectors: &[DispatchSelector],
    branches: &[IrFunctionBranch],
    depth: usize,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
    target: EcmaScriptTarget,
) -> String {
    let selector = selectors.get(depth).map_or_else(
        || "undefined".to_owned(),
        |selector| {
            if selector.normalize_plural {
                format!("{}({})", options.plural_function, selector.value)
            } else {
                format!("String({})", selector.value)
            }
        },
    );
    let items = branches
        .iter()
        .map(|branch| {
            let value = match &branch.value {
                IrFunctionBranchValue::Text(text) => text_expression_with_context_for_target(
                    text,
                    context,
                    default_formatters,
                    options,
                    target,
                ),
                IrFunctionBranchValue::Dispatch(children) => dispatch_expression_level(
                    selectors,
                    children,
                    depth + 1,
                    context,
                    default_formatters,
                    options,
                    target,
                ),
            };
            let callback = if target.is_typescript() {
                format!("(): string => {value}")
            } else {
                format!("/** @type {{() => string}} */ (() => {value})")
            };
            format!("{}: {callback}", property_key(&branch.key))
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("selectBranch({selector}, {{ {items} }})()")
}

fn apply_formatters(value: String, formatters: &[IrFormatter]) -> String {
    formatters.iter().fold(value, |current, formatter| {
        let formatter_options = formatter_options(formatter);
        match &formatter.kind {
            IrFormatterKind::Number => format!("formatNumber({current})"),
            IrFormatterKind::Currency => {
                let currency = formatter
                    .arguments
                    .iter()
                    .find(|argument| argument.name == "code")
                    .map_or("USD", |argument| argument.value.as_str());
                let fraction = compiled_currency_fraction(currency)
                    .expect("validated currency formatter must have CLDR fraction rules");
                format!(
                    "formatCurrency({current}, {}, {}, {formatter_options})",
                    fraction.digits, fraction.rounding
                )
            }
            IrFormatterKind::Date => format!("formatDate({current}, {formatter_options})"),
            IrFormatterKind::Unknown(_) => current,
        }
    })
}

fn formatter_options(formatter: &IrFormatter) -> String {
    let items = formatter
        .arguments
        .iter()
        .map(|argument| {
            format!(
                "{}: \"{}\"",
                property_key(&argument.name),
                escape_string(&argument.value)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {items} }}")
}

#[cfg(test)]
mod tests {
    use super::{expression_value, form_object, TypeScriptOptions};
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
        let options = TypeScriptOptions::default();

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["ready"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "ready()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["ready"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "ready"
        );
    }

    #[test]
    fn canonical_plural_call_uses_configured_function() {
        let context = BTreeMap::new();
        let options = TypeScriptOptions {
            plural_function: "selectPlural".to_owned(),
            ..TypeScriptOptions::default()
        };

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["Plural"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "selectPlural()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["plural"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "plural()"
        );
    }

    #[test]
    fn zero_argument_form_calls_differ_from_form_references() {
        let context = BTreeMap::from([("item".to_owned(), "Item".to_owned())]);
        let options = TypeScriptOptions::default();

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["item"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "__lgl_form_4974656D[item]()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["item"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "item"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["item", "label"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "__lgl_form_4974656D[item].label()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["item", "label"]),
                &context,
                &BTreeMap::new(),
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

        let emitted = form_object(entries, &TypeScriptOptions::default());

        assert!(emitted.contains("label: (gender: Gender) => selectBranch(String(gender),"));
        assert!(!emitted.contains("pluralEn(gender)"));
    }
}
