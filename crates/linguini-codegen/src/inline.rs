//! Lift expression-local dispatch into static functions with explicit captures.

use crate::parameter_formatters;
use linguini_ir::{
    is_plural_intrinsic, IrExpression, IrExpressionKind, IrFormEntry, IrFunction, IrFunctionBranch,
    IrFunctionBranchValue, IrFunctionKind, IrFunctionParameter, IrInlineFunctionInput, IrModule,
    IrModuleBuilder, IrOrigin, IrSymbolKind, IrText, IrTextPart, IrValue,
};
use std::collections::{BTreeMap, BTreeSet};

type Types = BTreeMap<String, String>;
type Defaults = BTreeMap<String, Vec<linguini_ir::IrFormatter>>;

/// Preserve eager input order and lexical captures without allocating runtime closures.
/// Each generated function retains the inline expression's source identity. The pass is
/// idempotent and runs after semantic validation, before dependency slicing and rendering.
pub fn lower_inline_functions(schema: &IrModule, module: &IrModule) -> IrModule {
    let mut lower = Lowering {
        functions: Vec::new(),
        origins: Vec::new(),
        owner: String::new(),
        names: module
            .functions()
            .iter()
            .map(|item| item.name.clone())
            .chain(module.variables().iter().map(|item| item.name.clone()))
            .chain(module.messages().iter().map(|item| item.name.clone()))
            .collect(),
    };
    let mut builder = IrModuleBuilder::seeded(module)
        .clear_variables()
        .clear_messages()
        .clear_forms()
        .clear_functions();
    for item in module.variables() {
        let mut item = item.clone();
        lower.owner.clone_from(&item.name);
        lower.text(&mut item.value, &Types::new(), &Defaults::new());
        builder = builder.push_variable(item);
    }
    for item in module.messages() {
        let mut item = item.clone();
        lower.owner.clone_from(&item.name);
        let signature = schema
            .messages()
            .iter()
            .find(|signature| signature.name == item.name)
            .unwrap_or(&item);
        let context = signature
            .parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
            .collect();
        let defaults = parameter_formatters(schema, signature);
        if let Some(body) = &mut item.body {
            lower.text(body, &context, &defaults);
        }
        builder = builder.push_message(item);
    }
    for item in module.forms() {
        let mut item = item.clone();
        lower.owner.clone_from(&item.name);
        for variant in &mut item.variants {
            for entry in &mut variant.entries {
                lower.entry(entry);
            }
        }
        builder = builder.push_form(item);
    }
    for item in module.functions() {
        let mut item = item.clone();
        lower.owner.clone_from(&item.name);
        let context = item
            .parameters
            .iter()
            .filter_map(|parameter| {
                parameter
                    .name
                    .as_ref()
                    .map(|name| (name.clone(), parameter.ty.clone()))
            })
            .collect();
        lower.branches(&mut item.branches, &context, &Defaults::new());
        builder = builder.push_function(item);
    }
    for function in lower.functions {
        builder = builder.push_function(function);
    }
    for origin in lower.origins {
        builder = builder.push_origin(origin);
    }
    builder
        .build()
        .expect("inline lowering reserves unique function names")
}

