use linguini_ir::IrModule;

use crate::ecmascript::{EcmaModule, EcmaModuleOutput, EcmaStatement};

use super::emit::{emit_enums, emit_type_aliases};
use super::names::{emit_docs_with_tags, escape_string, safe_identifier, JsDocTag};
use super::templates::{SHARED_DECLARATIONS, SHARED_RUNTIME};
use super::type_model::{render_jsdoc_type, TypeModel};

pub fn generate_shared_module(
    schema: &IrModule,
    output: EcmaModuleOutput,
    declaration: bool,
) -> String {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    let mut types = String::new();
    if target.is_typescript() {
        emit_enums(schema, &mut types);
        emit_type_aliases(schema, &mut types);
    } else {
        emit_jsdoc_schema_types(schema, &mut types);
    }
    let has_types = !types.trim().is_empty();
    if has_types {
        module.push_statement(if target.is_typescript() {
            EcmaStatement::type_declaration(types, None)
        } else {
            EcmaStatement::documentation(types, None)
        });
    }
    let runtime = if declaration {
        SHARED_DECLARATIONS.to_owned()
    } else if target.is_typescript() {
        SHARED_RUNTIME.to_owned()
    } else {
        javascript_shared_runtime()
    };
    module.push_statement(EcmaStatement::generated(if has_types {
        format!("\n{runtime}")
    } else {
        runtime
    }));
    module.render_code()
}

fn emit_jsdoc_schema_types(schema: &IrModule, output: &mut String) {
    for item in schema.enums() {
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
    }
    for item in schema.type_aliases() {
        emit_docs_with_tags(
            &item.docs,
            &[JsDocTag::Typedef {
                name: safe_identifier(&item.name),
                ty: render_jsdoc_type(&TypeModel::from_source_name(&item.target)),
            }],
            "",
            output,
        );
    }
}

fn javascript_shared_runtime() -> String {
    r#"/**
 * @template T
 * @param {string} key
 * @param {Record<string, T>} branches
 * @returns {T}
 */
export function selectBranch(key, branches) {
  if (Object.prototype.hasOwnProperty.call(branches, key)) {
    return branches[key];
  }
  if (Object.prototype.hasOwnProperty.call(branches, "_")) {
    return branches._;
  }
  if (Object.prototype.hasOwnProperty.call(branches, "other")) {
    return branches.other;
  }
  throw new Error(`Linguini dispatch has no branch for key ${JSON.stringify(key)}`);
}

/**
 * @param {readonly unknown[]} values
 * @param {readonly string[]} keys
 * @returns {unknown[]}
 */
export function normalizeMessageArgs(values, keys) {
  if (values.length !== 1) {
    return [...values];
  }
  const candidate = values[0];
  if (typeof candidate !== "object" || candidate === null) {
    return [...values];
  }
  const record = /** @type {Record<string, unknown>} */ (candidate);
  if (!keys.every((key) => Object.prototype.hasOwnProperty.call(record, key))) {
    return [...values];
  }
  const unknown = Reflect.ownKeys(record)
    .filter((key) => typeof key !== "string" || !keys.includes(key))
    .map(String)
    .sort();
  if (unknown.length > 0) {
    throw new TypeError(
      `Linguini message arguments contain unknown keys: ${unknown.join(", ")}`,
    );
  }
  return keys.map((key) => record[key]);
}
"#
    .to_owned()
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
