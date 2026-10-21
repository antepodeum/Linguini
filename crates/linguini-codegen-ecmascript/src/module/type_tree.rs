//! One namespace type renderer for locale declarations and the schema-owned contract.

use linguini_ir::{IrModule, IrSymbolKind};
use linguini_syntax::Span;

use crate::ecmascript::EcmaModuleKind;

use super::names::{emit_docs, property_key, safe_identifier};
use super::provenance::map_symbol_lines;
use super::signature::MessageCallSignature;
use super::tree::MessageTree;

pub(super) struct MessageTreeType<'a> {
    target: EcmaModuleKind,
    schema: Option<&'a IrModule>,
}

impl<'a> MessageTreeType<'a> {
    pub(super) fn locale_declaration() -> Self {
        Self {
            target: EcmaModuleKind::Declaration,
            schema: None,
        }
    }

    pub(super) fn schema(schema: &'a IrModule, target: EcmaModuleKind) -> Self {
        Self {
            target,
            schema: Some(schema),
        }
    }

    pub(super) fn render(&self, tree: &MessageTree) -> (String, Vec<(usize, Span)>) {
        self.render_tree(tree, "", 0)
    }

    fn key(&self, name: &str, depth: usize) -> String {
        if self.schema.is_some() && depth == 0 {
            safe_identifier(name)
        } else {
            property_key(name)
        }
    }

    fn map(
        &self,
        kind: IrSymbolKind,
        name: &str,
        start: usize,
        body: &str,
        mappings: &mut Vec<(usize, Span)>,
    ) {
        if let Some(schema) = self.schema {
            map_symbol_lines(schema, kind, name, start, body, mappings);
        }
    }

    fn render_tree(
        &self,
        tree: &MessageTree,
        prefix: &str,
        depth: usize,
    ) -> (String, Vec<(usize, Span)>) {
        let indent = "  ".repeat(depth);
        let child_indent = "  ".repeat(depth + 1);
        let mut body = String::new();
        let mut mappings = Vec::new();
        for entry in &tree.messages {
            let start = body.len();
            let signature = MessageCallSignature::from_message(&entry.signature);
            if self.target.is_declaration() && !signature.is_parameterized() {
                emit_docs(&entry.signature.docs, &child_indent, &mut body);
            }
            let ty = if self.target.is_declaration() {
                signature.callable_type_with_docs(&entry.signature.docs, &child_indent)
            } else {
                signature.javascript_callable_type()
            };
            body.push_str(&crate::templates::render(
                "declarations/property",
                minijinja::context! {
                    indent => &child_indent, name => self.key(&entry.property, depth), ty,
                },
            ));
            self.map(
                IrSymbolKind::Message,
                &entry.signature.name,
                start,
                &body,
                &mut mappings,
            );
        }
        for (name, child) in &tree.children {
            let start = body.len();
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };
            if self.target.is_declaration() {
                emit_docs(&child.docs, &child_indent, &mut body);
            }
            let first_line = body.bytes().filter(|byte| *byte == b'\n').count();
            let (ty, child_mappings) = self.render_tree(child, &path, depth + 1);
            let property = crate::templates::render(
                "declarations/property",
                minijinja::context! {
                    indent => &child_indent, name => self.key(name, depth), ty,
                },
            );
            let group_end = body.len() + property.find('\n').unwrap_or(property.len());
            body.push_str(&property);
            // A namespace owns its documentation and opening line; descendants retain their
            // own origins, rather than inheriting the namespace's source location.
            self.map(
                IrSymbolKind::Group,
                &path,
                start,
                &body[..group_end],
                &mut mappings,
            );
            mappings.extend(
                child_mappings
                    .into_iter()
                    .map(|(line, span)| (first_line + line, span)),
            );
        }
        let code = crate::templates::render("types/object", minijinja::context! { indent, body });
        // The object's opening brace precedes all member lines in both grammars.
        for (line, _) in &mut mappings {
            *line += 1;
        }
        (code, mappings)
    }
}
