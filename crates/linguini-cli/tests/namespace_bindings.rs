use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn namespace_bindings_preserve_public_names_and_compiler_helpers() {
    let project = TempDir::new().unwrap();
    let root = project.path();
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::write(root.join("package.json"), "{\"type\":\"module\"}").unwrap();
    fs::write(root.join("linguini.toml"), "[project]\nname=\"namespace-bindings\"\ndefault_locale=\"en\"\nlocales=[\"en\"]\n[paths]\nschema=\"schema\"\nlocale=\"locales\"\n[targets.ts]\nout=\"generated\"\n").unwrap();
    for (namespace, schema, source) in [
        (
            "formatNumber",
            "number(value: Number)\n",
            "number = {value @number}\n",
        ),
        (
            "normalizeMessageArgs",
            "hello(name: String)\n",
            "hello = Hello {name}\n",
        ),
        ("lgl", "hello\n", "hello = Locale\n"),
        (
            "String",
            "enum Choice { first }\nnumber(value: Choice)\n",
            "number = {value}\n",
        ),
        ("main", "main\n", "main = Main\n"),
        (
            "pluralEn",
            "count(value: Number)\n",
            "fn Count(Plural) {\n  one => One\n  _ => Many\n}\ncount = {Count(value)}\n",
        ),
    ] {
        fs::write(root.join(format!("schema/{namespace}.lgs")), schema).unwrap();
        fs::create_dir_all(root.join(format!("locales/{namespace}"))).unwrap();
        fs::write(root.join(format!("locales/{namespace}/en.lgl")), source).unwrap();
    }
    for action in ["check", "build"] {
        assert_cmd::cargo::cargo_bin_cmd!("linguini")
            .current_dir(root)
            .arg(action)
            .assert()
            .success();
    }
    let snapshots = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-complete-project/bindings/namespaces");
    let mut paths = vec![
        "shared.js".to_owned(),
        "shared.d.ts".to_owned(),
        "locales/en.js".to_owned(),
        "locales/en.d.ts".to_owned(),
        "locales/en/_runtime.js".to_owned(),
        "locales/en/_globals.js".to_owned(),
    ];
    for namespace in [
        "formatNumber",
        "normalizeMessageArgs",
        "lgl",
        "String",
        "main",
        "pluralEn",
    ] {
        for extension in ["js", "d.ts"] {
            paths.push(format!("locales/en/{namespace}.{extension}"));
        }
    }
    let baseline = paths
        .iter()
        .map(|path| fs::read(root.join("generated").join(path)).unwrap())
        .collect::<Vec<_>>();
    for (path, bytes) in paths.iter().zip(&baseline) {
        let expected = snapshots.join(path);
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            fs::create_dir_all(expected.parent().unwrap()).unwrap();
            fs::write(&expected, bytes).unwrap();
        }
        assert_eq!(*bytes, fs::read(expected).unwrap(), "{path}");
    }
    assert_cmd::cargo::cargo_bin_cmd!("linguini")
        .current_dir(root)
        .arg("build")
        .assert()
        .success();
    assert_eq!(
        paths
            .iter()
            .map(|path| fs::read(root.join("generated").join(path)).unwrap())
            .collect::<Vec<_>>(),
        baseline
    );
    let execution = std::process::Command::new("node").current_dir(root).args(["--input-type=module", "-e", r#"
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const module=await import('./generated/locales/en.js');
const messages=module.default;
assert.equal(messages.formatNumber.number({value:1234.5}),'1,234.5');
assert.equal(messages.normalizeMessageArgs.hello({name:'Name'}),'Hello Name');
assert.equal(messages.lgl.hello,'Locale');
assert.equal(messages.String.number('first'),'first');
assert.equal(messages.main.main,'Main');
assert.equal(messages.pluralEn.count(1),'One');
assert.equal(messages.pluralEn.count({value:2}),'Many');
for(const name of ['formatNumber','normalizeMessageArgs','lgl','String','main','pluralEn']) assert.equal(module[name],messages[name]);
const manifest=JSON.parse(readFileSync('./generated/bundler/manifest.json','utf8'));
for(const [name,argument,expected] of [['formatNumber.number',1234.5,'1,234.5'],['normalizeMessageArgs.hello','Name','Hello Name'],['String.number','first','first'],['pluralEn.count',2,'Many']]) {
  const {message}=await import('./generated/'+manifest.messages[name].locales.en.module);
  assert.equal(message(argument),expected);
}
console.log('verified');
"#]).output().unwrap();
    assert!(
        execution.status.success(),
        "{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap().trim(),
        "verified"
    );
    assert_cmd::cargo::cargo_bin_cmd!("linguini")
        .current_dir(root)
        .arg("generate")
        .assert()
        .success()
        .stdout(contains("=> Locale"))
        .stdout(contains("=> Main"));
}
