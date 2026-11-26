use super::*;
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

#[test]
fn selected_form_attributes_execute_in_ordinary_physical_and_standalone_modules() {
    let schema_source = "enum Fruit { apple, pear }\ntype Item = Fruit\ngender(fruit: Item)\nnom(fruit: Fruit, count: Number)\n";
    let locale_source = "impl Fruit {\napple { Gender = neuter\nemoji = UNUSED_EMOJI\nform nom(Plural) { one => apple\n_ => apples }\nform gen(Plural) { _ => UNUSED_GEN } }\npear { Gender = female\nemoji = UNUSED_EMOJI\nform nom(Plural) { one => pear\n_ => pears }\nform gen(Plural) { _ => UNUSED_GEN } }\n}\ngender = {fruit.Gender}\nnom = {fn(fruit) { _ => {fruit.nom(count)} }}\n";
    let schema = lower_schema(&parse_schema_in(schema_source, SourceId(1)).unwrap());
    let locales = [EcmaLocaleModule {
        locale: "en".into(),
        module: lower_locale(&parse_locale_in(locale_source, SourceId(2)).unwrap()),
    }];
    let sources = [
        crate::EcmaSource::new(SourceId(1), "schema.lgs", schema_source),
        crate::EcmaSource::new(SourceId(2), "en.lgl", locale_source),
    ];
    for selected in [
        vec!["gender".to_owned()],
        vec!["gender".to_owned(), "nom".to_owned()],
    ] {
        let options = EcmaProjectOptions {
            tree_shaking: true,
            included_messages: selected.clone(),
            ..project_options("en")
        };
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
        for artifact in project.message_artifacts().unwrap() {
            let compiled =
                crate::compile_bundler_message_artifact_module(&project, &artifact, &sources)
                    .unwrap();
            files.push(EcmaGeneratedFile {
                path: artifact.module_path,
                contents: compiled.code,
            });
        }
        for message in &selected {
            let file_name = format!("standalone-{message}.js");
            let compiled = crate::compile_message_module(
                &project,
                "en",
                message,
                &file_name,
                "./shared.js",
                &sources,
            )
            .unwrap();
            if message == "gender" {
                assert!(!compiled.code.contains("pluralEn"));
                assert!(!compiled.code.contains("nom:"));
            }
            files.push(EcmaGeneratedFile {
                path: file_name,
                contents: compiled.code,
            });
        }
        let code = files
            .iter()
            .filter(|file| file.path.ends_with(".js"))
            .map(|file| file.contents.as_str())
            .collect::<String>();
        assert!(!code.contains("UNUSED_EMOJI"));
        assert!(!code.contains("UNUSED_GEN"));
        if selected.len() == 1 {
            assert!(!code.contains("pluralEn"));
            assert!(!code.contains("nom:"));
            assert!(!code.contains("nom_inline"));
        }
        let gender_artifact = project
            .message_artifacts()
            .unwrap()
            .into_iter()
            .find(|artifact| artifact.message == "gender")
            .unwrap();
        let script = format!(
            r#"
import assert from 'node:assert/strict';
const ordinary = await import('./locales/en.js');
const physical = await import({physical:?});
const standalone = await import('./standalone-gender.js');
for (const gender of [ordinary.gender, physical.message, standalone.message]) {{
  assert.equal(gender('apple'), 'neuter');
  assert.equal(gender({{fruit:'pear'}}), 'female');
  const hints = [];
  const key = {{[Symbol.toPrimitive](hint) {{ hints.push(hint); return 'apple'; }}}};
  assert.equal(gender({{fruit:key}}), 'neuter');
  assert.deepEqual(hints, ['string']);
  const failure = new Error('key conversion');
  assert.throws(() => gender({{fruit:{{[Symbol.toPrimitive]() {{ throw failure; }}}}}}), error => error === failure);
  for (const key of ['missing', Symbol('missing')]) assert.throws(() => gender(key), TypeError);
  const observations = [];
  Object.defineProperty(Object.prototype, 'Gender', {{configurable:true, get() {{ observations.push(this); return 'prototype'; }}}});
  try {{
    assert.equal(gender('apple'), 'neuter');
    for (const key of ['__proto__','constructor','toString']) assert.equal(gender(key), 'prototype');
    assert.deepEqual(observations, [Object.prototype,Object,Object.prototype.toString]);
  }} finally {{ delete Object.prototype.Gender; }}
}}
if (ordinary.nom) {{
  const standaloneNom = await import('./standalone-nom.js');
  assert.equal(ordinary.nom('apple', 1), 'apple');
  assert.equal(ordinary.nom({{fruit:'pear',count:2}}), 'pears');
  assert.equal(standaloneNom.message('apple', 2), 'apples');
}}
console.log('verified');
"#,
            physical = format!("./{}", gender_artifact.module_path)
        );
        assert_eq!(execute_project(&files, &script), "verified");
    }
    let full = generate_project_files(
        &schema,
        &locales,
        &EcmaProjectOptions {
            tree_shaking: false,
            included_messages: vec!["gender".to_owned()],
            ..project_options("en")
        },
    )
    .unwrap();
    assert!(!full
        .iter()
        .any(|file| file.contents.contains("UNUSED_EMOJI")));
    assert!(!full.iter().any(|file| file.contents.contains("UNUSED_GEN")));
    assert_eq!(
        execute_project(
            &full,
            r#"import assert from 'node:assert/strict'; const {gender,nom}=await import('./locales/en.js'); assert.equal(gender('apple'),'neuter'); assert.equal(nom('pear',2),'pears'); console.log('verified');"#
        ),
        "verified"
    );
}

