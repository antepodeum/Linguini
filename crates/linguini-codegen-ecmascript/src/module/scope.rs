//! Source parameter types and their private ECMAScript bindings share one lexical scope.

use std::collections::BTreeMap;

use super::names::{property_access, public_value_name, safe_identifier};

#[derive(Default)]
pub(super) struct ExpressionScope {
    types: BTreeMap<String, String>,
    bindings: BTreeMap<String, String>,
}

impl From<BTreeMap<String, String>> for ExpressionScope {
    fn from(types: BTreeMap<String, String>) -> Self {
        Self {
            types,
            bindings: BTreeMap::new(),
        }
    }
}

impl ExpressionScope {
    pub(super) fn new(values: impl IntoIterator<Item = (String, String, String)>) -> Self {
        let mut scope = Self::default();
        for (source, ty, binding) in values {
            scope.types.insert(source.clone(), ty);
            scope.bindings.insert(source, binding);
        }
        scope
    }

    pub(super) fn get(&self, name: &str) -> Option<&String> {
        self.types.get(name)
    }

    pub(super) fn keys(&self) -> impl Iterator<Item = &String> {
        self.types.keys()
    }

    pub(super) fn binding(&self, name: &str) -> String {
        self.bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| safe_identifier(name))
    }

    pub(super) fn path(&self, path: &[String]) -> String {
        let Some((root, properties)) = path.split_first() else {
            return String::new();
        };
        let mut output = self.bindings.get(root).cloned().unwrap_or_else(|| {
            if self.types.contains_key(root) {
                safe_identifier(root)
            } else {
                public_value_name(root)
            }
        });
        for property in properties {
            output.push_str(&property_access(property));
        }
        output
    }
}