struct Lowering {
    functions: Vec<IrFunction>,
    origins: Vec<IrOrigin>,
    owner: String,
    names: BTreeSet<String>,
}
impl Lowering {
    fn text(&mut self, text: &mut IrText, context: &Types, defaults: &Defaults) {
        for part in &mut text.parts {
            if let IrTextPart::Placeholder(expression) = part {
                if expression.formatters.is_empty() {
                    if let Some(formatters) =
                        expression.path.first().and_then(|name| defaults.get(name))
                    {
                        expression.formatters.clone_from(formatters);
                    }
                }
                self.expression(expression, context, defaults);
            }
        }
    }
    fn expression(&mut self, expression: &mut IrExpression, context: &Types, defaults: &Defaults) {
        for argument in &mut expression.arguments {
            self.expression(argument, context, defaults);
        }
        let IrExpressionKind::InlineFunction { inputs, branches } = &mut expression.kind else {
            return;
        };
        let mut parameters = Vec::new();
        let mut arguments = Vec::new();
        let mut branch_context = context.clone();
        let mut branch_defaults = defaults.clone();
        let mut bindings = BTreeSet::new();
        for input in inputs {
            let (value, name) = match input {
                IrInlineFunctionInput::Selector { value, .. } => (value, None),
                IrInlineFunctionInput::Binding { name, value, .. } => (value, Some(name.clone())),
            };
            let mut ty = expression_type(value, context);
            if let Some(name) = &name {
                branch_context.insert(name.clone(), ty.clone());
                branch_defaults.remove(name);
                if value.formatters.is_empty()
                    && value.kind == IrExpressionKind::Reference
                    && value.path.len() == 1
                {
                    if let Some(formatters) = defaults.get(&value.path[0]) {
                        branch_defaults.insert(name.clone(), formatters.clone());
                    }
                }
                bindings.insert(name.clone());
            } else if plural_call(value) {
                ty = "String".to_owned();
            }
            self.expression(value, context, defaults);
            parameters.push(IrFunctionParameter { name, ty });
            arguments.push(value.clone());
        }
        let mut captures = BTreeSet::new();
        free_branch_names(branches, &bindings, &mut captures);
        for name in captures {
            if let Some(ty) = context.get(&name) {
                parameters.push(IrFunctionParameter {
                    name: Some(name.clone()),
                    ty: ty.clone(),
                });
                arguments.push(IrExpression {
                    kind: IrExpressionKind::Reference,
                    path: vec![name],
                    arguments: vec![],
                    formatters: vec![],
                    span: expression.span,
                });
            }
        }
        self.branches(branches, &branch_context, &branch_defaults);
        let owner = self.owner.rsplit('.').next().unwrap_or(&self.owner);
        let mut base = String::new();
        for character in owner.chars() {
            if character.is_uppercase() && !base.is_empty() {
                base.push('_');
            }
            base.extend(character.to_lowercase());
        }
        base.push_str("_inline");
        let mut name = base.clone();
        let mut suffix = 2;
        while context.contains_key(&name)
            || branch_context.contains_key(&name)
            || !self.names.insert(name.clone())
        {
            name = format!("{base}{suffix}");
            suffix += 1;
        }
        self.origins.push(IrOrigin {
            kind: IrSymbolKind::Function,
            name: name.clone(),
            span: expression.span,
            is_override: false,
        });
        self.functions.push(IrFunction {
            kind: IrFunctionKind::Function,
            name: name.clone(),
            docs: vec![],
            parameters,
            branches: branches.clone(),
        });
        expression.kind = IrExpressionKind::Call;
        expression.path = vec![name];
        expression.arguments = arguments;
    }
    fn branches(
        &mut self,
        branches: &mut [IrFunctionBranch],
        context: &Types,
        defaults: &Defaults,
    ) {
        for branch in branches {
            match &mut branch.value {
                IrFunctionBranchValue::Text(text) => self.text(text, context, defaults),
                IrFunctionBranchValue::Dispatch(children) => {
                    self.branches(children, context, defaults)
                }
            }
        }
    }
    fn entry(&mut self, entry: &mut IrFormEntry) {
        match entry {
            IrFormEntry::Branch(branch) => {
                self.text(&mut branch.value, &Types::new(), &Defaults::new())
            }
            IrFormEntry::Attribute {
                parameters, value, ..
            } => {
                let context = parameters
                    .iter()
                    .filter_map(|parameter| {
                        parameter
                            .name
                            .as_ref()
                            .map(|name| (name.clone(), parameter.ty.clone()))
                    })
                    .collect();
                match value {
                    IrValue::Text(text) => self.text(text, &context, &Defaults::new()),
                    IrValue::Map(branches) => {
                        for branch in branches {
                            self.text(&mut branch.value, &context, &Defaults::new());
                        }
                    }
                    IrValue::Object(entries) => {
                        for entry in entries {
                            self.entry(entry);
                        }
                    }
                }
            }
        }
    }
}
fn plural_call(expression: &IrExpression) -> bool {
    expression.kind == IrExpressionKind::Call
        && expression.path.len() == 1
        && is_plural_intrinsic(&expression.path[0])
}
fn expression_type(expression: &IrExpression, context: &Types) -> String {
    if plural_call(expression) {
        return "Plural".to_owned();
    }
    if expression.kind == IrExpressionKind::Reference && expression.path.len() == 1 {
        if let Some(ty) = context.get(&expression.path[0]) {
            return ty.clone();
        }
    }
    "String".to_owned()
}
fn free_branch_names(
    branches: &[IrFunctionBranch],
    bound: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    for branch in branches {
        match &branch.value {
            IrFunctionBranchValue::Text(text) => {
                for part in &text.parts {
                    if let IrTextPart::Placeholder(expression) = part {
                        free_expression_names(expression, bound, names);
                    }
                }
            }
            IrFunctionBranchValue::Dispatch(children) => free_branch_names(children, bound, names),
        }
    }
}
fn free_expression_names(
    expression: &IrExpression,
    bound: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    if let Some(root) = expression.path.first() {
        if !bound.contains(root) {
            names.insert(root.clone());
        }
    }
    for argument in &expression.arguments {
        free_expression_names(argument, bound, names);
    }
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        let mut inner = bound.clone();
        for input in inputs {
            let value = match input {
                IrInlineFunctionInput::Binding { name, value, .. } => {
                    inner.insert(name.clone());
                    value
                }
                IrInlineFunctionInput::Selector { value, .. } => value,
            };
            free_expression_names(value, bound, names);
        }
        free_branch_names(branches, &inner, names);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema, SourceId};

    #[test]
    fn lifting_preserves_captures_defaults_source_identity_and_input_order() {
        let schema = lower_schema(&parse_schema("type Price = Decimal @currency(code = \"EUR\")\nsummary(price: Price, label: String)\n").expect("schema"));
        let locale = lower_locale(&parse_locale_in("summary = {fn(Plural(price), copy: label) { one => {copy}: {price} _ => {fn(alias: price) { _ => {alias} }} }}\n", SourceId(19)).expect("locale"));
        let lowered = lower_inline_functions(&schema, &locale);
        assert!(!crate::module_uses_inline_functions(&lowered));
        assert_eq!(lower_inline_functions(&schema, &lowered), lowered);
        assert_eq!(lowered.functions().len(), 2);
        let body = lowered.messages()[0].body.as_ref().expect("body");
        let IrTextPart::Placeholder(call) = &body.parts[0] else {
            panic!("inline call");
        };
        assert_eq!(call.kind, IrExpressionKind::Call);
        assert_eq!(
            call.arguments
                .iter()
                .map(|argument| argument.path.join("."))
                .collect::<Vec<_>>(),
            vec!["Plural", "label", "price"]
        );
        let helper = lowered
            .functions()
            .iter()
            .find(|function| function.name == call.path[0])
            .expect("outer function");
        assert_eq!(
            helper
                .parameters
                .iter()
                .map(|parameter| (parameter.name.as_deref(), parameter.ty.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (None, "String"),
                (Some("copy"), "String"),
                (Some("price"), "Price")
            ]
        );
        assert_eq!(
            crate::formatter_requests(&schema, &lowered),
            crate::formatter_requests(&schema, &locale)
        );
        for function in lowered.functions() {
            let origin = lowered
                .origins()
                .iter()
                .find(|origin| {
                    origin.kind == IrSymbolKind::Function && origin.name == function.name
                })
                .expect("generated source identity");
            assert_eq!(origin.span.source, SourceId(19));
        }
        let closure = crate::message_dependency_closure(&schema, "en", &lowered, "summary")
            .expect("dependency closure");
        assert_eq!(closure.locale_module().functions().len(), 2);
        assert!(closure.source_ids().contains(&SourceId(19)));
    }

    #[test]
    fn lifted_names_do_not_shadow_source_bindings_or_existing_functions() {
        let schema =
            lower_schema(&parse_schema("summary(summary_inline: String)\n").expect("schema"));
        let locale = lower_locale(&parse_locale_in("fn summary_inline(value: String) { _ => {value} }\nfn summary_inline2(value: String) { _ => {value} }\nsummary = {fn(copy: summary_inline) { _ => {copy} }}\n", SourceId(20)).expect("locale"));
        let lowered = lower_inline_functions(&schema, &locale);
        assert_eq!(
            lowered.functions().last().expect("helper").name,
            "summary_inline3"
        );
    }
}
