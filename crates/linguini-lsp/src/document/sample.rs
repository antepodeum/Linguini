use std::collections::BTreeMap;

use linguini_format::SourceKind;
use linguini_ir::{
    lower_locale, lower_schema, IrBranch, IrExpression, IrForm, IrFormEntry, IrFunction,
    IrFunctionBranch, IrFunctionBranchValue, IrModule, IrText, IrTextPart, IrValue,
};
use linguini_syntax::{parse_locale_with_recovery, parse_schema_with_recovery};

use super::LinguiniDocument;

#[derive(Debug, Clone)]
enum SampleValue {
    String(String),
    Number(i64),
}

impl SampleValue {
    fn as_text(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
        }
    }
}

pub(super) fn render_message_sample(
    document: &LinguiniDocument,
    workspace: impl IntoIterator<Item = LinguiniDocument>,
    name: &str,
) -> Option<String> {
    let locale_file = parse_locale_with_recovery(&document.text).ast?;
    let locale_module = lower_locale(&locale_file);
    let (schema_module, inputs) = sample_inputs_for_message(workspace, name)?;
    let locale = locale_from_uri(&document.uri).unwrap_or_else(|| "en".to_owned());
    Some(Renderer::new(&schema_module, &locale_module, &locale).render_message(name, &inputs))
        .filter(|value| !value.is_empty())
}

fn sample_inputs_for_message(
    workspace: impl IntoIterator<Item = LinguiniDocument>,
    name: &str,
) -> Option<(IrModule, BTreeMap<String, SampleValue>)> {
    for document in workspace {
        if document.kind != SourceKind::Schema {
            continue;
        }
        let Some(schema) = parse_schema_with_recovery(&document.text).ast else {
            continue;
        };
        let module = lower_schema(&schema);
        let Some(message) = module.messages.iter().find(|message| message.name == name) else {
            continue;
        };
        let inputs = message
            .parameters
            .iter()
            .map(|parameter| {
                (
                    parameter.name.clone(),
                    sample_value(&module, &parameter.ty)
                        .unwrap_or_else(|| SampleValue::String("sample".to_owned())),
                )
            })
            .collect();
        return Some((module, inputs));
    }
    None
}

fn sample_value(schema: &IrModule, ty: &str) -> Option<SampleValue> {
    let resolved = resolve_type(schema, ty);
    if let Some(enumeration) = schema.enums.iter().find(|item| item.name == ty) {
        return enumeration
            .variants
            .first()
            .map(|variant| SampleValue::String(variant.clone()));
    }
    match resolved.as_str() {
        "Number" | "Decimal" | "Integer" => Some(SampleValue::Number(3)),
        "Date" | "DateTime" => Some(SampleValue::String("2026-05-13".to_owned())),
        _ => None,
    }
}

fn resolve_type(schema: &IrModule, ty: &str) -> String {
    schema
        .type_aliases
        .iter()
        .find(|alias| alias.name == ty)
        .map(|alias| resolve_type(schema, &alias.target))
        .unwrap_or_else(|| ty.to_owned())
}

struct Renderer<'a> {
    schema: &'a IrModule,
    module: &'a IrModule,
    locale: &'a str,
}

impl<'a> Renderer<'a> {
    fn new(schema: &'a IrModule, module: &'a IrModule, locale: &'a str) -> Self {
        Self {
            schema,
            module,
            locale,
        }
    }

    fn render_message(&self, name: &str, inputs: &BTreeMap<String, SampleValue>) -> String {
        let Some(message) = self
            .module
            .messages
            .iter()
            .find(|message| message.name == name)
        else {
            return String::new();
        };
        let Some(body) = &message.body else {
            return String::new();
        };
        self.render_text(body, &self.context(name), inputs)
    }

