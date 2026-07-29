use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use linguini_cldr::{
    built_in_plural_rules, compiled_currency_formatting, compiled_date_formatting,
    compiled_number_formatting, DateFormatData, NumberFormatData, NumberPattern,
};
use linguini_ir::{
    IrBranch, IrExpression, IrExpressionKind, IrForm, IrFormEntry, IrFormatter,
    IrFormatterArgument, IrFormatterKind, IrFunction, IrFunctionBranch, IrFunctionBranchValue,
    IrModule, IrText, IrTextPart, IrValue,
};

use super::SampleValue;

const MAX_EVALUATION_DEPTH: usize = 128;
const MAX_DECIMAL_DIGITS: usize = 8_192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RenderError {
    MissingSymbol {
        kind: &'static str,
        name: String,
    },
    MissingBody {
        message: String,
    },
    MissingForm {
        parameter: String,
        ty: String,
    },
    MissingVariant {
        form: String,
        variant: String,
    },
    MissingProperty {
        form: String,
        variant: String,
        path: String,
    },
    MissingBranch {
        owner: String,
        selector: String,
    },
    InvalidArity {
        function: String,
        expected: usize,
        actual: usize,
    },
    InvalidNumber {
        value: String,
    },
    InvalidDate {
        value: String,
    },
    AliasCycle {
        chain: Vec<String>,
    },
    Unsupported {
        detail: String,
    },
    EvaluationDepthExceeded,
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSymbol { kind, name } => {
                write!(formatter, "{kind} `{name}` is missing")
            }
            Self::MissingBody { message } => {
                write!(formatter, "message `{message}` has no renderable body")
            }
            Self::MissingForm { parameter, ty } => write!(
                formatter,
                "form `{ty}` for parameter `{parameter}` is missing"
            ),
            Self::MissingVariant { form, variant } => {
                write!(formatter, "form `{form}` has no variant `{variant}`")
            }
            Self::MissingProperty {
                form,
                variant,
                path,
            } => write!(
                formatter,
                "form `{form}` variant `{variant}` has no property `{path}`"
            ),
            Self::MissingBranch { owner, selector } => write!(
                formatter,
                "{owner} has no branch matching selector `{selector}` and no fallback"
            ),
            Self::InvalidArity {
                function,
                expected,
                actual,
            } => write!(
                formatter,
                "function `{function}` expected {expected} arguments but received {actual}"
            ),
            Self::InvalidNumber { value } => {
                write!(formatter, "`{value}` is not a supported decimal number")
            }
            Self::InvalidDate { value } => {
                write!(formatter, "`{value}` is not a valid YYYY-MM-DD date")
            }
            Self::AliasCycle { chain } => {
                write!(formatter, "formatter alias cycle: {}", chain.join(" -> "))
            }
            Self::Unsupported { detail } => formatter.write_str(detail),
            Self::EvaluationDepthExceeded => write!(
                formatter,
                "evaluation exceeded the {MAX_EVALUATION_DEPTH}-step recursion limit"
            ),
        }
    }
}

pub(super) struct Renderer<'a> {
    schema: &'a IrModule,
    module: &'a IrModule,
    locale: &'a str,
}

impl<'a> Renderer<'a> {
    pub(super) fn new(schema: &'a IrModule, module: &'a IrModule, locale: &'a str) -> Self {
        Self {
            schema,
            module,
            locale,
        }
    }

