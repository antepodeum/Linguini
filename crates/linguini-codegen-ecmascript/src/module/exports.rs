//! Public value names and their independently allocated local bindings.

use super::names::{property_key, public_value_name, safe_identifier};
use crate::{EcmaImport, EcmaNamedImport};
use minijinja::{context, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ValueExport {
    pub name: String,
    pub binding: String,
}

impl ValueExport {
    pub fn new(source: &str) -> Self {
        Self::alias(source, &public_value_name(source))
    }

    pub fn alias(source: &str, binding: &str) -> Self {
        Self {
            name: safe_identifier(source),
            binding: binding.to_owned(),
        }
    }

    pub fn is_direct(&self) -> bool {
        self.name == self.binding
    }

    pub fn import(&self, path: String) -> EcmaImport {
        EcmaImport::named(path, vec![EcmaNamedImport::new(&self.name, &self.binding)])
    }

    pub fn statement(&self, blank_line: bool) -> String {
        crate::templates::render(
            "module/value_export",
            context! {
                name => &self.name, binding => &self.binding, blank_line,
            },
        )
    }

    pub fn append_alias(&self, output: &mut String) {
        if !self.is_direct() {
            output.push_str(&self.statement(true));
        }
    }

    fn context(&self) -> Value {
        context! { name => &self.name, binding => &self.binding, key => property_key(&self.name) }
    }
}

pub(super) fn default_context<'a>(
    exports: impl IntoIterator<Item = &'a ValueExport>,
    namespaces: &[String],
) -> Value {
    let exports = exports
        .into_iter()
        .map(ValueExport::context)
        .chain(
            namespaces
                .iter()
                .map(|name| ValueExport::new(name).context()),
        )
        .collect::<Vec<_>>();
    context! { exports }
}
