use super::*;
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

const SCHEMA: &str = concat!(
    "enum Choice { a, b }\n",
    "inline(value: Number)\nnamed(value: Number)\nnested(value: Number)\n",
    "eager(choice: Choice, value: Number, other: Number)\n",
    "currency(value: Number)\ndate(value: Date)\nchain(value: Number)\nunformatted(value: Number)\n",
);
const SOURCE: &str = concat!(
    "fn Echo(text: String) { _ => {text} }\n",
    "fn Raw(input: Number) { _ => {input} }\n",
    "inline = {fn(copy: value @number) { _ => {copy} }}\n",
    "named = {Echo(value @number)}\nnested = {Echo(Echo(value @number))}\n",
    "eager = {fn(choice, first: value @number, second: other @number) {\n a => {first}\n _ => other\n}}\n",
    "currency = {Echo(value @currency(code = \"USD\"))}\n",
    "date = {Echo(value @date(skeleton = \"yMMMd\"))}\n",
    "chain = {Echo(value @number @currency(code = \"USD\"))}\n",
    "unformatted = {Raw(value)}\n",
);

#[test]
fn formatted_values_execute_once_before_calls_in_ordinary_and_physical_graphs() {
    let schema = lower_schema(&parse_schema_in(SCHEMA, SourceId(1)).unwrap());
    let locales = [EcmaLocaleModule {
        locale: "en".into(),
        module: lower_locale(&parse_locale_in(SOURCE, SourceId(2)).unwrap()),
    }];
    let options = project_options("en");
    let project = ValidatedEcmaProject::try_new(&schema, &locales, &options).unwrap();
    let selected_options = EcmaProjectOptions {
        tree_shaking: true,
        included_messages: vec!["inline".into()],
        ..options.clone()
    };
    let selected_project =
        ValidatedEcmaProject::try_new(&schema, &locales, &selected_options).unwrap();
    let selected_files = crate::generate_project_files(&selected_project).unwrap();
    let runtime = &selected_files
        .iter()
        .find(|file| file.path == "locales/en/_runtime.js")
        .unwrap()
        .contents;
    assert!(runtime.contains("function formatNumber("));
    assert!(!runtime.contains("formatCurrency"));
    assert!(!runtime.contains("formatDate"));
    assert_eq!(
        execute_project(
            &selected_files,
            "import messages from './locales/en.js'; console.log(messages.inline(1234.5));"
        ),
        "1,234.5"
    );
    let mut files = crate::generate_project_files(&project).unwrap();
    let sources = [
        crate::EcmaSource::new(SourceId(1), "schema.lgs", SCHEMA),
        crate::EcmaSource::new(SourceId(2), "en.lgl", SOURCE),
    ];
    let requests = linguini_codegen::formatter_requests(&schema, &locales[0].module);
    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests,
        linguini_codegen::formatter_requests(
            &schema,
            &linguini_codegen::lower_inline_functions(&schema, &locales[0].module)
        )
    );
    for artifact in project.semantic_artifacts().unwrap() {
        let compiled =
            crate::compile_bundler_semantic_module(&project, &artifact, &sources).unwrap();
        files.push(EcmaGeneratedFile {
            path: artifact.module_path,
            contents: compiled.code,
        });
    }
    let mut exports = Vec::new();
    for artifact in project.message_artifacts().unwrap() {
        let compiled =
            crate::compile_bundler_message_artifact_module(&project, &artifact, &sources).unwrap();
        exports.push(format!(
            "export {{ message as {} }} from {:?};",
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
        contents: exports.join("\n"),
    });
    assert_eq!(
        execute_project(
            &files,
            r#"
import assert from 'node:assert/strict';
import ordinary from './locales/en.js';
import * as physical from './physical.js';
Intl.NumberFormat = Intl.DateTimeFormat = class { constructor() { throw Error('host formatting'); } };
for (const messages of [ordinary,physical]) {
  for (const name of ['inline','named','nested']) {
    assert.equal(messages[name](1234.5), '1,234.5', name);
    assert.equal(messages[name]({value:1234.5}), '1,234.5', name);
    const calls=[];
    const input={ [Symbol.toPrimitive](hint) { calls.push(hint); return '1234.5'; } };
    assert.equal(messages[name](input), '1,234.5');
    assert.deepEqual(calls, ['string'], name+' converts its input once');
  }
  assert.equal(messages.currency(1234.5), '$1,234.50');
  assert.equal(messages.date('2024-01-02'), 'Jan 2, 2024');
  assert.equal(messages.chain(12.5), '$12.50');
  assert.equal(messages.unformatted(1234.5), '1234.5', 'forwarding alone does not format an argument');
  for (const [choice,expected] of [['a','1,234.5'],['b','other']]) {
    const calls=[];
    const input=name=>({[Symbol.toPrimitive](hint){calls.push(name+':'+hint);return '1234.5';}});
    assert.equal(messages.eager(choice,input('first'),input('second')),expected);
    assert.deepEqual(calls,['first:string','second:string']);
    const failure=Error('second input failed');
    const broken={toString(){calls.push('failure');throw failure;}};
    assert.throws(()=>messages.eager(choice,0,broken),error=>error===failure);
    assert.equal(calls.filter(value=>value==='failure').length,1);
    assert.throws(()=>messages.eager(choice,0,'invalid'),RangeError);
  }
}
console.log('nested-formatters-ok');
"#
        ),
        "nested-formatters-ok"
    );
}
