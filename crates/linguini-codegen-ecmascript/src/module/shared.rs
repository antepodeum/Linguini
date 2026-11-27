use linguini_ir::{IrModule, IrSymbolKind};
use linguini_syntax::Span;

use crate::ecmascript::{EcmaModule, EcmaModuleOutput, EcmaStatement};

use super::emit::{emit_enum, emit_type_alias};
use super::names::{emit_docs_with_tags, safe_type_identifier, JsDocTag};
use super::provenance::map_symbol_lines;
use super::shared_runtime::{shared_declarations, shared_runtime};
use super::type_model::{render_jsdoc_type, render_string_union, TypeModel};

pub fn generate_shared_module(
    schema: &IrModule,
    output: EcmaModuleOutput,
    declaration: bool,
) -> String {
    shared_module(schema, output, declaration).render_code()
}

pub(super) fn shared_module(
    schema: &IrModule,
    output: EcmaModuleOutput,
    declaration: bool,
) -> EcmaModule {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    let mut types = String::new();
    let mut mappings = Vec::new();
    if target.is_declaration() {
        for item in schema.enums() {
            let start = types.len();
            emit_enum(item, &mut types);
            map_symbol_lines(
                schema,
                IrSymbolKind::Enum,
                &item.name,
                start,
                &types,
                &mut mappings,
            );
        }
        for item in schema.type_aliases() {
            let start = types.len();
            emit_type_alias(item, &mut types);
            map_symbol_lines(
                schema,
                IrSymbolKind::TypeAlias,
                &item.name,
                start,
                &types,
                &mut mappings,
            );
        }
    } else {
        emit_jsdoc_schema_types(schema, &mut types, &mut mappings);
    }
    let has_types = !types.trim().is_empty();
    if has_types {
        module.push_statement_with_mappings(
            if target.is_declaration() {
                EcmaStatement::type_declaration(types, None)
            } else {
                EcmaStatement::documentation(types, None)
            },
            mappings,
        );
    }
    let runtime = if declaration {
        shared_declarations()
    } else {
        shared_runtime()
    };
    module.push_statement(EcmaStatement::generated(if has_types {
        format!("\n{runtime}")
    } else {
        runtime
    }));
    module
}

fn emit_jsdoc_schema_types(
    schema: &IrModule,
    output: &mut String,
    mappings: &mut Vec<(usize, Span)>,
) {
    for item in schema.enums() {
        let start = output.len();
        let ty = render_string_union(&item.variants);
        emit_docs_with_tags(
            &item.docs,
            &[JsDocTag::Typedef {
                name: safe_type_identifier(&item.name),
                ty,
            }],
            "",
            output,
        );
        map_symbol_lines(
            schema,
            IrSymbolKind::Enum,
            &item.name,
            start,
            output,
            mappings,
        );
    }
    for item in schema.type_aliases() {
        let start = output.len();
        emit_docs_with_tags(
            &item.docs,
            &[JsDocTag::Typedef {
                name: safe_type_identifier(&item.name),
                ty: render_jsdoc_type(&TypeModel::from_source_name(&item.target)),
            }],
            "",
            output,
        );
        map_symbol_lines(
            schema,
            IrSymbolKind::TypeAlias,
            &item.name,
            start,
            output,
            mappings,
        );
    }
}

#[cfg(test)]
mod tests {
    use linguini_ir::lower_schema_typed;
    use linguini_syntax::parse_schema;

    use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput};

    use super::generate_shared_module;

    #[test]
    fn record_named_source_types_preserve_shared_helpers_and_public_names() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-complete-project/type-names");
        let mut files = Vec::new();
        for (kind, source) in [
            ("enum", "enum Record { apple, pear }\n"),
            ("alias", "type Record = String\n"),
        ] {
            let schema = lower_schema_typed(&parse_schema(source).unwrap());
            for mut file in crate::generate_schema_files(&schema)
                .into_iter()
                .filter(|file| file.path.starts_with("shared."))
            {
                let path = root.join(kind).join(&file.path);
                if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    std::fs::write(&path, &file.contents).unwrap();
                }
                assert_eq!(file.contents, std::fs::read_to_string(path).unwrap());
                file.path = format!("{kind}/{}", file.path);
                files.push(file);
            }
        }
        assert_eq!(
            crate::test_runtime::execute_project(
                &files,
                r#"
import assert from 'node:assert/strict';
import {normalizeMessageArgs as enumArgs} from './enum/shared.js';
import {normalizeMessageArgs as aliasArgs} from './alias/shared.js';
for (const normalize of [enumArgs, aliasArgs]) {
  assert.deepEqual(normalize([{value:'apple'}], ['value']), ['apple']);
  assert.throws(()=>normalize([{value:'apple', extra:1}], ['value']), TypeError);
}
console.log('ok');
"#
            ),
            "ok"
        );
    }

    #[test]
    fn javascript_schema_module_uses_jsdoc_without_typescript_syntax() {
        let schema = lower_schema_typed(
            &parse_schema("/// Tone docs\nenum Tone { warm, cool }\ntype Amount = Number\n")
                .expect("schema"),
        );
        let output = generate_shared_module(
            schema.as_module(),
            EcmaModuleOutput::new(
                EcmaModuleKind::JavaScript,
                "shared.js",
                Some("shared.d.ts".to_owned()),
            ),
            false,
        );

        assert!(output.contains("@typedef {\"warm\" | \"cool\"} Tone"));
        assert!(output.contains("@typedef {number | bigint | string} Amount"));
        assert!(!output.contains("selectBranch"));
        assert!(output.contains("export function normalizeMessageArgs(values, keys)"));
        assert!(!output.contains("export type"));
        assert!(!output.contains(" as Record"));
        assert_eq!(
            crate::test_runtime::execute_project(
                &[crate::EcmaGeneratedFile { path: "shared.js".into(), contents: output }],
                "import {normalizeMessageArgs} from './shared.js'; console.log(normalizeMessageArgs([{value:'ok'}],['value'])[0]);"
            ),
            "ok"
        );
    }
}
