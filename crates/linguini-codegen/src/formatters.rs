//! Exact statically used formatter requests, independent of runtime syntax.

use linguini_core::TypeKind;
use linguini_ir::{
    IrExpression, IrExpressionKind, IrFormEntry, IrFormatter, IrFormatterKind, IrFunctionBranch,
    IrFunctionBranchValue, IrInlineFunctionInput, IrMessage, IrModule, IrText, IrTextPart, IrValue,
};
use std::collections::{BTreeMap, BTreeSet};

/// A canonical formatter request. Argument order never changes its identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatterRequest {
    pub formatter: IrFormatter,
}

impl FormatterRequest {
    pub fn new(formatter: &IrFormatter) -> Self {
        let mut formatter = formatter.clone();
        formatter.arguments.sort_by(|a, b| a.name.cmp(&b.name));
        Self { formatter }
    }

    /// Stable, collision-free key for backend bindings and deduplication.
    pub fn key(&self) -> String {
        let mut key = match &self.formatter.kind {
            IrFormatterKind::Number => "number".to_owned(),
            IrFormatterKind::Currency => "currency".to_owned(),
            IrFormatterKind::Date => "date".to_owned(),
            IrFormatterKind::Unknown(name) => name.clone(),
        };
        for argument in &self.formatter.arguments {
            for part in [&argument.name, &argument.value] {
                key.push('_');
                for byte in part.bytes() {
                    use std::fmt::Write;
                    write!(key, "{byte:02x}").expect("writing to a string cannot fail");
                }
            }
        }
        key
    }

    pub fn argument(&self, name: &str) -> Option<&str> {
        self.formatter
            .arguments
            .iter()
            .find(|item| item.name == name)
            .map(|item| item.value.as_str())
    }
}

/// Resolve automatic formatting through schema aliases, stopping at the first explicit chain.
pub fn default_type_formatters(schema: &IrModule, ty: &str) -> Option<Vec<IrFormatter>> {
    let mut current = ty;
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(current) {
            return None;
        }
        if let Some(alias) = schema
            .type_aliases()
            .iter()
            .find(|alias| alias.name == current)
        {
            if !alias.formatters.is_empty() {
                return Some(alias.formatters.clone());
            }
            current = &alias.target;
        } else {
            return Some(vec![IrFormatter {
                kind: TypeKind::from_name(current)?.default_formatter()?,
                arguments: vec![],
            }]);
        }
    }
}

pub fn parameter_formatters(
    schema: &IrModule,
    signature: &IrMessage,
) -> BTreeMap<String, Vec<IrFormatter>> {
    signature
        .parameters
        .iter()
        .filter_map(|parameter| {
            default_type_formatters(schema, &parameter.ty)
                .map(|formatters| (parameter.name.clone(), formatters))
        })
        .collect()
}

/// Collect only requests that the implementation executes. Unused aliases and parameters do
/// not demand runtime code; an explicit formatter overrides automatic formatting.
pub fn formatter_requests(schema: &IrModule, locale: &IrModule) -> Vec<FormatterRequest> {
    let mut collector = Collector::default();
    let empty = BTreeMap::new();
    for variable in locale.variables() {
        collector.text(&variable.value, &empty);
    }
    for message in locale.messages() {
        if let Some(body) = &message.body {
            let defaults = schema
                .messages()
                .iter()
                .find(|item| item.name == message.name)
                .map(|signature| parameter_formatters(schema, signature))
                .unwrap_or_default();
            collector.text(body, &defaults);
        }
    }
    for form in locale.forms() {
        for variant in &form.variants {
            for entry in &variant.entries {
                collector.entry(entry);
            }
        }
    }
    for function in locale.functions() {
        collector.branches(&function.branches, &empty);
    }
    collector.requests.into_values().collect()
}

