//! Declaration provenance within a generated statement, independent of the output target.

use linguini_ir::{IrModule, IrSymbolKind};
use linguini_syntax::Span;

pub(super) fn map_symbol_lines(
    schema: &IrModule,
    kind: IrSymbolKind,
    name: &str,
    start: usize,
    code: &str,
    mappings: &mut Vec<(usize, Span)>,
) {
    let Some(origin) = schema
        .origins()
        .iter()
        .rev()
        .find(|origin| origin.kind == kind && origin.name == name)
    else {
        return;
    };
    let first_line = code[..start].bytes().filter(|byte| *byte == b'\n').count();
    for (offset, line) in code[start..].split_inclusive('\n').enumerate() {
        if !line.trim().is_empty() {
            mappings.push((first_line + offset, origin.span));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ecmascript::{EcmaModuleOutput, EcmaScriptTarget, EcmaSource};
    use linguini_ir::lower_schema;
    use linguini_syntax::{parse_schema_in, SourceId};

    /// Decode the public v3 map rather than inspecting emitter bookkeeping.
    fn positions(map: &str) -> Vec<(usize, usize, usize, usize)> {
        let encoded = map
            .split("\"mappings\":\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let mut source = 0_i64;
        let mut line = 0_i64;
        let mut column = 0_i64;
        encoded
            .split(';')
            .enumerate()
            .filter_map(|(generated, segment)| {
                if segment.is_empty() {
                    return None;
                }
                let mut fields = Vec::new();
                let mut value = 0_i64;
                let mut shift = 0;
                for digit in segment.bytes() {
                    let digit = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
                        .iter()
                        .position(|byte| *byte == digit)
                        .unwrap() as i64;
                    value |= (digit & 31) << shift;
                    if digit & 32 != 0 {
                        shift += 5;
                        continue;
                    }
                    fields.push(if value & 1 == 0 {
                        value >> 1
                    } else {
                        -(value >> 1)
                    });
                    value = 0;
                    shift = 0;
                }
                assert_eq!(fields.len(), 4, "one mapping per declaration line");
                assert_eq!(fields[0], 0);
                source += fields[1];
                line += fields[2];
                column += fields[3];
                Some((generated, source as usize, line as usize, column as usize))
            })
            .collect()
    }

    #[test]
    fn schema_types_map_each_symbol_across_source_files_and_targets() {
        let first = "/// Tone 😀\nenum Tone { warm, cool }\nfirst\n";
        let second = "/// Amount\ntype Amount = Number\n/// Shop docs\nshop {\n  /// Greeting\n  hello(name: String, tone: Tone)\n  nested { title }\n}\n";
        let mut schema = lower_schema(&parse_schema_in(first, SourceId(17)).unwrap());
        schema
            .try_append(lower_schema(&parse_schema_in(second, SourceId(3)).unwrap()))
            .unwrap();
        // Input order deliberately differs from the stable SourceId order.
        let sources = [
            EcmaSource::new(SourceId(17), "schema/first.lgs", first),
            EcmaSource::new(SourceId(3), "schema/second.lgs", second),
        ];
        for target in [EcmaScriptTarget::TypeScript, EcmaScriptTarget::JavaScript] {
            for declaration in [false, true] {
                if declaration && !target.is_typescript() {
                    continue;
                }
                let extension = if declaration {
                    "d.ts"
                } else if target.is_typescript() {
                    "ts"
                } else {
                    "js"
                };
                for (stem, module) in [
                    (
                        "shared",
                        super::super::shared::shared_module(
                            &schema,
                            EcmaModuleOutput::new(target, format!("shared.{extension}"), None),
                            declaration,
                        ),
                    ),
                    (
                        "messages",
                        super::super::messages::messages_module(
                            &schema,
                            EcmaModuleOutput::new(target, format!("messages.{extension}"), None),
                            "./shared.js",
                        ),
                    ),
                ] {
                    assert_eq!(module.source_ids(), [SourceId(3), SourceId(17)]);
                    let ordered =
                        super::super::message::ordered_sources(&module.source_ids(), &sources)
                            .unwrap();
                    let rendered = module.render(&ordered);
                    assert!(rendered
                        .source_map
                        .contains("\"sources\":[\"schema/second.lgs\",\"schema/first.lgs\"]"));
                    assert!(rendered.source_map.contains("Tone 😀\\nenum Tone"));
                    let mappings = positions(&rendered.source_map);
                    for (needle, source, original_line, original_column) in if stem == "shared" {
                        vec![
                            (
                                if target.is_typescript() {
                                    "export type Tone"
                                } else {
                                    "@typedef {\"warm\""
                                },
                                1,
                                1,
                                0,
                            ),
                            (
                                if target.is_typescript() {
                                    "export type Amount"
                                } else {
                                    "@typedef {number"
                                },
                                0,
                                1,
                                0,
                            ),
                        ]
                    } else {
                        vec![
                            ("readonly first:", 1, 2, 0),
                            ("readonly shop:", 0, 3, 0),
                            ("readonly hello:", 0, 5, 2),
                            ("readonly title:", 0, 6, 11),
                        ]
                    } {
                        let generated_line = rendered
                            .code
                            .lines()
                            .position(|line| line.contains(needle))
                            .unwrap();
                        assert!(
                            mappings.contains(&(
                                generated_line,
                                source,
                                original_line,
                                original_column
                            )),
                            "{stem}.{extension}: {needle} at {generated_line}: {mappings:?}"
                        );
                    }
                }
            }
        }
    }
}