#[test]
fn form_pruning_preserves_eager_initializers_and_callable_rows() {
    let schema_source =
        "enum Fruit { apple }\nenum Unused { entry }\nlabel(fruit: Fruit)\ncount(fruit: Fruit, value: Number)\n";
    let locale_source = "let marker = EAGER\nfn Echo(value: String) { _ => {value} }\nimpl Fruit { apple {\none => apple\n_ => apples\nlabel = Apple\neager = {Echo(marker)}\nunused = UNUSED\n} }\nimpl Unused { entry { one => unused\n_ => unused\nlabel = UNUSED_CALLABLE } }\nlabel = {fruit.label}\ncount = {fruit(value)}\n";
    let schema = lower_schema(&parse_schema_in(schema_source, SourceId(1)).unwrap());
    let locales = [EcmaLocaleModule {
        locale: "en".into(),
        module: lower_locale(&parse_locale_in(locale_source, SourceId(2)).unwrap()),
    }];
    let project = ValidatedEcmaProject::try_new(
        &schema,
        &locales,
        &EcmaProjectOptions {
            tree_shaking: true,
            ..project_options("en")
        },
    )
    .unwrap();
    let mut files = crate::generate_project_files(&project).unwrap();
    let sources = [
        crate::EcmaSource::new(SourceId(1), "schema.lgs", schema_source),
        crate::EcmaSource::new(SourceId(2), "en.lgl", locale_source),
    ];
    for artifact in project.semantic_artifacts().unwrap() {
        let compiled =
            crate::compile_bundler_semantic_module(&project, &artifact, &sources).unwrap();
        files.push(EcmaGeneratedFile {
            path: artifact.module_path,
            contents: compiled.code,
        });
    }
    let mut physical_exports = String::new();
    for artifact in project.message_artifacts().unwrap() {
        let compiled =
            crate::compile_bundler_message_artifact_module(&project, &artifact, &sources).unwrap();
        physical_exports.push_str(&format!(
            "export {{message as {}}} from {:?};\n",
            artifact.message,
            format!("./{}", artifact.module_path)
        ));
        files.push(EcmaGeneratedFile {
            path: artifact.module_path,
            contents: compiled.code,
        });
    }
    files.push(EcmaGeneratedFile {
        path: "physical.js".into(),
        contents: physical_exports,
    });
    for message in ["label", "count"] {
        let name = format!("standalone-{message}.js");
        let compiled =
            crate::compile_message_module(&project, "en", message, &name, "./shared.js", &sources)
                .unwrap();
        if message == "label" {
            assert!(compiled.code.contains("UNUSED"));
            assert!(
                compiled.code.contains("Object.assign"),
                "callable row prototype is retained"
            );
        } else {
            assert!(
                compiled.code.contains("UNUSED"),
                "whole callable form demand remains conservative"
            );
        }
        files.push(EcmaGeneratedFile {
            path: name,
            contents: compiled.code,
        });
    }
    assert_eq!(
        execute_project(
            &files,
            r#"
import assert from 'node:assert/strict';
const original = globalThis.String;
const assign = Object.assign;
const conversions = [];
const assignments = [];
globalThis.String = value => { conversions.push(value); return original(value); };
Object.assign = (target, ...sources) => { assignments.push(sources.map(source => Object.keys(source))); return assign(target,...sources); };
let ordinary, physical, label, count;
try {
  ordinary = await import('./locales/en.js');
  assert.deepEqual(conversions.splice(0), ['EAGER']);
  assert.deepEqual(assignments.splice(0), [[['label','eager','unused']], [['label']]]);
  physical = await import('./physical.js');
  assert.deepEqual(conversions.splice(0), ['EAGER']);
  assert.deepEqual(assignments.splice(0), [[['label','eager','unused']]]);
  label = (await import('./standalone-label.js')).message;
  assert.deepEqual(conversions.splice(0), ['EAGER']);
  assert.deepEqual(assignments.splice(0), [[['label','eager','unused']]]);
  count = (await import('./standalone-count.js')).message;
  assert.deepEqual(conversions.splice(0), ['EAGER']);
  assert.deepEqual(assignments.splice(0), [[['label','eager','unused']]]);
} finally { globalThis.String = original; Object.assign = assign; }
for (const render of [ordinary.label,physical.label,label]) assert.equal(render('apple'), 'Apple');
for (const render of [ordinary.count,physical.count,count]) {
  assert.equal(render('apple',1), 'apple');
  assert.equal(render({fruit:'apple',value:2}), 'apples');
}
console.log('verified');
"#
        ),
        "verified"
    );
}