    fn render_text(
        &self,
        text: &IrText,
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> String {
        text.parts
            .iter()
            .map(|part| match part {
                IrTextPart::Text(value) => value.clone(),
                IrTextPart::Placeholder(expression) => {
                    self.eval_expression(expression, context, inputs)
                }
            })
            .collect()
    }

    fn eval_expression(
        &self,
        expression: &IrExpression,
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> String {
        let args = expression
            .arguments
            .iter()
            .map(|argument| self.eval_expression(argument, context, inputs))
            .collect::<Vec<_>>();
        match expression.path.as_slice() {
            [root] if !args.is_empty() && context.contains_key(root) => {
                self.eval_form_call(root, None, &args, context, inputs)
            }
            [root, property] if !args.is_empty() && context.contains_key(root) => {
                self.eval_form_call(root, Some(property), &args, context, inputs)
            }
            [function] if function == "plural" && args.len() == 1 => {
                plural_key(self.locale, &args[0])
            }
            [function] if !args.is_empty() => self.eval_function(function, &args, context, inputs),
            [root] => inputs
                .get(root)
                .map(SampleValue::as_text)
                .unwrap_or_default(),
            [root, property] if context.contains_key(root) => {
                self.eval_form_property(root, &[property.as_str()], context, inputs)
            }
            [root, tail @ ..] if context.contains_key(root) => {
                let path = tail.iter().map(String::as_str).collect::<Vec<_>>();
                self.eval_form_property(root, &path, context, inputs)
            }
            _ => String::new(),
        }
    }

    fn eval_form_call(
        &self,
        root: &str,
        property: Option<&String>,
        args: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> String {
        let Some(form) = self.form_for(root, context) else {
            return String::new();
        };
        let Some(variant) = inputs.get(root).map(SampleValue::as_text) else {
            return String::new();
        };
        let Some(variant) = form.variants.iter().find(|item| item.name == variant) else {
            return String::new();
        };
        if let Some(property) = property {
            return find_entry_value(&variant.entries, &[property.as_str()])
                .map(|value| {
                    self.eval_value(value, args.first().map(String::as_str), context, inputs)
                })
                .unwrap_or_default();
        }
        select_branch(
            args.first().map(String::as_str).unwrap_or("other"),
            variant
                .entries
                .iter()
                .filter_map(|entry| match entry {
                    IrFormEntry::Branch(branch) => Some(branch),
                    IrFormEntry::Attribute { .. } => None,
                })
                .collect(),
            self.locale,
            context,
            inputs,
            self,
        )
    }

    fn eval_form_property(
        &self,
        root: &str,
        path: &[&str],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> String {
        let Some(form) = self.form_for(root, context) else {
            return String::new();
        };
        let Some(variant) = inputs.get(root).map(SampleValue::as_text) else {
            return String::new();
        };
        let Some(variant) = form.variants.iter().find(|item| item.name == variant) else {
            return String::new();
        };
        find_entry_value(&variant.entries, path)
            .map(|value| self.eval_value(value, None, context, inputs))
            .unwrap_or_default()
    }

    fn eval_value(
        &self,
        value: &IrValue,
        selector: Option<&str>,
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> String {
        match value {
            IrValue::Text(text) => self.render_text(text, context, inputs),
            IrValue::Map(branches) => select_branch(
                selector.unwrap_or("other"),
                branches.iter().collect(),
                self.locale,
                context,
                inputs,
                self,
            ),
            IrValue::Object(_) => String::new(),
        }
    }

    fn eval_function(
        &self,
        name: &str,
        args: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> String {
        let Some(function) = self
            .module
            .functions
            .iter()
            .find(|function| function.name == name)
        else {
            return String::new();
        };
        self.eval_dispatch(function, &function.branches, 0, args, context, inputs)
    }

    fn eval_dispatch(
        &self,
        function: &IrFunction,
        branches: &[IrFunctionBranch],
        depth: usize,
        args: &[String],
        context: &BTreeMap<String, String>,
        inputs: &BTreeMap<String, SampleValue>,
    ) -> String {
        let parameter_index = dispatch_parameter_indices(function)
            .get(depth)
            .copied()
            .unwrap_or(depth);
        let Some(selector) = args.get(parameter_index) else {
            return String::new();
        };
        let key = function
            .parameters
            .get(parameter_index)
            .filter(|parameter| parameter.ty == "Plural")
            .map(|_| plural_key(self.locale, selector))
            .unwrap_or_else(|| selector.clone());
        let Some(branch) = matching_function_branch(branches, &key) else {
            return String::new();
        };
        match &branch.value {
            IrFunctionBranchValue::Text(text) => self.render_text(text, context, inputs),
            IrFunctionBranchValue::Dispatch(branches) => {
                self.eval_dispatch(function, branches, depth + 1, args, context, inputs)
            }
        }
    }

    fn form_for(&self, root: &str, context: &BTreeMap<String, String>) -> Option<&'a IrForm> {
        let ty = context.get(root)?;
        self.module.forms.iter().find(|form| form.name == *ty)
    }

    fn context(&self, message_name: &str) -> BTreeMap<String, String> {
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
            .unwrap_or_default()
    }
}

fn find_entry_value<'a>(entries: &'a [IrFormEntry], path: &[&str]) -> Option<&'a IrValue> {
    let (head, tail) = path.split_first()?;
    for entry in entries {
        if let IrFormEntry::Attribute { name, value } = entry {
            if name == head {
                return if tail.is_empty() {
                    Some(value)
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

fn select_branch(
    selector: &str,
    branches: Vec<&IrBranch>,
    locale: &str,
    context: &BTreeMap<String, String>,
    inputs: &BTreeMap<String, SampleValue>,
    renderer: &Renderer<'_>,
) -> String {
    let key = plural_key(locale, selector);
    branches
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
        .map(|branch| renderer.render_text(&branch.value, context, inputs))
        .unwrap_or_default()
}

fn plural_key(locale: &str, selector: &str) -> String {
    if let Some(rules) = linguini_cldr::built_in_plural_rules(locale) {
        if let Ok(category) = rules.category_for(selector) {
            return category.to_owned();
        }
    }
    selector.to_owned()
}

fn locale_from_uri(uri: &str) -> Option<String> {
    uri.rsplit('/')
        .next()
        .and_then(|name| name.strip_suffix(".lgl"))
        .map(str::to_owned)
}
