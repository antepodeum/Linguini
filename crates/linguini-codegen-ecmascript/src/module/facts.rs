//! Expression facts and physical form exports selected before syntax emission.

use std::collections::BTreeSet;

use linguini_codegen::{independent_form_attribute_names, StringResultFacts};
use linguini_ir::{IrExpression, IrExpressionKind, IrModule};

use super::names::{
    form_attribute_binding_name, form_binding_name, private_value_name, property_access,
};

const FORM_VALUE_BINDING: &str = "__lgl_value";

#[derive(Default)]
pub(super) struct ExpressionFacts {
    strings: StringResultFacts,
    bundler: bool,
    independent_forms: BTreeSet<String>,
    local_form: Option<String>,
    functions: BTreeSet<String>,
    variables: BTreeSet<String>,
    reserved: BTreeSet<String>,
}

impl ExpressionFacts {
    pub(super) fn for_module(schema: &IrModule, module: &IrModule) -> Self {
        let functions = module
            .functions()
            .iter()
            .map(|item| item.name.clone())
            .collect::<BTreeSet<_>>();
        let reserved = functions
            .iter()
            .map(|name| private_value_name(name))
            .chain(super::formatters::formatter_helper_names(schema, module))
            .chain(["String", "undefined", "normalizeMessageArgs"].map(str::to_owned))
            .collect();
        Self {
            strings: StringResultFacts::for_module(module),
            bundler: false,
            independent_forms: BTreeSet::new(),
            local_form: None,
            functions,
            variables: module
                .variables()
                .iter()
                .map(|item| item.name.clone())
                .collect(),
            reserved,
        }
    }

    pub(super) fn for_bundler_module(
        schema: &IrModule,
        module: &IrModule,
        local_form: Option<&str>,
    ) -> Self {
        Self {
            bundler: true,
            independent_forms: module
                .forms()
                .iter()
                .filter(|form| independent_form_attribute_names(form).is_some())
                .map(|form| form.name.clone())
                .collect(),
            local_form: local_form.map(str::to_owned),
            ..Self::for_module(schema, module)
        }
    }

    pub(super) fn reserved_bindings(&self) -> &BTreeSet<String> {
        &self.reserved
    }

    pub(super) fn is_global_call(&self, expression: &IrExpression) -> bool {
        expression.kind == IrExpressionKind::Call
            && (linguini_ir::is_plural_intrinsic(&expression.path.join("."))
                || self.functions.contains(&expression.path.join(".")))
    }

    pub(super) fn global_variable_binding(&self, path: &[String]) -> Option<String> {
        let name = path.join(".");
        if self.variables.contains(&name) {
            return Some(private_value_name(&name));
        }
        let (root, properties) = path.split_first()?;
        if !self.variables.contains(root) {
            return None;
        }
        let mut binding = private_value_name(root);
        for property in properties {
            binding.push_str(&property_access(property));
        }
        Some(binding)
    }

    pub(super) fn expression_has_string_result<'a>(
        &self,
        expression: &IrExpression,
        locals: impl IntoIterator<Item = &'a str>,
    ) -> bool {
        self.strings
            .expression_has_string_result(expression, locals)
    }

    pub(super) fn independent_form(&self, name: &str) -> bool {
        self.bundler && self.independent_forms.contains(name)
    }

    pub(super) fn form_export_binding(&self, name: &str) -> String {
        if self.bundler {
            FORM_VALUE_BINDING.to_owned()
        } else {
            form_binding_name(name)
        }
    }

    pub(super) fn form_lookup(&self, name: &str, path: &[String]) -> String {
        let binding = form_binding_name(name);
        if !self.bundler {
            return binding;
        }
        let export = if self.independent_form(name) {
            form_attribute_binding_name(
                path.first()
                    .expect("independent forms require an attribute"),
            )
        } else {
            FORM_VALUE_BINDING.to_owned()
        };
        if self.local_form.as_deref() == Some(name) {
            export
        } else {
            format!("{binding}{}", property_access(&export))
        }
    }
}