#[derive(Default)]
struct Collector {
    requests: BTreeMap<String, FormatterRequest>,
}
impl Collector {
    fn text(&mut self, text: &IrText, defaults: &BTreeMap<String, Vec<IrFormatter>>) {
        for part in &text.parts {
            if let IrTextPart::Placeholder(expression) = part {
                let formatters = if expression.formatters.is_empty() {
                    expression
                        .path
                        .first()
                        .and_then(|name| defaults.get(name))
                        .map_or(expression.formatters.as_slice(), Vec::as_slice)
                } else {
                    &expression.formatters
                };
                for formatter in formatters {
                    if !matches!(formatter.kind, IrFormatterKind::Unknown(_)) {
                        let request = FormatterRequest::new(formatter);
                        self.requests.insert(request.key(), request);
                    }
                }
                self.value(expression, defaults);
            }
        }
    }
    fn value(&mut self, expression: &IrExpression, defaults: &BTreeMap<String, Vec<IrFormatter>>) {
        for argument in &expression.arguments {
            self.value(argument, defaults);
        }
        if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
            let mut branch_defaults = defaults.clone();
            for input in inputs {
                let value = match input {
                    IrInlineFunctionInput::Binding { name, value, .. } => {
                        // Mirrors the lexical alias inheritance of an inline binding. Inputs
                        // are evaluated in the outer scope, before any branch is selected.
                        branch_defaults.remove(name);
                        if value.formatters.is_empty()
                            && value.kind == IrExpressionKind::Reference
                            && value.path.len() == 1
                        {
                            if let Some(formatters) = defaults.get(&value.path[0]) {
                                branch_defaults.insert(name.clone(), formatters.clone());
                            }
                        }
                        value
                    }
                    IrInlineFunctionInput::Selector { value, .. } => value,
                };
                self.value(value, defaults);
            }
            self.branches(branches, &branch_defaults);
        }
    }
    fn branches(
        &mut self,
        branches: &[IrFunctionBranch],
        defaults: &BTreeMap<String, Vec<IrFormatter>>,
    ) {
        for branch in branches {
            match &branch.value {
                IrFunctionBranchValue::Text(text) => self.text(text, defaults),
                IrFunctionBranchValue::Dispatch(branches) => self.branches(branches, defaults),
            }
        }
    }
    fn entry(&mut self, entry: &IrFormEntry) {
        let empty = BTreeMap::new();
        match entry {
            IrFormEntry::Branch(branch) => self.text(&branch.value, &empty),
            IrFormEntry::Attribute { value, .. } => match value {
                IrValue::Text(text) => self.text(text, &empty),
                IrValue::Map(branches) => {
                    for branch in branches {
                        self.text(&branch.value, &empty);
                    }
                }
                IrValue::Object(entries) => {
                    for entry in entries {
                        self.entry(entry);
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale, parse_schema};

    #[test]
    fn collects_executed_requests_with_alias_overrides_and_inline_shadowing() {
        let schema = lower_schema(&parse_schema("type Unused = Date @date(style = \"full\")\ntype Price = Decimal @currency(code = \"EUR\")\nsummary(price: Price, unused: Date)\n").expect("schema"));
        let locale = lower_locale(&parse_locale("summary = {price @number} {fn(copy: price) { _ => {copy} }} {fn(price: price) { _ => {price @number} }}\n").expect("locale"));
        let requests = formatter_requests(&schema, &locale);
        assert_eq!(requests.len(), 2);
        assert!(requests
            .iter()
            .any(|request| request.formatter.kind == IrFormatterKind::Number));
        assert!(requests.iter().any(
            |request| request.formatter.kind == IrFormatterKind::Currency
                && request.argument("code") == Some("EUR")
        ));
        assert!(!requests
            .iter()
            .any(|request| request.formatter.kind == IrFormatterKind::Date));
    }

    #[test]
    fn canonical_keys_preserve_distinct_argument_boundaries() {
        let request = |arguments: &[(&str, &str)]| {
            FormatterRequest::new(&IrFormatter {
                kind: IrFormatterKind::Date,
                arguments: arguments
                    .iter()
                    .map(|(name, value)| linguini_ir::IrFormatterArgument {
                        name: (*name).into(),
                        value: (*value).into(),
                    })
                    .collect(),
            })
        };
        assert_eq!(
            request(&[("style", "short"), ("time_style", "long")]).key(),
            request(&[("time_style", "long"), ("style", "short")]).key()
        );
        assert_ne!(request(&[("ab", "c")]).key(), request(&[("a", "bc")]).key());
    }
}