    pub(super) fn render_message(
        &self,
        name: &str,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> Result<String, RenderError> {
        let message = self
            .module
            .messages
            .iter()
            .find(|message| message.name == name)
            .ok_or_else(|| RenderError::MissingSymbol {
                kind: "message implementation",
                name: name.to_owned(),
            })?;
        let body = message
            .body
            .as_ref()
            .ok_or_else(|| RenderError::MissingBody {
                message: name.to_owned(),
            })?;
        let context = self.context(name)?;
        self.render_text(body, &context, inputs, 0)
    }

    fn render_text(
        &self,
        text: &IrText,
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        let mut output = String::new();
        for part in &text.parts {
            match part {
                IrTextPart::Text(value) => output.push_str(value),
                IrTextPart::Placeholder(expression) => output.push_str(&self.eval_expression(
                    expression,
                    context,
                    inputs,
                    depth + 1,
                )?),
            }
        }
        Ok(output)
    }

    fn eval_expression(
        &self,
        expression: &IrExpression,
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        let value = self.eval_expression_value(expression, context, inputs, depth)?;
        self.apply_formatters(value, expression, context)
    }

    fn eval_expression_value(
        &self,
        expression: &IrExpression,
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        if expression.path.is_empty() {
            return Err(RenderError::Unsupported {
                detail: "expression path is empty".to_owned(),
            });
        }

        let args = expression
            .arguments
            .iter()
            .map(|argument| self.eval_expression_value(argument, context, inputs, depth + 1))
            .collect::<Result<Vec<_>, _>>()?;

        match expression.path.as_slice() {
            [root] if expression.kind == IrExpressionKind::Call && context.contains_key(root) => {
                self.eval_form_call(root, None, &args, context, inputs, depth + 1)
            }
            [root, property]
                if expression.kind == IrExpressionKind::Call && context.contains_key(root) =>
            {
                self.eval_form_call(
                    root,
                    Some(property.as_str()),
                    &args,
                    context,
                    inputs,
                    depth + 1,
                )
            }
            [function] if expression.kind == IrExpressionKind::Call && function == "plural" => {
                if args.len() != 1 {
                    return Err(RenderError::InvalidArity {
                        function: function.clone(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                Ok(plural_key(self.locale, &args[0]))
            }
            [function] if expression.kind == IrExpressionKind::Call => {
                self.eval_function(function, &args, context, inputs, depth + 1)
            }
            [root] => {
                if let Some(value) = inputs.get(root) {
                    return Ok(value.as_text());
                }
                let variable = self
                    .module
                    .variables
                    .iter()
                    .find(|variable| variable.name == *root)
                    .ok_or_else(|| RenderError::MissingSymbol {
                        kind: "input or variable",
                        name: root.clone(),
                    })?;
                self.render_text(&variable.value, context, inputs, depth + 1)
            }
            [root, property] if context.contains_key(root) => {
                self.eval_form_property(root, &[property.as_str()], context, inputs, depth + 1)
            }
            [root, tail @ ..] if context.contains_key(root) => {
                let path = tail.iter().map(String::as_str).collect::<Vec<_>>();
                self.eval_form_property(root, &path, context, inputs, depth + 1)
            }
            _ => Err(RenderError::MissingSymbol {
                kind: "expression",
                name: expression.path.join("."),
            }),
        }
    }

    fn eval_form_call(
        &self,
        root: &str,
        property: Option<&str>,
        args: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        let form = self.form_for(root, context)?;
        let variant_name = inputs.get(root).map(SampleValue::as_text).ok_or_else(|| {
            RenderError::MissingSymbol {
                kind: "form input",
                name: root.to_owned(),
            }
        })?;
        let variant = form
            .variants
            .iter()
            .find(|item| item.name == variant_name)
            .ok_or_else(|| RenderError::MissingVariant {
                form: form.name.clone(),
                variant: variant_name.clone(),
            })?;

        if let Some(property) = property {
            let (parameters, value) =
                find_entry_value(&variant.entries, &[property]).ok_or_else(|| {
                    RenderError::MissingProperty {
                        form: form.name.clone(),
                        variant: variant_name,
                        path: property.to_owned(),
                    }
                })?;
            return self.eval_ir_value(
                value,
                args.first().map(String::as_str),
                parameters
                    .first()
                    .map_or(true, |parameter| parameter.ty == "Plural"),
                context,
                inputs,
                depth + 1,
            );
        }

        let branches = variant
            .entries
            .iter()
            .filter_map(|entry| match entry {
                IrFormEntry::Branch(branch) => Some(branch),
                IrFormEntry::Attribute { .. } => None,
            })
            .collect::<Vec<_>>();
        select_branch(
            args.first().map(String::as_str).unwrap_or("other"),
            &branches,
            true,
            self.locale,
            &format!("form `{}` variant `{variant_name}`", form.name),
            context,
            inputs,
            self,
            depth + 1,
        )
    }

    fn eval_form_property(
        &self,
        root: &str,
        path: &[&str],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        let form = self.form_for(root, context)?;
        let variant_name = inputs.get(root).map(SampleValue::as_text).ok_or_else(|| {
            RenderError::MissingSymbol {
                kind: "form input",
                name: root.to_owned(),
            }
        })?;
        let variant = form
            .variants
            .iter()
            .find(|item| item.name == variant_name)
            .ok_or_else(|| RenderError::MissingVariant {
                form: form.name.clone(),
                variant: variant_name.clone(),
            })?;
        let (parameters, value) = find_entry_value(&variant.entries, path).ok_or_else(|| {
            RenderError::MissingProperty {
                form: form.name.clone(),
                variant: variant_name,
                path: path.join("."),
            }
        })?;
        self.eval_ir_value(
            value,
            None,
            parameters
                .first()
                .map_or(true, |parameter| parameter.ty == "Plural"),
            context,
            inputs,
            depth + 1,
        )
    }

    fn eval_ir_value(
        &self,
        value: &IrValue,
        selector: Option<&str>,
        plural_selector: bool,
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        match value {
            IrValue::Text(text) => self.render_text(text, context, inputs, depth + 1),
            IrValue::Map(branches) => {
                let branches = branches.iter().collect::<Vec<_>>();
                select_branch(
                    selector.unwrap_or("other"),
                    &branches,
                    plural_selector,
                    self.locale,
                    "form value",
                    context,
                    inputs,
                    self,
                    depth + 1,
                )
            }
            IrValue::Object(_) => Err(RenderError::Unsupported {
                detail: "object-valued form property cannot be rendered as text".to_owned(),
            }),
        }
    }

    fn eval_function(
        &self,
        name: &str,
        args: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        let function = self
            .module
            .functions
            .iter()
            .find(|function| function.name == name)
            .ok_or_else(|| RenderError::MissingSymbol {
                kind: "function",
                name: name.to_owned(),
            })?;
        self.eval_dispatch(
            function,
            &function.branches,
            0,
            args,
            context,
            inputs,
            depth + 1,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn eval_dispatch(
        &self,
        function: &IrFunction,
        branches: &[IrFunctionBranch],
        dispatch_depth: usize,
        args: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        let parameter_index = dispatch_parameter_indices(function)
            .get(dispatch_depth)
            .copied()
            .unwrap_or(dispatch_depth);
        let selector = if function.parameters.is_empty() {
            "undefined"
        } else {
            args.get(parameter_index)
                .map(String::as_str)
                .ok_or_else(|| RenderError::InvalidArity {
                    function: function.name.clone(),
                    expected: parameter_index + 1,
                    actual: args.len(),
                })?
        };
        let key = function
            .parameters
            .get(parameter_index)
            .filter(|parameter| parameter.ty == "Plural")
            .map(|_| plural_key(self.locale, selector))
            .unwrap_or_else(|| selector.to_owned());
        let branch =
            matching_function_branch(branches, &key).ok_or_else(|| RenderError::MissingBranch {
                owner: format!("function `{}`", function.name),
                selector: key,
            })?;
        match &branch.value {
            IrFunctionBranchValue::Text(text) => self.render_text(text, context, inputs, depth + 1),
            IrFunctionBranchValue::Dispatch(branches) => self.eval_dispatch(
                function,
                branches,
                dispatch_depth + 1,
                args,
                context,
                inputs,
                depth + 1,
            ),
        }
    }

    fn form_for(
        &self,
        root: &str,
        context: &BTreeMap<String, String>,
    ) -> Result<&'a IrForm, RenderError> {
        let ty = context
            .get(root)
            .ok_or_else(|| RenderError::MissingSymbol {
                kind: "message parameter",
                name: root.to_owned(),
            })?;
        self.module
            .forms
            .iter()
            .find(|form| form.name == *ty)
            .ok_or_else(|| RenderError::MissingForm {
                parameter: root.to_owned(),
                ty: ty.clone(),
            })
    }

    fn context(&self, message_name: &str) -> Result<BTreeMap<String, String>, RenderError> {
        self.schema
            .messages
            .iter()
            .find(|message| message.name == message_name)
            .map(|message| {
                message
                    .parameters
                    .iter()
                    .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                    .collect()
            })
            .ok_or_else(|| RenderError::MissingSymbol {
                kind: "schema message",
                name: message_name.to_owned(),
            })
    }

    fn apply_formatters(
        &self,
        value: String,
        expression: &IrExpression,
        context: &BTreeMap<String, String>,
    ) -> Result<String, RenderError> {
        let default_formatters;
        let formatters = if expression.formatters.is_empty() {
            default_formatters = expression
                .path
                .first()
                .and_then(|name| context.get(name))
                .map(|ty| self.default_type_formatters(ty))
                .transpose()?
                .unwrap_or_default();
            default_formatters.as_slice()
        } else {
            expression.formatters.as_slice()
        };

        formatters.iter().try_fold(value, |current, formatter| {
            self.apply_formatter(&current, formatter)
        })
    }

    fn default_type_formatters(&self, ty: &str) -> Result<Vec<IrFormatter>, RenderError> {
        let mut current = ty;
        let mut visited = BTreeSet::new();
        let mut chain = Vec::new();

        loop {
            if !visited.insert(current.to_owned()) {
                chain.push(current.to_owned());
                return Err(RenderError::AliasCycle { chain });
            }
            chain.push(current.to_owned());

            if let Some(alias) = self
                .schema
                .type_aliases
                .iter()
                .find(|alias| alias.name == current)
            {
                if !alias.formatters.is_empty() {
                    return Ok(alias.formatters.clone());
                }
                current = &alias.target;
                continue;
            }

            let kind = match current {
                "Number" | "Decimal" => Some(IrFormatterKind::Number),
                "Date" => Some(IrFormatterKind::Date),
                "String" | "Boolean" => None,
                _ => None,
            };
            return Ok(kind
                .map(|kind| {
                    vec![IrFormatter {
                        kind,
                        arguments: Vec::new(),
                    }]
                })
                .unwrap_or_default());
        }
    }

    fn apply_formatter(&self, value: &str, formatter: &IrFormatter) -> Result<String, RenderError> {
        match &formatter.kind {
            IrFormatterKind::Number => self.format_number(value),
            IrFormatterKind::Currency => self.format_currency(value, &formatter.arguments),
            IrFormatterKind::Date => self.format_date(value, &formatter.arguments),
            IrFormatterKind::Unknown(name) => Err(RenderError::Unsupported {
                detail: format!("formatter `{name}` cannot be previewed"),
            }),
        }
    }

    fn format_number(&self, value: &str) -> Result<String, RenderError> {
        let Some(numbers) = compiled_number_formatting(self.locale) else {
            return Ok(value.to_owned());
        };
        format_number_pattern(value, &numbers.decimal_pattern, &numbers, None)
    }

    fn format_currency(
        &self,
        value: &str,
        arguments: &[IrFormatterArgument],
    ) -> Result<String, RenderError> {
        let code = formatter_argument(arguments, "code").unwrap_or("USD");
        let accounting = formatter_argument(arguments, "accounting") == Some("true");
        let (Some(numbers), Some(currency)) = (
            compiled_number_formatting(self.locale),
            compiled_currency_formatting(self.locale),
        ) else {
            return Ok(format!("{code} {value}"));
        };
        let pattern = if accounting {
            currency
                .accounting_pattern
                .as_ref()
                .unwrap_or(&currency.standard_pattern)
        } else {
            &currency.standard_pattern
        };
        let symbol = currency_symbol(code);
        format_number_pattern(value, pattern, &numbers, Some(&symbol))
    }

    fn format_date(
        &self,
        value: &str,
        arguments: &[IrFormatterArgument],
    ) -> Result<String, RenderError> {
        let Some(dates) = compiled_date_formatting(self.locale) else {
            return Ok(value.to_owned());
        };
        let pattern = match formatter_argument(arguments, "style").unwrap_or("medium") {
            "full" => dates.date_formats.full,
            "long" => dates.date_formats.long,
            "short" => dates.date_formats.short,
            _ => dates.date_formats.medium,
        };
        format_date_pattern(value, pattern, &dates)
    }
}

fn formatter_argument<'a>(arguments: &'a [IrFormatterArgument], name: &str) -> Option<&'a str> {
    arguments
        .iter()
        .find(|argument| argument.name == name)
        .map(|argument| argument.value.as_str())
}

fn find_entry_value<'a>(
    entries: &'a [IrFormEntry],
    path: &[&str],
) -> Option<(&'a [linguini_ir::IrFunctionParameter], &'a IrValue)> {
    let (head, tail) = path.split_first()?;
    for entry in entries {
        if let IrFormEntry::Attribute {
            name,
            parameters,
            value,
        } = entry
        {
            if name == head {
                return if tail.is_empty() {
                    Some((parameters, value))
                } else if let IrValue::Object(entries) = value {
                    find_entry_value(entries, tail)
                } else {
                    None
                };
            }
        }
    }
    None
}

fn matching_function_branch<'a>(
    branches: &'a [IrFunctionBranch],
    key: &str,
) -> Option<&'a IrFunctionBranch> {
    branches
        .iter()
        .find(|branch| branch.key == key)
        .or_else(|| branches.iter().find(|branch| branch.key == "_"))
}

fn dispatch_parameter_indices(function: &IrFunction) -> Vec<usize> {
    function
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| (parameter.ty != "String").then_some(index))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn select_branch(
    selector: &str,
    branches: &[&IrBranch],
    plural_selector: bool,
    locale: &str,
    owner: &str,
    context: &BTreeMap<String, String>,
    inputs: &BTreeMap<String, SampleValue>,
    renderer: &Renderer<'_>,
    depth: usize,
) -> Result<String, RenderError> {
    ensure_depth(depth)?;
    let key = if plural_selector {
        plural_key(locale, selector)
    } else {
        selector.to_owned()
    };
    let branch = branches
        .iter()
        .copied()
        .find(|branch| branch.keys.iter().any(|candidate| candidate == &key))
        .or_else(|| {
            branches
                .iter()
                .copied()
                .find(|branch| branch.keys.iter().any(|candidate| candidate == "_"))
        })
        .or_else(|| {
            branches
                .iter()
                .copied()
                .find(|branch| branch.keys.iter().any(|candidate| candidate == "other"))
        })
        .ok_or_else(|| RenderError::MissingBranch {
            owner: owner.to_owned(),
            selector: key,
        })?;
    renderer.render_text(&branch.value, context, inputs, depth + 1)
}

fn plural_key(locale: &str, selector: &str) -> String {
    if let Some(rules) = built_in_plural_rules(locale) {
        if let Ok(category) = rules.category_for(selector) {
            return category.to_owned();
        }
    }
    selector.to_owned()
}

fn ensure_depth(depth: usize) -> Result<(), RenderError> {
    if depth > MAX_EVALUATION_DEPTH {
        Err(RenderError::EvaluationDepthExceeded)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecimalParts {
    negative: bool,
    integer: String,
    fraction: String,
}

fn parse_decimal(value: &str) -> Result<DecimalParts, RenderError> {
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |value| (true, value));
    let unsigned = unsigned.strip_prefix('+').unwrap_or(unsigned);
    let (mantissa, exponent) =
        unsigned
            .split_once(['e', 'E'])
            .map_or((unsigned, 0i32), |(mantissa, exponent)| {
                let exponent = exponent.parse::<i32>().unwrap_or(i32::MAX);
                (mantissa, exponent)
            });
    if exponent.unsigned_abs() as usize > MAX_DECIMAL_DIGITS {
        return Err(RenderError::InvalidNumber {
            value: value.to_owned(),
        });
    }

    let mut parts = mantissa.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || (integer.is_empty() && fraction.is_empty())
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(RenderError::InvalidNumber {
            value: value.to_owned(),
        });
    }

    let mut digits = format!("{integer}{fraction}");
    if digits.len() > MAX_DECIMAL_DIGITS {
        return Err(RenderError::InvalidNumber {
            value: value.to_owned(),
        });
    }
    if digits.is_empty() {
        digits.push('0');
    }
    let decimal_position = i32::try_from(integer.len())
        .ok()
        .and_then(|position| position.checked_add(exponent))
        .ok_or_else(|| RenderError::InvalidNumber {
            value: value.to_owned(),
        })?;

    let (mut integer, fraction) = if decimal_position <= 0 {
        let zero_count =
            usize::try_from(-decimal_position).map_err(|_| RenderError::InvalidNumber {
                value: value.to_owned(),
            })?;
        (
            "0".to_owned(),
            format!("{}{digits}", "0".repeat(zero_count)),
        )
    } else {
        let position =
            usize::try_from(decimal_position).map_err(|_| RenderError::InvalidNumber {
                value: value.to_owned(),
            })?;
        if position >= digits.len() {
            (
                format!("{digits}{}", "0".repeat(position - digits.len())),
                String::new(),
            )
        } else {
            (digits[..position].to_owned(), digits[position..].to_owned())
        }
    };
    let trimmed = integer.trim_start_matches('0');
    integer = if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    };

