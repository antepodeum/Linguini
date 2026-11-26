//! Expression facts and physical form exports selected before syntax emission.

use std::collections::BTreeSet;

use linguini_codegen::{independent_form_attribute_names, StringResultFacts};
use linguini_ir::{IrExpression, IrModule};

use super::names::{form_attribute_binding_name, form_binding_name, property_access};

const FORM_VALUE_BINDING: &str = "__lgl_value";

#[derive(Default)]
pub(super) struct ExpressionFacts {
    strings: StringResultFacts,
    bundler: bool,
    independent_forms: BTreeSet<String>,
    local_form: Option<String>,
}

impl ExpressionFacts {
    pub(super) fn for_module(module: &IrModule) -> Self {
        Self {
            strings: StringResultFacts::for_module(module),
            bundler: false,
            independent_forms: BTreeSet::new(),
            local_form: None,
        }
    }

    pub(super) fn for_bundler_module(module: &IrModule, local_form: Option<&str>) -> Self {
        Self {
            strings: StringResultFacts::for_module(module),
            bundler: true,
            independent_forms: module
                .forms()
                .iter()
                .filter(|form| independent_form_attribute_names(form).is_some())
                .map(|form| form.name.clone())
                .collect(),
            local_form: local_form.map(str::to_owned),
        }
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
