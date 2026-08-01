use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use linguini_cldr::{
    built_in_plural_rules, compiled_currency_formatting, compiled_currency_fraction,
    compiled_date_formatting, compiled_number_formatting, DateFormatData, NumberFormatData,
    NumberPattern, Operand, PluralRule, PluralRules, Range, Relation, RelationOperator,
};
use linguini_ir::{
    is_plural_intrinsic, IrBranch, IrExpression, IrExpressionKind, IrForm, IrFormEntry,
    IrFormatter, IrFormatterArgument, IrFormatterKind, IrFunctionBranch, IrFunctionBranchValue,
    IrFunctionParameter, IrInlineFunctionInput, IrModule, IrText, IrTextPart, IrValue,
};

use super::SampleValue;

const MAX_EVALUATION_DEPTH: usize = 128;
const MAX_DECIMAL_DIGITS: usize = 8_192;
type ParameterScope = (BTreeMap<String, String>, BTreeMap<String, SampleValue>);

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
        if let IrExpressionKind::InlineFunction {
            inputs: inline_inputs,
            branches,
        } = &expression.kind
        {
            return self.eval_inline_function(inline_inputs, branches, context, inputs, depth + 1);
        }
        let args = expression
            .arguments
            .iter()
            .map(|argument| self.eval_expression_value(argument, context, inputs, depth + 1))
            .collect::<Result<Vec<_>, _>>()?;
        if expression.path.is_empty() {
            return Err(RenderError::Unsupported {
                detail: "expression path is empty".to_owned(),
            });
        }

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
            [function]
                if expression.kind == IrExpressionKind::Call && is_plural_intrinsic(function) =>
            {
                if args.len() != 1 {
                    return Err(RenderError::InvalidArity {
                        function: function.clone(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                plural_key(self.locale, &args[0])
            }
            [function] if expression.kind == IrExpressionKind::Call => {
                self.eval_function(function, &args, depth + 1)
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

    fn eval_inline_function(
        &self,
        inline_inputs: &[IrInlineFunctionInput],
        branches: &[IrFunctionBranch],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        let mut selector_values = Vec::new();
        let mut binding_values = Vec::new();

        // Evaluate every input against the same enclosing lexical scope. This
        // gives inline inputs ordinary call semantics: sibling bindings are
        // not visible from another binding RHS.
        for input in inline_inputs {
            let value_expression = match input {
                IrInlineFunctionInput::Binding { value, .. }
                | IrInlineFunctionInput::Selector { value, .. } => value,
            };
            let mut value =
                self.eval_expression_value(value_expression, context, inputs, depth + 1)?;
            match input {
                IrInlineFunctionInput::Selector { .. } => {
                    if inferred_inline_value_type(value_expression, context) == "Plural" {
                        value = plural_key(self.locale, &value)?;
                    }
                    selector_values.push(value);
                }
                IrInlineFunctionInput::Binding {
                    name, value: rhs, ..
                } => {
                    let ty = inferred_inline_value_type(rhs, context);
                    binding_values.push((name.clone(), ty, value));
                }
            }
        }

        let mut branch_context = context.clone();
        let mut branch_inputs = inputs.clone();
        for (name, ty, value) in binding_values {
            branch_context.insert(name.clone(), ty.clone());
            branch_inputs.insert(name, sample_value_for_type(&ty, value));
        }

        self.eval_inline_dispatch(
            branches,
            0,
            &selector_values,
            &branch_context,
            &branch_inputs,
            depth + 1,
        )
    }

    fn eval_inline_dispatch(
        &self,
        branches: &[IrFunctionBranch],
        dispatch_depth: usize,
        selectors: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        let selector = selectors
            .get(dispatch_depth)
            .map_or("undefined", String::as_str);
        let branch = matching_function_branch(branches, selector).ok_or_else(|| {
            RenderError::MissingBranch {
                owner: "inline fn".to_owned(),
                selector: selector.to_owned(),
            }
        })?;
        match &branch.value {
            IrFunctionBranchValue::Text(text) => self.render_text(text, context, inputs, depth + 1),
            IrFunctionBranchValue::Dispatch(children) => self.eval_inline_dispatch(
                children,
                dispatch_depth + 1,
                selectors,
                context,
                inputs,
                depth + 1,
            ),
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
            let callable = format!("{}.{}", form.name, property);
            let (value_context, value_inputs) = parameter_scope(&callable, parameters, args)?;
            return self.eval_ir_value(
                value,
                args.first().map(String::as_str),
                parameters
                    .first()
                    .map_or(true, |parameter| parameter.ty == "Plural"),
                &value_context,
                &value_inputs,
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
            &BTreeMap::new(),
            &BTreeMap::new(),
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
        let callable = format!("{}.{}", form.name, path.join("."));
        let (value_context, value_inputs) = parameter_scope(&callable, parameters, &[])?;
        self.eval_ir_value(
            value,
            None,
            parameters
                .first()
                .map_or(true, |parameter| parameter.ty == "Plural"),
            &value_context,
            &value_inputs,
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
        let (function_context, function_inputs) =
            parameter_scope(name, &function.parameters, args)?;
        self.eval_dispatch(
            name,
            &function.parameters,
            &function.branches,
            0,
            args,
            &function_context,
            &function_inputs,
            depth + 1,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn eval_dispatch(
        &self,
        callable: &str,
        parameters: &[IrFunctionParameter],
        branches: &[IrFunctionBranch],
        dispatch_depth: usize,
        args: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
        depth: usize,
    ) -> Result<String, RenderError> {
        ensure_depth(depth)?;
        let parameter_index = dispatch_parameter_indices(parameters).nth(dispatch_depth);
        let selector = parameter_index
            .map(|index| {
                args.get(index)
                    .map(String::as_str)
                    .ok_or_else(|| RenderError::InvalidArity {
                        function: callable.to_owned(),
                        expected: index + 1,
                        actual: args.len(),
                    })
            })
            .transpose()?
            .unwrap_or("undefined");
        let plural_parameter = parameter_index
            .and_then(|index| parameters.get(index))
            .is_some_and(|parameter| parameter.ty == "Plural");
        let key = if plural_parameter {
            plural_key(self.locale, selector)?
        } else {
            selector.to_owned()
        };
        let branch =
            matching_function_branch(branches, &key).ok_or_else(|| RenderError::MissingBranch {
                owner: dispatch_owner(callable),
                selector: key,
            })?;
        match &branch.value {
            IrFunctionBranchValue::Text(text) => self.render_text(text, context, inputs, depth + 1),
            IrFunctionBranchValue::Dispatch(branches) => self.eval_dispatch(
                callable,
                parameters,
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
            return Err(missing_cldr_data("number formatting", self.locale));
        };
        format_number_pattern(value, &numbers.decimal_pattern, &numbers, None, None)
    }

    fn format_currency(
        &self,
        value: &str,
        arguments: &[IrFormatterArgument],
    ) -> Result<String, RenderError> {
        let code = formatter_argument(arguments, "code").unwrap_or("USD");
        let accounting = formatter_argument(arguments, "accounting") == Some("true");
        let (Some(numbers), Some(currency), Some(fraction)) = (
            compiled_number_formatting(self.locale),
            compiled_currency_formatting(self.locale),
            compiled_currency_fraction(code),
        ) else {
            return Err(RenderError::Unsupported {
                detail: format!(
                    "missing required CLDR currency formatting or fraction data for preview locale `{}` and currency `{code}`",
                    self.locale
                ),
            });
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
        format_number_pattern(
            value,
            pattern,
            &numbers,
            Some(&symbol),
            Some((fraction.digits, fraction.rounding)),
        )
    }

    fn format_date(
        &self,
        value: &str,
        arguments: &[IrFormatterArgument],
    ) -> Result<String, RenderError> {
        let Some(dates) = compiled_date_formatting(self.locale) else {
            return Err(missing_cldr_data("date formatting", self.locale));
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

fn missing_cldr_data(kind: &str, locale: &str) -> RenderError {
    RenderError::Unsupported {
        detail: format!("missing required CLDR {kind} data for preview locale `{locale}`"),
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
        .or_else(|| branches.iter().find(|branch| branch.key == "other"))
}

fn dispatch_parameter_indices(
    parameters: &[IrFunctionParameter],
) -> impl Iterator<Item = usize> + '_ {
    parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| parameter.name.is_none().then_some(index))
}

fn inferred_inline_value_type(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
) -> String {
    match &expression.kind {
        IrExpressionKind::InlineFunction { .. } => "String".to_owned(),
        IrExpressionKind::Call
            if expression.path.len() == 1 && is_plural_intrinsic(&expression.path[0]) =>
        {
            "Plural".to_owned()
        }
        IrExpressionKind::Call => "String".to_owned(),
        IrExpressionKind::Reference if expression.path.len() == 1 => context
            .get(&expression.path[0])
            .cloned()
            .unwrap_or_else(|| "String".to_owned()),
        IrExpressionKind::Reference => "String".to_owned(),
    }
}

fn sample_value_for_type(ty: &str, value: String) -> SampleValue {
    match ty {
        "Number" | "Decimal" => SampleValue::Number(value),
        "Boolean" => value
            .parse::<bool>()
            .map_or_else(|_| SampleValue::String(value), SampleValue::Boolean),
        _ => SampleValue::String(value),
    }
}

fn parameter_scope(
    callable: &str,
    parameters: &[IrFunctionParameter],
    args: &[String],
) -> Result<ParameterScope, RenderError> {
    if args.len() != parameters.len() {
        return Err(RenderError::InvalidArity {
            function: callable.to_owned(),
            expected: parameters.len(),
            actual: args.len(),
        });
    }

    let mut context = BTreeMap::new();
    let mut inputs = BTreeMap::new();
    for (parameter, value) in parameters.iter().zip(args) {
        if let Some(name) = &parameter.name {
            context.insert(name.clone(), parameter.ty.clone());
            inputs.insert(name.clone(), SampleValue::String(value.clone()));
        }
    }
    Ok((context, inputs))
}

fn dispatch_owner(callable: &str) -> String {
    if callable == "inline fn" {
        callable.to_owned()
    } else {
        format!("function `{callable}`")
    }
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
        plural_key(locale, selector)?
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

fn plural_key(locale: &str, selector: &str) -> Result<String, RenderError> {
    if matches!(selector, "zero" | "one" | "two" | "few" | "many" | "other") {
        return Ok(selector.to_owned());
    }
    let rules =
        built_in_plural_rules(locale).ok_or_else(|| missing_cldr_data("plural rules", locale))?;
    let operands = ExactPluralOperands::parse(selector)?;
    Ok(exact_plural_category(&rules, &operands).to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExactPluralOperands {
    integer: String,
    fraction: String,
    trimmed_fraction: String,
    visible_fraction_digits: String,
    trimmed_visible_fraction_digits: String,
    compact_exponent: String,
}

impl ExactPluralOperands {
    fn parse(value: &str) -> Result<Self, RenderError> {
        let sample = value.trim();
        let unsigned = match sample.as_bytes().first() {
            Some(b'+' | b'-') => &sample[1..],
            _ => sample,
        };
        if unsigned.is_empty() || matches!(unsigned.as_bytes().first(), Some(b'+' | b'-')) {
            return Err(invalid_number(value));
        }
        let exponent_marker = unsigned
            .char_indices()
            .find(|(_, character)| matches!(character, 'c' | 'C' | 'e' | 'E'));
        let (mantissa, exponent_source) = exponent_marker.map_or((unsigned, "0"), |(index, _)| {
            (&unsigned[..index], &unsigned[index + 1..])
        });
        let exponent = parse_plural_exponent(exponent_source, value)?;
        let mut parts = mantissa.split('.');
        let integer = parts.next().unwrap_or_default();
        let fraction = parts.next().unwrap_or_default();
        if parts.next().is_some()
            || integer.is_empty()
            || !integer.bytes().all(|digit| digit.is_ascii_digit())
            || !fraction.bytes().all(|digit| digit.is_ascii_digit())
        {
            return Err(invalid_number(value));
        }
        let source_digits = integer
            .len()
            .checked_add(fraction.len())
            .ok_or_else(|| invalid_number(value))?;
        if source_digits > MAX_DECIMAL_DIGITS {
            return Err(invalid_number(value));
        }
        let decimal_position = integer
            .len()
            .checked_add(exponent)
            .ok_or_else(|| invalid_number(value))?;
        if decimal_position.max(source_digits) > MAX_DECIMAL_DIGITS {
            return Err(invalid_number(value));
        }

        let consumed_fraction_digits = exponent.min(fraction.len());
        let mut shifted_integer = format!("{integer}{}", &fraction[..consumed_fraction_digits]);
        shifted_integer
            .extend(std::iter::repeat('0').take(exponent.saturating_sub(consumed_fraction_digits)));
        let shifted_integer = normalize_decimal(&shifted_integer).to_owned();
        let shifted_fraction = fraction[consumed_fraction_digits..].to_owned();
        let trimmed_fraction = shifted_fraction.trim_end_matches('0').to_owned();
        Ok(Self {
            integer: shifted_integer,
            visible_fraction_digits: shifted_fraction.len().to_string(),
            trimmed_visible_fraction_digits: trimmed_fraction.len().to_string(),
            fraction: shifted_fraction,
            trimmed_fraction,
            compact_exponent: exponent.to_string(),
        })
    }

    fn operand(&self, operand: Operand) -> ExactPluralOperand<'_> {
        match operand {
            Operand::N => ExactPluralOperand {
                digits: &self.integer,
                has_fraction: self.fraction.bytes().any(|digit| digit != b'0'),
            },
            Operand::I => ExactPluralOperand::integer(&self.integer),
            Operand::V => ExactPluralOperand::integer(&self.visible_fraction_digits),
            Operand::W => ExactPluralOperand::integer(&self.trimmed_visible_fraction_digits),
            Operand::F => ExactPluralOperand::integer(&self.fraction),
            Operand::T => ExactPluralOperand::integer(&self.trimmed_fraction),
            Operand::C | Operand::E => ExactPluralOperand::integer(&self.compact_exponent),
        }
    }
}

fn parse_plural_exponent(source: &str, original: &str) -> Result<usize, RenderError> {
    if source.is_empty()
        || source.len() > MAX_DECIMAL_DIGITS
        || !source.bytes().all(|digit| digit.is_ascii_digit())
    {
        return Err(invalid_number(original));
    }
    let mut exponent = 0_usize;
    for digit in source.bytes() {
        exponent = exponent
            .checked_mul(10)
            .and_then(|value| value.checked_add(usize::from(digit - b'0')))
            .ok_or_else(|| invalid_number(original))?;
        if exponent > MAX_DECIMAL_DIGITS {
            return Err(invalid_number(original));
        }
    }
    Ok(exponent)
}

fn invalid_number(value: &str) -> RenderError {
    RenderError::InvalidNumber {
        value: value.to_owned(),
    }
}

fn normalize_decimal(value: &str) -> &str {
    let normalized = value.trim_start_matches('0');
    if normalized.is_empty() {
        "0"
    } else {
        normalized
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExactPluralOperand<'a> {
    digits: &'a str,
    has_fraction: bool,
}

impl<'a> ExactPluralOperand<'a> {
    fn integer(digits: &'a str) -> Self {
        Self {
            digits,
            has_fraction: false,
        }
    }
}

fn exact_plural_category<'a>(rules: &'a PluralRules, operands: &ExactPluralOperands) -> &'a str {
    rules
        .categories
        .iter()
        .filter(|category| category.category != "other")
        .find(|category| exact_plural_rule_matches(&category.rule, operands))
        .map(|category| category.category.as_str())
        .unwrap_or("other")
}

fn exact_plural_rule_matches(rule: &PluralRule, operands: &ExactPluralOperands) -> bool {
    !rule.conditions.is_empty()
        && rule.conditions.iter().any(|condition| {
            condition
                .relations
                .iter()
                .all(|relation| exact_plural_relation_matches(relation, operands))
        })
}

fn exact_plural_relation_matches(relation: &Relation, operands: &ExactPluralOperands) -> bool {
    let allow_fraction = matches!(
        relation.operator,
        RelationOperator::Within | RelationOperator::NotWithin
    );
    let matches = exact_plural_operand_matches(
        operands.operand(relation.expression.operand),
        relation.expression.modulo,
        allow_fraction,
        &relation.ranges.ranges,
    );
    match relation.operator {
        RelationOperator::Equal | RelationOperator::In | RelationOperator::Within => matches,
        RelationOperator::NotEqual | RelationOperator::NotIn | RelationOperator::NotWithin => {
            !matches
        }
    }
}

fn exact_plural_operand_matches(
    operand: ExactPluralOperand<'_>,
    modulo: Option<u64>,
    allow_fraction: bool,
    ranges: &[Range],
) -> bool {
    if operand.has_fraction && !allow_fraction {
        return false;
    }
    if let Some(modulo) = modulo.filter(|value| *value != 0) {
        let value = decimal_modulo(operand.digits, modulo);
        return ranges.iter().any(|range| {
            value >= range.start
                && (value < range.end || (value == range.end && !operand.has_fraction))
        });
    }
    ranges.iter().any(|range| {
        compare_decimal_to_u64(operand.digits, range.start).is_ge()
            && (compare_decimal_to_u64(operand.digits, range.end).is_lt()
                || (compare_decimal_to_u64(operand.digits, range.end).is_eq()
                    && !operand.has_fraction))
    })
}

fn decimal_modulo(value: &str, modulo: u64) -> u64 {
    value.bytes().fold(0_u64, |remainder, digit| {
        u64::try_from((u128::from(remainder) * 10 + u128::from(digit - b'0')) % u128::from(modulo))
            .expect("remainder is below u64 modulo")
    })
}

fn compare_decimal_to_u64(value: &str, target: u64) -> std::cmp::Ordering {
    compare_decimal(value, &target.to_string())
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
    let (negative, unsigned) = match value.as_bytes().first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    if matches!(unsigned.as_bytes().first(), Some(b'+' | b'-')) {
        return Err(RenderError::InvalidNumber {
            value: value.to_owned(),
        });
    }
    let (mantissa, exponent_source) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    if exponent_source.is_some_and(|source| source.len() > MAX_DECIMAL_DIGITS) {
        return Err(invalid_number(value));
    }
    let exponent = exponent_source.map_or(0i32, |source| source.parse::<i32>().unwrap_or(i32::MAX));
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
    let expanded_length = if decimal_position <= 0 {
        usize::try_from(-decimal_position)
            .ok()
            .and_then(|zero_count| zero_count.checked_add(digits.len()))
    } else {
        usize::try_from(decimal_position)
            .ok()
            .map(|position| position.max(digits.len()))
    }
    .ok_or_else(|| RenderError::InvalidNumber {
        value: value.to_owned(),
    })?;
    if expanded_length > MAX_DECIMAL_DIGITS {
        return Err(RenderError::InvalidNumber {
            value: value.to_owned(),
        });
    }

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
    currency_fraction: Option<(u8, u16)>,
) -> Result<String, RenderError> {
    let mut decimal = parse_decimal(value)?;
    let (min_fraction_digits, max_fraction_digits, rounding_increment) = currency_fraction.map_or(
        (
            usize::from(pattern.positive.min_fraction_digits),
            usize::from(pattern.positive.max_fraction_digits),
            0,
        ),
        |(digits, rounding)| (usize::from(digits), usize::from(digits), rounding),
    );
    round_fraction(
        &mut decimal.integer,
        &mut decimal.fraction,
        max_fraction_digits,
        rounding_increment,
    );
    while decimal.fraction.len() > min_fraction_digits && decimal.fraction.ends_with('0') {
        decimal.fraction.pop();
    }
    while decimal.fraction.len() < min_fraction_digits {
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

fn round_fraction(
    integer: &mut String,
    fraction: &mut String,
    max_digits: usize,
    rounding_increment: u16,
) {
    let source_fraction = std::mem::take(fraction);
    let mut kept_fraction = source_fraction
        .get(..max_digits.min(source_fraction.len()))
        .unwrap_or_default()
        .to_owned();
    kept_fraction
        .extend(std::iter::repeat('0').take(max_digits.saturating_sub(kept_fraction.len())));
    let discarded = source_fraction
        .get(max_digits.min(source_fraction.len())..)
        .unwrap_or_default();
    let scaled = format!("{integer}{kept_fraction}");
    let quantum = u32::from(rounding_increment.max(1));
    let (mut quotient, remainder) = divide_decimal_by_u32(&scaled, quantum);
    if should_round_up(remainder, discarded, quantum) {
        increment_integer(&mut quotient);
    }
    let mut rounded = multiply_decimal_by_u32(&quotient, quantum);
    let required_length = max_digits.saturating_add(1);
    if rounded.len() < required_length {
        rounded.insert_str(0, &"0".repeat(required_length - rounded.len()));
    }
    if max_digits == 0 {
        *integer = rounded;
        return;
    }
    let split = rounded.len() - max_digits;
    *integer = rounded[..split].to_owned();
    *fraction = rounded[split..].to_owned();
}

fn divide_decimal_by_u32(value: &str, divisor: u32) -> (String, u32) {
    debug_assert_ne!(divisor, 0);
    let mut quotient = String::with_capacity(value.len());
    let mut remainder = 0_u32;
    for byte in value.bytes() {
        let current = u64::from(remainder) * 10 + u64::from(byte - b'0');
        let digit = u32::try_from(current / u64::from(divisor)).expect("quotient digit fits");
        remainder = u32::try_from(current % u64::from(divisor)).expect("remainder fits");
        if !quotient.is_empty() || digit != 0 {
            quotient.push(char::from(
                b'0' + u8::try_from(digit).expect("single decimal digit"),
            ));
        }
    }
    if quotient.is_empty() {
        quotient.push('0');
    }
    (quotient, remainder)
}

fn should_round_up(remainder: u32, discarded: &str, quantum: u32) -> bool {
    if discarded.is_empty() {
        return remainder * 2 >= quantum;
    }
    let exact_remainder = format!("{remainder}{discarded}");
    let doubled = multiply_decimal_by_u32(&exact_remainder, 2);
    let threshold = format!("{quantum}{}", "0".repeat(discarded.len()));
    compare_decimal(&doubled, &threshold).is_ge()
}

fn multiply_decimal_by_u32(value: &str, multiplier: u32) -> String {
    let mut reversed = Vec::with_capacity(value.len() + 5);
    let mut carry = 0_u64;
    for byte in value.bytes().rev() {
        let product = u64::from(byte - b'0') * u64::from(multiplier) + carry;
        reversed.push(b'0' + u8::try_from(product % 10).expect("single decimal digit"));
        carry = product / 10;
    }
    while carry != 0 {
        reversed.push(b'0' + u8::try_from(carry % 10).expect("single decimal digit"));
        carry /= 10;
    }
    while reversed.len() > 1 && reversed.last() == Some(&b'0') {
        reversed.pop();
    }
    reversed.reverse();
    String::from_utf8(reversed).expect("ASCII decimal digits")
}

fn compare_decimal(left: &str, right: &str) -> std::cmp::Ordering {
    let left = left.trim_start_matches('0');
    let right = right.trim_start_matches('0');
    left.len()
        .cmp(&right.len())
        .then_with(|| left.as_bytes().cmp(right.as_bytes()))
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
    let normalized = code.to_ascii_uppercase();
    match normalized.as_str() {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" | "CNY" => "¥",
        "RUB" => "₽",
        "INR" => "₹",
        "KRW" => "₩",
        _ => &normalized,
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
            None,
        )
        .expect("format number");

        assert_eq!(rendered, "1,234,567,890.123");
    }

    #[test]
    fn currency_formatter_applies_cldr_fraction_and_rounding_rules() {
        let schema = lower_schema(&parse_schema("").expect("schema"));
        let locale = lower_locale(&parse_locale("").expect("locale"));
        let renderer = renderer(&schema, &locale, "en");
        let format = |value: &str, code: &str| {
            let arguments = vec![IrFormatterArgument {
                name: "code".to_owned(),
                value: code.to_owned(),
            }];
            renderer
                .format_currency(value, &arguments)
                .expect("format currency")
        };
        let numeric_part = |value: String| {
            value
                .chars()
                .filter(|character| character.is_ascii_digit() || matches!(character, '.' | '-'))
                .collect::<String>()
        };

        assert_eq!(numeric_part(format("1.25", "JPY")), "1");
        assert!(format("1.25", "jpy").contains('¥'));
        assert_eq!(numeric_part(format("1.2345", "KWD")), "1.235");
        assert_eq!(numeric_part(format("1.23456", "CLF")), "1.2346");
        assert_eq!(numeric_part(format("1.23", "CHF")), "1.23");

        let numbers = compiled_number_formatting("en").expect("number data");
        let currency = compiled_currency_formatting("en").expect("currency data");
        let cash_rounded = format_number_pattern(
            "1.23",
            &currency.standard_pattern,
            &numbers,
            Some("CHF"),
            Some((2, 5)),
        )
        .expect("format cash increment");
        assert_eq!(numeric_part(cash_rounded), "1.25");
    }

    #[test]
    fn number_parser_matches_generated_sign_and_size_limits() {
        for value in ["-+1", "+-1", "--1", "++1", "1e8192"] {
            assert!(parse_decimal(value).is_err(), "{value:?} must be rejected");
        }
        assert!(parse_decimal(&"1".repeat(MAX_DECIMAL_DIGITS)).is_ok());
        assert!(parse_decimal(&"1".repeat(MAX_DECIMAL_DIGITS + 1)).is_err());
        let padded_exponent = format!("1e{}1", "0".repeat(MAX_DECIMAL_DIGITS));
        assert!(parse_decimal(&padded_exponent).is_err());
    }

    #[test]
    fn missing_formatter_data_is_reported() {
        let schema = lower_schema(&parse_schema("").expect("schema"));
        let locale = lower_locale(&parse_locale("").expect("locale"));

        let error = renderer(&schema, &locale, "zz")
            .format_number("1")
            .expect_err("unsupported preview locale must fail");

        assert!(matches!(error, RenderError::Unsupported { .. }));
    }

    #[test]
    fn inline_function_dispatch_renders_with_captured_inputs() {
        let schema = lower_schema(
            &parse_schema(
                "enum Gender { masculine, feminine, other }\ngreeting(name: String, gender: Gender)\n",
            )
            .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "greeting = Hello {fn(gender) {\n  masculine => dear {name}\n  feminine => kind {name}\n  _ => friend {name}\n}}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([
            ("name".to_owned(), SampleValue::String("Artemy".to_owned())),
            (
                "gender".to_owned(),
                SampleValue::String("masculine".to_owned()),
            ),
        ]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("greeting", &inputs)
            .expect("inline fn renders");

        assert_eq!(rendered, "Hello dear Artemy");
    }

    #[test]
    fn inline_plural_dispatch_uses_the_other_fallback_in_preview() {
        let schema = lower_schema(&parse_schema("summary(count: Number)\n").expect("schema"));
        let locale = lower_locale(
            &parse_locale(
                "summary = {fn(Plural(count)) {\n  one => one item\n  other => many items\n}}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([("count".to_owned(), SampleValue::Number("2".to_owned()))]);

        let rendered = renderer(&schema, &locale, "ru")
            .render_message("summary", &inputs)
            .expect("plural other branch renders");

        assert_eq!(rendered, "many items");
    }

    #[test]
    fn inline_plural_dispatch_supports_huge_exact_selectors_in_preview() {
        let schema = lower_schema(&parse_schema("summary(count: Number)\n").expect("schema"));
        let locale = lower_locale(
            &parse_locale(
                "summary = {fn(Plural(count)) {\n  one => one\n  many => many\n  other => other\n}}\n",
            )
            .expect("locale"),
        );
        let one = format!("1{}1", "0".repeat(MAX_DECIMAL_DIGITS - 2));
        let many = format!("1{}11", "0".repeat(MAX_DECIMAL_DIGITS - 3));

        for (value, expected) in [(one, "one"), (many, "many")] {
            let inputs = BTreeMap::from([("count".to_owned(), SampleValue::Number(value))]);
            let rendered = renderer(&schema, &locale, "ru")
                .render_message("summary", &inputs)
                .expect("huge plural selector renders");
            assert_eq!(rendered, expected);
        }
    }

    #[test]
    fn inline_plural_dispatch_rejects_invalid_selectors_before_fallback() {
        let schema = lower_schema(&parse_schema("summary(count: Number)\n").expect("schema"));
        let locale = lower_locale(
            &parse_locale("summary = {fn(Plural(count)) {\n  one => one\n  _ => fallback\n}}\n")
                .expect("locale"),
        );
        let inputs = BTreeMap::from([(
            "count".to_owned(),
            SampleValue::Number("not-a-number".to_owned()),
        )]);

        let error = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect_err("invalid selector must not reach fallback");

        assert!(matches!(error, RenderError::InvalidNumber { .. }));
    }

    #[test]
    fn inferred_plural_inline_selector_normalizes_numeric_inputs() {
        let schema = lower_schema(&parse_schema("summary(count: Number)\n").expect("schema"));
        let locale = lower_locale(
            &parse_locale(
                "fn Render(category: Plural) {\n  _ => {fn(category) {\n    one => one\n    _ => other\n  }}\n}\nsummary = {Render(count)}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([("count".to_owned(), SampleValue::Number("1".to_owned()))]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect("inferred Plural selector renders");

        assert_eq!(rendered, "one");
    }

    #[test]
    fn nested_plural_function_dispatch_is_idempotent_for_category_values() {
        let schema = lower_schema(
            &parse_schema("enum Tone { formal, casual }\nsummary(tone: Tone, count: Number)\n")
                .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "fn Greeting(Tone, Plural) {\n  formal {\n    one => formal one\n    _ => formal many\n  }\n  _ {\n    _ => casual\n  }\n}\nsummary = {Greeting(tone, Plural(count))}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([
            ("tone".to_owned(), SampleValue::String("formal".to_owned())),
            ("count".to_owned(), SampleValue::Number("1".to_owned())),
        ]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect("pre-classified plural category dispatches");

        assert_eq!(rendered, "formal one");
    }

    #[test]
    fn exact_plural_operands_follow_compact_decimal_semantics() {
        let operands = ExactPluralOperands::parse("1.20050c3").expect("compact decimal");
        assert_eq!(operands.integer, "1200");
        assert_eq!(operands.fraction, "50");
        assert_eq!(operands.trimmed_fraction, "5");
        assert_eq!(operands.visible_fraction_digits, "2");
        assert_eq!(operands.trimmed_visible_fraction_digits, "1");
        assert_eq!(operands.compact_exponent, "3");

        let shifted = ExactPluralOperands::parse("  +1.2C6  ").expect("trimmed compact decimal");
        assert_eq!(shifted.integer, "1200000");
        assert_eq!(shifted.compact_exponent, "6");

        for invalid in [".5", "1c", "1c-3", "1c+3", "1e-2", "1c2e3", "--1"] {
            assert!(
                ExactPluralOperands::parse(invalid).is_err(),
                "{invalid:?} must be rejected"
            );
        }
    }

    #[test]
    fn exact_plural_evaluator_matches_canonical_rules_within_u64_range() {
        let samples = [
            "0",
            "1",
            "2",
            "3",
            "5",
            "11",
            "21",
            "101",
            "1.0",
            "1.5",
            "1.20050c3",
            "  +1.2C6  ",
        ];
        for locale in ["en", "ru", "ar", "pl", "cy", "fr", "sl"] {
            let rules = built_in_plural_rules(locale).expect("plural rules");
            for sample in samples {
                let expected = rules.category_for(sample).expect("canonical operands");
                let actual = plural_key(locale, sample).expect("exact operands");
                assert_eq!(actual, expected, "locale={locale}, sample={sample:?}");
            }
        }
    }

    #[test]
    fn named_and_inline_function_dispatch_have_preview_parity() {
        let schema = lower_schema(
            &parse_schema(
                "enum Tone { formal, casual }\nsummary(name: String, tone: Tone, count: Number)\n",
            )
            .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "fn Named(Tone, Plural, name: String) {\n  formal {\n    one => Dear {name}\n    other => Dear all, {name}\n  }\n  _ {\n    _ => Hi {name}\n  }\n}\nsummary = {Named(tone, count, name)} | {fn(tone, Plural(count)) {\n  formal {\n    one => Dear {name}\n    other => Dear all, {name}\n  }\n  _ {\n    _ => Hi {name}\n  }\n}}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([
            ("name".to_owned(), SampleValue::String("Artemy".to_owned())),
            ("tone".to_owned(), SampleValue::String("formal".to_owned())),
            ("count".to_owned(), SampleValue::Number("2".to_owned())),
        ]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect("named and inline functions render");

        assert_eq!(rendered, "Dear all, Artemy | Dear all, Artemy");
    }

    #[test]
    fn inline_function_branch_scope_is_a_lexical_closure() {
        let schema = lower_schema(
            &parse_schema("enum Tone { formal, casual }\nsummary(tone: Tone, hidden: String)\n")
                .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "let brand = Linguini\nsummary = {fn(tone) {\n  formal => {brand}\n  _ => {hidden}\n}}\n",
            )
            .expect("locale"),
        );
        let base_inputs = BTreeMap::from([(
            "hidden".to_owned(),
            SampleValue::String("must not leak".to_owned()),
        )]);
        let mut formal_inputs = base_inputs.clone();
        formal_inputs.insert("tone".to_owned(), SampleValue::String("formal".to_owned()));

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &formal_inputs)
            .expect("locale global remains visible");
        assert_eq!(rendered, "Linguini");

        let mut casual_inputs = base_inputs;
        casual_inputs.insert("tone".to_owned(), SampleValue::String("casual".to_owned()));
        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &casual_inputs)
            .expect("enclosing message input remains visible");

        assert_eq!(rendered, "must not leak");
    }

    #[test]
    fn inline_nested_dispatch_can_call_functions_and_forms() {
        let schema = lower_schema(
            &parse_schema(
                "enum Tone { formal, casual }\nenum Fruit { apple }\nsummary(name: String, tone: Tone, fruit: Fruit, count: Number)\n",
            )
            .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "fn Salute(name: String) {\n  _ => Hello {name}\n}\nimpl Fruit {\n  apple {\n    form nom(Plural) {\n      one => apple\n      _ => apples\n    }\n  }\n}\nsummary = {fn(tone, fruit, Plural(count)) {\n  formal {\n    apple {\n      one => {Salute(name)}: {fruit.nom(count)}\n      other => {Salute(name)}: {fruit.nom(count)}\n    }\n  }\n  _ {\n    _ {\n      _ => fallback\n    }\n  }\n}}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([
            ("name".to_owned(), SampleValue::String("Artemy".to_owned())),
            ("tone".to_owned(), SampleValue::String("formal".to_owned())),
            ("fruit".to_owned(), SampleValue::String("apple".to_owned())),
            ("count".to_owned(), SampleValue::Number("2".to_owned())),
        ]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect("nested inline branch callables render");

        assert_eq!(rendered, "Hello Artemy: apples");
    }

    #[test]
    fn inline_function_captures_the_immediate_named_function_scope() {
        let schema = lower_schema(
            &parse_schema("enum Tone { formal, casual }\nsummary(source: String, tone: Tone)\n")
                .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "fn Outer(Tone, label: String) {\n  formal => {fn(copy: label) {\n    _ => {copy}\n  }}\n  _ => fallback\n}\nsummary = {Outer(tone, source)}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([
            (
                "source".to_owned(),
                SampleValue::String("Artemy".to_owned()),
            ),
            ("tone".to_owned(), SampleValue::String("formal".to_owned())),
        ]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect("named function parameter is captured");

        assert_eq!(rendered, "Artemy");
    }

    #[test]
    fn inline_function_captures_the_immediate_form_attribute_scope() {
        let schema = lower_schema(
            &parse_schema(
                "enum Fruit { apple }\nenum Gender { masculine, feminine }\nsummary(fruit: Fruit, source: Gender)\n",
            )
            .expect("schema"),
        );
        let locale = lower_locale(
            &parse_locale(
                "impl Fruit {\n  apple {\n    form label(gender: Gender) {\n      masculine => {fn(gender) {\n        masculine => his apple\n        feminine => her apple\n      }}\n      feminine => fallback\n    }\n  }\n}\nsummary = {fruit.label(source)}\n",
            )
            .expect("locale"),
        );
        let inputs = BTreeMap::from([
            ("fruit".to_owned(), SampleValue::String("apple".to_owned())),
            (
                "source".to_owned(),
                SampleValue::String("masculine".to_owned()),
            ),
        ]);

        let rendered = renderer(&schema, &locale, "en")
            .render_message("summary", &inputs)
            .expect("form parameter is captured");

        assert_eq!(rendered, "his apple");
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
