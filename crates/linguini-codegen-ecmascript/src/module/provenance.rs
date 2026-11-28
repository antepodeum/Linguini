//! Declaration provenance within a generated statement, independent of the output target.

use linguini_ir::{IrModule, IrSymbolKind};
use linguini_syntax::Span;
use std::collections::BTreeMap;

/// Index once per rendered schema, retaining the last origin of each declaration.
pub(super) struct DeclarationOrigins<'a> {
    symbols: BTreeMap<IrSymbolKind, BTreeMap<&'a str, Span>>,
}

impl<'a> DeclarationOrigins<'a> {
    pub(super) fn new(schema: &'a IrModule) -> Self {
        let mut symbols: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();
        for origin in schema.origins() {
            symbols
                .entry(origin.kind)
                .or_default()
                .insert(origin.name.as_str(), origin.span);
        }
        Self { symbols }
    }

    /// Map only newly emitted text; callers retain its already known starting line.
    pub(super) fn map_lines(
        &self,
        kind: IrSymbolKind,
        name: &str,
        first_line: usize,
        code: &str,
        mappings: &mut Vec<(usize, Span)>,
    ) {
        let Some(&span) = self.symbols.get(&kind).and_then(|names| names.get(name)) else {
            return;
        };
        for (offset, line) in code.split_inclusive('\n').enumerate() {
            if !line.trim().is_empty() {
                mappings.push((first_line + offset, span));
            }
        }
    }
}

#[cfg(test)]
mod tests;
