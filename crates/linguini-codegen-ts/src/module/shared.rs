use linguini_ir::{IrModule, IrSymbolKind};
use linguini_syntax::Span;

use crate::ecmascript::{EcmaModule, EcmaModuleOutput, EcmaStatement};

use super::emit::{emit_enum, emit_type_alias};
use super::names::{emit_docs_with_tags, escape_string, safe_identifier, JsDocTag};
use super::provenance::map_symbol_lines;
use super::shared_runtime::{shared_declarations, shared_runtime};
use super::type_model::{render_jsdoc_type, TypeModel};

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
    if target.is_typescript() {
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
            if target.is_typescript() {
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
        shared_runtime(target)
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
        let ty = item
            .variants
            .iter()
            .map(|variant| format!("\"{}\"", escape_string(variant)))
            .collect::<Vec<_>>()
            .join(" | ");
        emit_docs_with_tags(
            &item.docs,
            &[JsDocTag::Typedef {
                name: safe_identifier(&item.name),
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
                name: safe_identifier(&item.name),
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

    use crate::ecmascript::{EcmaModuleOutput, EcmaScriptTarget};

    use super::generate_shared_module;

    #[test]
    fn javascript_schema_module_uses_jsdoc_without_typescript_syntax() {
        let schema = lower_schema_typed(
            &parse_schema("/// Tone docs\nenum Tone { warm, cool }\ntype Amount = Number\n")
                .expect("schema"),
        );
        let output = generate_shared_module(
            schema.as_module(),
            EcmaModuleOutput::new(
                EcmaScriptTarget::JavaScript,
                "shared.js",
                Some("shared.d.ts".to_owned()),
            ),
            false,
        );

        assert!(output.contains("@typedef {\"warm\" | \"cool\"} Tone"));
        assert!(output.contains("@typedef {number | bigint | string} Amount"));
        assert!(output.contains("@template T"));
        assert!(output.contains("export function normalizeMessageArgs(values, keys)"));
        assert!(!output.contains("export type"));
        assert!(!output.contains(" as Record"));
        assert!(!output.contains(": string"));
    }
}