    Ok(DecimalParts {
        negative,
        integer,
        fraction,
    })
}

fn format_number_pattern(
    value: &str,
    pattern: &NumberPattern,
    numbers: &NumberFormatData,
    currency_symbol: Option<&str>,
) -> Result<String, RenderError> {
    let mut decimal = parse_decimal(value)?;
    round_fraction(
        &mut decimal.integer,
        &mut decimal.fraction,
        usize::from(pattern.positive.max_fraction_digits),
    );
    while decimal.fraction.len() > usize::from(pattern.positive.min_fraction_digits)
        && decimal.fraction.ends_with('0')
    {
        decimal.fraction.pop();
    }
    while decimal.fraction.len() < usize::from(pattern.positive.min_fraction_digits) {
        decimal.fraction.push('0');
    }
    while decimal.integer.len() < usize::from(pattern.positive.min_integer_digits) {
        decimal.integer.insert(0, '0');
    }

    let grouped = group_integer_digits(
        &decimal.integer,
        pattern.positive.primary_group_size,
        pattern.positive.secondary_group_size,
        numbers.group_symbol,
    );
    let formatted = if decimal.fraction.is_empty() {
        grouped
    } else {
        format!("{grouped}{}{}", numbers.decimal_symbol, decimal.fraction)
    };

    let (prefix, suffix) = if decimal.negative {
        pattern.negative.as_ref().map_or_else(
            || {
                (
                    format!(
                        "-{}",
                        render_affix(pattern.positive.prefix, currency_symbol)
                    ),
                    render_affix(pattern.positive.suffix, currency_symbol),
                )
            },
            |part| {
                (
                    render_affix(part.prefix, currency_symbol),
                    render_affix(part.suffix, currency_symbol),
                )
            },
        )
    } else {
        (
            render_affix(pattern.positive.prefix, currency_symbol),
            render_affix(pattern.positive.suffix, currency_symbol),
        )
    };
    Ok(format!("{prefix}{formatted}{suffix}"))
}

