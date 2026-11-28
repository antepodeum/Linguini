use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, EcmaSource};
use linguini_ir::lower_schema;
use linguini_syntax::{parse_schema_in, SourceId};

#[test]
fn indexed_origins_preserve_kind_and_last_override_for_appended_lines() {
    use linguini_ir::{IrModuleBuilder, IrOrigin, IrSymbolKind};
    let first = lower_schema(&parse_schema_in("first\n", SourceId(1)).unwrap()).origins()[0].span;
    let second = lower_schema(&parse_schema_in("second\n", SourceId(2)).unwrap()).origins()[0].span;
    let mut builder = IrModuleBuilder::new();
    for (kind, span, is_override) in [
        (IrSymbolKind::Message, first, false),
        (IrSymbolKind::Group, first, false),
        (IrSymbolKind::Message, second, true),
    ] {
        builder = builder.push_origin(IrOrigin {
            kind,
            name: "same".to_owned(),
            span,
            is_override,
        });
    }
    let schema = builder.build().unwrap();
    let origins = super::DeclarationOrigins::new(&schema);
    let mut mappings = vec![];
    origins.map_lines(
        IrSymbolKind::Message,
        "same",
        13,
        "one\n \t\ntwo",
        &mut mappings,
    );
    origins.map_lines(IrSymbolKind::Group, "same", 27, "group\n", &mut mappings);
    origins.map_lines(
        IrSymbolKind::TypeAlias,
        "same",
        31,
        "absent\n",
        &mut mappings,
    );
    origins.map_lines(
        IrSymbolKind::Message,
        "absent",
        32,
        "absent\n",
        &mut mappings,
    );
    assert_eq!(mappings, [(13, second), (15, second), (27, first)]);
}

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
    let second = "/// Amount\ntype Amount = Number\n/// Shop docs\nshop {\n  /// Greeting\n  hello(name: String, tone: Tone)\n  nested { title }\n  /// Sibling docs\n  sibling { title }\n}\n/// Last docs\ntail { title }\n";
    let mut schema = lower_schema(&parse_schema_in(first, SourceId(17)).unwrap());
    schema
        .try_append(lower_schema(&parse_schema_in(second, SourceId(3)).unwrap()))
        .unwrap();
    // Input order deliberately differs from the stable SourceId order.
    let sources = [
        EcmaSource::new(SourceId(17), "schema/first.lgs", first),
        EcmaSource::new(SourceId(3), "schema/second.lgs", second),
    ];
    for target in [EcmaModuleKind::Declaration, EcmaModuleKind::JavaScript] {
        {
            let declaration = target.is_declaration();
            let extension = if declaration { "d.ts" } else { "js" };
            for (stem, module) in [
                (
                    "shared",
                    super::super::shared::shared_module(
                        &schema,
                        EcmaModuleOutput::new(target, format!("shared.{extension}"), None),
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
                    super::super::message::ordered_sources(&module.source_ids(), &sources).unwrap();
                let rendered = module.render(&ordered);
                assert!(rendered
                    .source_map
                    .contains("\"sources\":[\"schema/second.lgs\",\"schema/first.lgs\"]"));
                assert!(rendered.source_map.contains("Tone 😀\\nenum Tone"));
                let mappings = positions(&rendered.source_map);
                for (needle, source, original_line, original_column) in if stem == "shared" {
                    vec![
                        (
                            if target.is_declaration() {
                                "export type Tone"
                            } else {
                                "@typedef {\"warm\""
                            },
                            1,
                            1,
                            0,
                        ),
                        (
                            if target.is_declaration() {
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
                        ("readonly sibling:", 0, 8, 2),
                        ("readonly tail:", 0, 11, 0),
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
