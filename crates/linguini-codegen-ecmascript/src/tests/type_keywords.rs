use super::*;
use crate::test_runtime::root_project_artifacts;
use linguini_ir::{IrModuleBuilder, LocaleIr, SchemaIr};

#[test]
fn accepted_ir_type_keywords_keep_narrow_checked_js_and_declaration_contracts() {
    let names = [
        "any",
        "unknown",
        "never",
        "number",
        "bigint",
        "boolean",
        "string",
        "scoped.symbol",
        "object",
        "undefined",
        "keyof",
        "unique",
        "readonly",
        "infer",
        "as",
        "void",
        "intrinsic",
        "type",
        "abstract",
        "constructor",
        "__lgl_type_string",
    ];
    let arguments = names
        .iter()
        .enumerate()
        .map(|(index, name)| match index {
            0 => "string".to_owned(),
            1 => "number".to_owned(),
            _ if name.starts_with("__lgl_") => "arg_custom".to_owned(),
            _ => format!("arg_{}", name.rsplit('.').next().unwrap()),
        })
        .collect::<Vec<_>>();
    let function_parameters = arguments
        .iter()
        .map(|name| format!("{name}: Choice"))
        .collect::<Vec<_>>()
        .join(", ");
    let body = arguments
        .iter()
        .map(|name| format!("{{{name}}}"))
        .collect::<Vec<_>>()
        .join("|");
    let source = format!(
        "fn Forward({function_parameters}) {{ _ => {body} }}\nroot = {{Forward({})}}\n",
        arguments.join(", ")
    );
    let base = lower_schema(&parse_schema("enum Choice { entry }\nroot(value: Choice)\n").unwrap());
    let alias_base =
        lower_schema(&parse_schema("type Choice = String\nroot(value: Choice)\n").unwrap());
    let implementation = lower_locale(&parse_locale(&source).unwrap());
    for kind in ["enum", "alias"] {
        let mut schema = IrModuleBuilder::new();
        let mut signature = base.messages()[0].clone();
        signature.parameters.clear();
        for (name, argument) in names.iter().zip(&arguments) {
            if kind == "enum" {
                let mut item = base.enums()[0].clone();
                item.name = (*name).into();
                if name.starts_with("__lgl_") {
                    item.variants = vec!["other".into()];
                }
                schema = schema.push_enum(item);
            } else {
                let mut item = alias_base.type_aliases()[0].clone();
                item.name = (*name).into();
                schema = schema.push_type_alias(item);
            }
            let mut parameter = base.messages()[0].parameters[0].clone();
            parameter.name = argument.clone();
            parameter.ty = (*name).into();
            signature.parameters.push(parameter);
        }
        let schema =
            SchemaIr::try_from_module(schema.push_message(signature).build().unwrap()).unwrap();
        let mut function = implementation.functions()[0].clone();
        for (parameter, name) in function.parameters.iter_mut().zip(names) {
            parameter.ty = name.into();
        }
        let locale = LocaleIr::try_from_module(
            IrModuleBuilder::new()
                .push_function(function)
                .push_message(implementation.messages()[0].clone())
                .build()
                .unwrap(),
        )
        .unwrap();
        let locales = [EcmaLocaleModule {
            locale: "en".into(),
            module: locale,
        }];
        let project =
            ValidatedEcmaProject::try_new(&schema, &locales, &project_options("en")).unwrap();
        let sources = [crate::EcmaSource::new(
            linguini_syntax::SourceId(0),
            "en.lgl",
            &source,
        )];
        let (files, physical) = root_project_artifacts("en", &project, &sources);
        for file in files.iter().filter(|file| {
            file.path.starts_with("shared.")
                || file.path.starts_with("messages.")
                || file.path.starts_with("locales/")
                || file.path.starts_with("bundler/")
                || file.path == "standalone.js"
        }) {
            assert_snapshot(&format!("tests/fixtures/golden/snapshots/js-complete-project/type-names/keywords/{kind}/{}", file.path), &file.contents);
        }
        let values = names.map(|name| {
            if name.starts_with("__lgl_") {
                "other"
            } else {
                "entry"
            }
        });
        let script = format!(
            r#"
import assert from 'node:assert/strict';
const ordinary=await import('./locales/en.js');
const physical=await import({physical:?});
const standalone=await import('./standalone.js');
const keys={keys},values={values};
const args=Object.fromEntries(keys.map((key,index)=>[key,values[index]]));
for(const root of [ordinary.root,physical.message,standalone.message]) {{
 assert.equal(root(...values),values.join('|'));
 assert.equal(root(args),values.join('|'));
 const reads=[];
 const observed=Object.defineProperties({{}},Object.fromEntries(keys.map((key,index)=>[key,{{enumerable:true,get(){{reads.push(key);return values[index]}}}}])));
 assert.equal(root(observed),values.join('|'));assert.deepEqual(reads,keys);
}}
console.log('verified');
"#,
            keys = serde_json::to_string(&arguments).unwrap(),
            values = serde_json::to_string(&values).unwrap()
        );
        assert_eq!(execute_project(&files, &script), "verified");
    }
}