fn round_fraction(integer: &mut String, fraction: &mut String, max_digits: usize) {
    if fraction.len() <= max_digits {
        while fraction.len() < max_digits {
            fraction.push('0');
        }
        return;
    }

    let round_up = fraction.as_bytes()[max_digits] >= b'5';
    fraction.truncate(max_digits);
    if !round_up {
        return;
    }

    for index in (0..fraction.len()).rev() {
        let digit = fraction.as_bytes()[index];
        if digit < b'9' {
            fraction.replace_range(index..=index, &char::from(digit + 1).to_string());
            return;
        }
        fraction.replace_range(index..=index, "0");
    }
    increment_integer(integer);
}

fn increment_integer(integer: &mut String) {
    for index in (0..integer.len()).rev() {
        let digit = integer.as_bytes()[index];
        if digit < b'9' {
            integer.replace_range(index..=index, &char::from(digit + 1).to_string());
            return;
        }
        integer.replace_range(index..=index, "0");
    }
    integer.insert(0, '1');
}

fn group_integer_digits(
    integer: &str,
    primary_group_size: Option<u8>,
    secondary_group_size: Option<u8>,
    group_symbol: &str,
) -> String {
    let Some(primary) = primary_group_size.map(usize::from) else {
        return integer.to_owned();
    };
    if primary == 0 || integer.len() <= primary {
        return integer.to_owned();
    }

    let secondary = secondary_group_size.map(usize::from).unwrap_or(primary);
    let mut groups = Vec::new();
    let mut end = integer.len();
    let mut size = primary;
    while end > 0 {
        let start = end.saturating_sub(size);
        groups.push(&integer[start..end]);
        end = start;
        size = secondary;
    }
    groups.reverse();
    groups.join(group_symbol)
}

