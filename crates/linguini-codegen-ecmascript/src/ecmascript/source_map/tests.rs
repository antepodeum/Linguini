use super::{source_positions, SourcePosition};
use crate::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaReExport,
    EcmaSource, EcmaStatement,
};
use linguini_syntax::{SourceId, Span};

#[test]
fn source_offsets_preserve_utf16_positions_and_original_mapping_order() {
    let sources = [
        EcmaSource::new(SourceId(9), "old.lgs", "superseded"),
        EcmaSource::new(SourceId(3), "second.lgs", "\nA\rB"),
        EcmaSource::new(SourceId(9), "last.lgs", "é😀\r\nx𑄶z\nশেষ"),
    ];
    let cases = [
        (9, 24, Some((2, 2, 3))),
        (3, 3, Some((1, 1, 2))),
        (9, 13, Some((2, 1, 3))),
        (9, 0, Some((2, 0, 0))),
        (9, 9, Some((2, 1, 1))),
        (9, 2, Some((2, 0, 1))),
        (9, 7, Some((2, 0, 4))),
        (9, 15, Some((2, 2, 0))),
        (9, 13, Some((2, 1, 3))),
        (3, 4, Some((1, 1, 3))),
        (3, 0, Some((1, 0, 0))),
        (3, 1, Some((1, 1, 0))),
        (9, 3, None), // Inside a supplementary UTF-8 character.
        (9, 25, None),
        (77, 0, None),
    ];
    let mappings = cases
        .iter()
        .enumerate()
        .map(|(line, (id, offset, _))| (line, Span::in_source(SourceId(*id), *offset, *offset)))
        .collect::<Vec<_>>();
    let expected = cases
        .iter()
        .map(|(_, _, position)| {
            position.map(|(source_index, line, column)| SourcePosition {
                source_index,
                line,
                column,
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(source_positions(&sources, &mappings), expected);
    assert!(source_positions(&sources, &[]).is_empty());
}

#[test]
fn module_mapping_lines_follow_edges_empty_statements_and_trimmed_newlines() {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::JavaScript,
        "output.js",
        None,
    ));
    module.push_import(EcmaImport {
        specifier: "./erased.js".into(),
        bindings: EcmaImportBindings::TypeNamed(vec![]),
    });
    module.push_import(EcmaImport {
        specifier: "./dependency\nname.js".into(),
        bindings: EcmaImportBindings::SideEffect,
    });
    module.push_re_export(EcmaReExport::all("./other.js"));
    module.push_statement(EcmaStatement::type_declaration(
        "export type Erased = string;\n",
        Some(Span::in_source(SourceId(9), 0, 0)),
    ));
    for (code, offset) in [
        ("/* first\r\nsecond */\n\n", 2),
        ("", 9),
        ("export const result = 1;\n", 24),
    ] {
        module.push_statement(EcmaStatement::mapped(
            code,
            Span::in_source(SourceId(9), offset, offset),
        ));
    }
    let result = module.render(&[EcmaSource::new(SourceId(9), "input.lgs", "é😀\r\nx𑄶z\nশেষ")]);
    let map: serde_json::Value = serde_json::from_str(&result.source_map).unwrap();
    assert_eq!(map["mappings"], ";;;AAAC;;AACA;AACE");
    assert_eq!(
        result.code,
        "import \"./dependency\\nname.js\";\nexport * from \"./other.js\";\n\n/* first\r\nsecond */\n\nexport const result = 1;\n//# sourceMappingURL=output.js.map\n"
    );
}
