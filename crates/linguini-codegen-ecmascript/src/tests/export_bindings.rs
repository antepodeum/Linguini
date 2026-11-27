use super::*;
use linguini_ir::{IrModuleBuilder, IrTextPart, LocaleIr, SchemaIr};

fn public_models() -> (SchemaIr, LocaleIr) {
    let numeric = lower_schema(&parse_schema("root(value: Number)\n").unwrap());
    let formatted = lower_locale(&parse_locale("root = {value @number}\n").unwrap());
    let choices =
        lower_schema(&parse_schema("enum Choice { first }\nroot(value: Choice)\n").unwrap());
    let choice = lower_locale(&parse_locale("root = {value}\n").unwrap());
    let literal = lower_locale(&parse_locale("root = Catalog\n").unwrap());
    let mut schema = IrModuleBuilder::new().push_enum(choices.enums()[0].clone());
    let mut locale = IrModuleBuilder::new();
    for name in ["formatNumber", "normalizeMessageArgs", "message"] {
        let mut signature = numeric.messages()[0].clone();
        signature.name = name.into();
        let mut implementation = formatted.messages()[0].clone();
        implementation.name = name.into();
        schema = schema.push_message(signature);
        locale = locale.push_message(implementation);
    }
    let mut signature = choices.messages()[0].clone();
    signature.name = "String".into();
    let mut implementation = choice.messages()[0].clone();
    implementation.name = "String".into();
    schema = schema.push_message(signature);
    locale = locale.push_message(implementation);
    let mut signature = numeric.messages()[0].clone();
    signature.name = "lgl".into();
    signature.parameters.clear();
    let mut implementation = literal.messages()[0].clone();
    implementation.name = "lgl".into();
    (
        SchemaIr::try_from_module(schema.push_message(signature).build().unwrap()).unwrap(),
        LocaleIr::try_from_module(locale.push_message(implementation).build().unwrap()).unwrap(),
    )
}

#[test]
fn public_value_exports_preserve_names_without_shadowing_compiler_bindings() {
    let (schema, locale) = public_models();
    let locales = [EcmaLocaleModule {
        locale: "en".into(),
        module: locale,
    }];
    let options = project_options("en");
    let project = ValidatedEcmaProject::try_new(&schema, &locales, &options).unwrap();
    let files = crate::generate_project_files(&project).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-complete-project/bindings/public-values");
    for file in files
        .iter()
        .filter(|file| file.path.starts_with("locales/") || file.path.starts_with("shared."))
    {
        let path = root.join(&file.path);
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &file.contents).unwrap();
        }
        assert_eq!(file.contents, fs::read_to_string(path).unwrap());
    }
    assert_eq!(
        execute_project(
            &files,
            r#"
import assert from 'node:assert/strict';
const module=await import('./locales/en.js');
const {createLinguini}=await import('./index.js');
assert.deepEqual(Object.keys(module).sort(),['String','default','formatNumber','lgl','message','normalizeMessageArgs'].sort());
assert.equal(module.formatNumber({value:1234.5}),'1,234.5');
assert.equal(module.normalizeMessageArgs(2n),'2');
assert.equal(module.message(2),'2');
assert.equal(module.String('first'),'first');
assert.equal(module.lgl,'Catalog');
for(const name of ['formatNumber','normalizeMessageArgs','message','String','lgl']) assert.equal(module[name],module.default[name]);
assert.equal(createLinguini('en'),module.default);
console.log('verified');
"#
        ),
        "verified"
    );
}

#[test]
fn private_source_functions_cannot_redeclare_generated_helpers() {
    for name in [
        "formatNumber",
        "normalizeMessageArgs",
        "parseGeneratedDecimal",
        "roundTo3FractionDigits",
        "throwInvalidNumber",
        "MAX_GENERATED_DECIMAL_DIGITS",
        "lgl",
        "message",
    ] {
        let schema = lower_schema(&parse_schema("root(value: Number)\n").unwrap());
        let sources = [crate::EcmaSource::new(
            linguini_syntax::SourceId(0),
            "schema.lgs",
            "root(value: Number)\n",
        )];
        let parsed = lower_locale(&parse_locale("fn Helper(value: Number) { _ => USER {value} }\nroot = {Helper(value)} {value @number}\n").unwrap());
        let mut function = parsed.functions()[0].clone();
        function.name = name.into();
        let mut message = parsed.messages()[0].clone();
        for part in &mut message.body.as_mut().unwrap().parts {
            if let IrTextPart::Placeholder(expression) = part {
                if expression.path == ["Helper"] {
                    expression.path[0] = name.into();
                }
            }
        }
        let locale = LocaleIr::try_from_module(
            IrModuleBuilder::new()
                .push_function(function)
                .push_message(message)
                .build()
                .unwrap(),
        )
        .unwrap();
        let locales = [EcmaLocaleModule {
            locale: "en".into(),
            module: locale,
        }];
        let options = project_options("en");
        let project = ValidatedEcmaProject::try_new(&schema, &locales, &options).unwrap();
        let mut files = crate::generate_project_files(&project).unwrap();
        for artifact in project.semantic_artifacts().unwrap() {
            let compiled =
                crate::compile_bundler_semantic_module(&project, &artifact, &sources).unwrap();
            files.push(EcmaGeneratedFile {
                path: artifact.module_path,
                contents: compiled.code,
            });
        }
        let artifact = project.message_artifacts().unwrap().remove(0);
        let physical =
            crate::compile_bundler_message_artifact_module(&project, &artifact, &sources).unwrap();
        files.push(EcmaGeneratedFile {
            path: artifact.module_path.clone(),
            contents: physical.code,
        });
        let standalone = crate::compile_message_module(
            &project,
            "en",
            "root",
            "standalone.js",
            "./shared.js",
            &sources,
        )
        .unwrap();
        files.push(EcmaGeneratedFile {
            path: "standalone.js".into(),
            contents: standalone.code,
        });
        let physical = format!("./{}", artifact.module_path);
        let script = format!(
            r#"
import assert from 'node:assert/strict';
const ordinary=await import('./locales/en.js');
const physical=await import({physical:?});
const standalone=await import('./standalone.js');
for(const root of [ordinary.root,physical.message,standalone.message]) {{
  assert.equal(root({{value:2n}}),'USER 2 2');
  assert.equal(root(NaN),'USER NaN NaN');
  assert.throws(()=>root('invalid'),RangeError);
}}
console.log('verified');
"#
        );
        assert_eq!(execute_project(&files, &script), "verified", "{name}");
    }
}