fn render_affix(value: &str, currency_symbol: Option<&str>) -> String {
    currency_symbol.map_or_else(
        || value.to_owned(),
        |symbol| value.replace('\u{a4}', symbol),
    )
}

fn currency_symbol(code: &str) -> String {
    match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" | "CNY" => "¥",
        "RUB" => "₽",
        "INR" => "₹",
        "KRW" => "₩",
        _ => code,
    }
    .to_owned()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DateParts {
    year: u32,
    month: u32,
    day: u32,
    weekday: usize,
}

fn parse_date(value: &str) -> Result<DateParts, RenderError> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(RenderError::InvalidDate {
            value: value.to_owned(),
        });
    }
    let year = value[0..4]
        .parse::<u32>()
        .map_err(|_| RenderError::InvalidDate {
            value: value.to_owned(),
        })?;
    let month = value[5..7]
        .parse::<u32>()
        .map_err(|_| RenderError::InvalidDate {
            value: value.to_owned(),
        })?;
    let day = value[8..10]
        .parse::<u32>()
        .map_err(|_| RenderError::InvalidDate {
            value: value.to_owned(),
        })?;
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > max_day {
        return Err(RenderError::InvalidDate {
            value: value.to_owned(),
        });
    }

    Ok(DateParts {
        year,
        month,
        day,
        weekday: weekday(year, month, day),
    })
}

const fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn weekday(year: u32, month: u32, day: u32) -> usize {
    const OFFSETS: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut year = i64::from(year);
    if month < 3 {
        year -= 1;
    }
    let value = year + year / 4 - year / 100
        + year / 400
        + OFFSETS[usize::try_from(month - 1).expect("validated month")]
        + i64::from(day);
    usize::try_from(value.rem_euclid(7)).expect("weekday is non-negative")
}

fn format_date_pattern(
    value: &str,
    pattern: &str,
    dates: &DateFormatData,
) -> Result<String, RenderError> {
    let date = parse_date(value)?;
    let mut output = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' {
            for next in chars.by_ref() {
                if next == '\'' {
                    break;
                }
                output.push(next);
            }
            continue;
        }
        if matches!(character, 'y' | 'M' | 'L' | 'd' | 'E') {
            let mut width = 1;
            while chars.peek() == Some(&character) {
                chars.next();
                width += 1;
            }
            output.push_str(&date_field(character, width, date, dates));
            continue;
        }
        output.push(character);
    }
    Ok(output)
}

fn date_field(field: char, width: usize, date: DateParts, dates: &DateFormatData) -> String {
    match field {
        'y' if width == 2 => format!("{:02}", date.year % 100),
        'y' => date.year.to_string(),
        'M' | 'L' if width >= 4 => {
            dates.months.wide[usize::try_from(date.month - 1).expect("validated month")].to_owned()
        }
        'M' | 'L' if width == 3 => dates.months.abbreviated
            [usize::try_from(date.month - 1).expect("validated month")]
        .to_owned(),
        'M' | 'L' if width == 2 => format!("{:02}", date.month),
        'M' | 'L' => date.month.to_string(),
        'd' if width == 2 => format!("{:02}", date.day),
        'd' => date.day.to_string(),
        'E' if width >= 4 => dates.weekdays.wide[date.weekday].to_owned(),
        'E' => dates.weekdays.abbreviated[date.weekday].to_owned(),
        _ => unreachable!("date pattern dispatch only accepts supported date fields"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale, parse_schema};

    fn renderer<'a>(
        schema: &'a IrModule,
        locale: &'a IrModule,
        locale_name: &'a str,
    ) -> Renderer<'a> {
        Renderer::new(schema, locale, locale_name)
    }

    #[test]
    fn missing_input_is_reported_instead_of_hidden() {
        let schema = lower_schema(&parse_schema("hello(name: String)\n").expect("schema"));
        let locale = lower_locale(&parse_locale("hello = Hello, {name}\n").expect("locale"));

        let error = renderer(&schema, &locale, "en")
            .render_message("hello", &BTreeMap::new())
            .expect_err("missing input must fail");

        assert_eq!(
            error,
            RenderError::MissingSymbol {
                kind: "input or variable",
                name: "name".to_owned()
            }
        );
    }

    #[test]
    fn zero_argument_calls_are_not_rendered_as_references() {
        let schema = lower_schema(&parse_schema("hello\n").expect("schema"));
        let locale = lower_locale(
            &parse_locale("fn ready() { _ => Ready }\nhello = {ready()}\n").expect("locale"),
        );

        let rendered = renderer(&schema, &locale, "en")
            .render_message("hello", &BTreeMap::new())
            .expect("zero-argument call renders");

        assert_eq!(rendered, "Ready");
    }

    #[test]
    fn formatter_alias_cycles_are_reported() {
        let schema = lower_schema(
            &parse_schema("type A = B\ntype B = A\nhello(value: A)\n").expect("schema"),
        );
        let locale = lower_locale(&parse_locale("hello = {value}\n").expect("locale"));
        let inputs = BTreeMap::from([("value".to_owned(), SampleValue::String("x".to_owned()))]);

        let error = renderer(&schema, &locale, "en")
            .render_message("hello", &inputs)
            .expect_err("alias cycle must fail");

        assert!(matches!(error, RenderError::AliasCycle { .. }));
    }

    #[test]
    fn arbitrary_precision_decimal_rounding_is_bounded() {
        let numbers = compiled_number_formatting("en").expect("number data");
        let rendered = format_number_pattern(
            "1234567890.123456789",
            &numbers.decimal_pattern,
            &numbers,
            None,
        )
        .expect("format number");

        assert_eq!(rendered, "1,234,567,890.123");
    }

    #[test]
    fn date_formatter_uses_compiled_locale_pattern() {
        let dates = compiled_date_formatting("en").expect("date data");
        let rendered = format_date_pattern("2026-05-13", dates.date_formats.full, &dates)
            .expect("format date");

        assert!(rendered.contains("2026"));
        assert!(rendered.contains("May"));
        assert!(rendered.contains("Wednesday"));
    }

    #[test]
    fn schema_formatter_aliases_use_locale_data() {
        let schema = lower_schema(
            &parse_schema(
                "type Price = Decimal @currency(code = \"EUR\")\ntype ShortDate = Date @date(style = \"short\")\nsummary(price: Price, created: ShortDate)\n",
            )
            .expect("schema"),
        );
        let locale =
            lower_locale(&parse_locale("summary = {price} on {created}\n").expect("locale"));
        let inputs = BTreeMap::from([
            ("price".to_owned(), SampleValue::Number("1234.5".to_owned())),
            (
                "created".to_owned(),
                SampleValue::String("2026-05-13".to_owned()),
            ),
        ]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect("render message");

        assert!(rendered.contains('€'), "{rendered}");
        assert!(!rendered.contains("2026-05-13"), "{rendered}");
    }
}
