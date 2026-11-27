use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn linguini(root: &Path) -> Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("linguini");
    command.current_dir(root);
    command
}

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

#[test]
fn native_fallback_globals_keep_parent_priority_coverage_and_override_guards() {
    let project = TempDir::new().unwrap();
    let root = project.path();
    write(
        root,
        "linguini.toml",
        r#"
[project]
name = "fallback-globals"
default_locale = "en"
locales = ["en", "en-001", "en-AU", "fr"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "generated"
[targets.ts.bundler]
sources = ["src"]
locale_loading = "dynamic"
"#,
    );
    write(root, "package.json", "{\"type\":\"module\"}");
    write(root, "src/index.js", "export {};\n");
    write(
        root,
        "schema/main.lgs",
        r#"enum Fruit { apple }
type Item = Fruit
summary(fruit: Item, count: Number)
collide(render: Item, count: Number)
typed(fruit: Item)
other
"#,
    );
    let body = "{fruit.label}|{render(count)}|{suffix}";
    let collide = "collide = {render}|{render(count)}\n";
    let globals = r#"let label = BASE
let suffix = BASE_SUFFIX
enum Tone { formal, casual }
form Word(Tone) {
  formal => FORMAL
  casual => CASUAL
}
fn render(count: Number) { _ => {label}:{count} }
impl Fruit { apple {
  label = Apple
  Tone = formal
  tail = BASE_TAIL
} }
"#;
    write(root, "locales/main/en.lgl", &format!(
        "{globals}summary = EN {body}\n{collide}typed = {{Word(fruit.Tone)}}\nother = UNUSED_FALLBACK_MESSAGE\n"
    ));
    write(
        root,
        "locales/main/en-001.lgl",
        "let label = PARENT\nimpl Fruit { apple { label = ParentApple\nTone = casual } }\n",
    );
    write(
        root,
        "locales/main/en-AU.lgl",
        &format!("summary = AU {body}\n{collide}"),
    );
    let french = format!(
        r#"let label = LOCAL
form Word(Tone) {{
  formal => FR_FORMAL
  casual => FR_CASUAL
}}
summary = FR {body}
{collide}typed = {{Word(fruit.Tone)}}
"#
    );
    write(root, "locales/main/fr.lgl", &french);

    let checked = linguini(root)
        .args(["check", "--format", "json"])
        .assert()
        .success();
    let report: serde_json::Value = serde_json::from_slice(&checked.get_output().stdout).unwrap();
    assert_eq!(report["summary"]["errors"], 0);
    assert_eq!(report["summary"]["warnings"], 3);
    assert!(report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["message"].as_str().unwrap().contains(
            "locale `fr` for schema namespace `main` is missing 1 schema message: `other`"
        )));
    linguini(root)
        .args(["check", "--deny-warnings", "--format", "json"])
        .assert()
        .failure();
    linguini(root).arg("build").assert().success();
    let manifest = root.join("generated/bundler/manifest.json");
    let baseline = fs::read(&manifest).unwrap();
    linguini(root)
        .args(["build", "--deny-warnings"])
        .assert()
        .failure();
    assert_eq!(fs::read(&manifest).unwrap(), baseline);
    let execution = std::process::Command::new("node").current_dir(root)
        .args(["--input-type=module", "--eval", r#"
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
const {createLinguini,prepareLinguini}=await import('./generated/index.js');
const manifest=JSON.parse(await readFile('./generated/bundler/manifest.json','utf8'));
for (const [locale,prefix,fruit,label] of [
  ['en','EN','Apple','BASE'], ['en-001','EN','ParentApple','PARENT'],
  ['en-AU','AU','ParentApple','PARENT'], ['fr','FR','Apple','LOCAL']
]) {
  await prepareLinguini(locale);
  const paths=new Map(manifest.sources.map(source=>[source.id,source.path]));
  const expectedSources=['schema/main.lgs','locales/main/en.lgl'];
  if (locale==='en-001'||locale==='en-AU') expectedSources.push('locales/main/en-001.lgl');
  if (locale==='en-AU'||locale==='fr') expectedSources.push('locales/main/'+locale+'.lgl');
  assert.deepEqual(manifest.messages['main.summary'].locales[locale].source_ids.map(id=>paths.get(id)).sort(),expectedSources.sort());
  const ordinary=createLinguini(locale).main;
  const physical=async name=>(await import('./generated/'+manifest.messages['main.'+name].locales[locale].module)).message;
  for (const summary of [ordinary.summary,await physical('summary')]) {
    assert.equal(summary('apple',2),`${prefix} ${fruit}|${label}:2|BASE_SUFFIX`);
    assert.equal(summary({count:1,fruit:'apple'}),`${prefix} ${fruit}|${label}:1|BASE_SUFFIX`);
  }
  for (const collide of [ordinary.collide,await physical('collide')]) {
    assert.equal(collide('apple',2),`apple|${label}:2`);
    assert.equal(collide({count:1,render:'apple'}),`apple|${label}:1`);
  }
  for (const typed of [ordinary.typed,await physical('typed')]) {
    const expected=locale==='fr'?'FR_FORMAL':locale==='en'?'FORMAL':'CASUAL';
    assert.equal(typed('apple'),expected);
    assert.equal(typed({fruit:'apple'}),expected);
  }
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
    linguini(root)
        .arg("generate")
        .assert()
        .success()
        .stdout(contains("=> FR Apple|LOCAL:2|BASE_SUFFIX"))
        .stdout(contains("=> AU ParentApple|PARENT:2|BASE_SUFFIX"));

    for (invalid, expected) in [
        (
            format!("impl Fruit {{ apple {{ extra = Own }} }}\n{french}"),
            "unknown form property `label`",
        ),
        (format!("let render = BLOCKED\n{french}"), "unnamed form"),
        (
            french.replacen("render(count)", "render(fruit)", 1),
            "expects `Number`, got `Fruit`",
        ),
    ] {
        write(root, "locales/main/fr.lgl", &invalid);
        linguini(root)
            .args(["check", "--format", "json"])
            .assert()
            .failure()
            .stdout(contains(expected));
        let failed = linguini(root)
            .arg("build")
            .assert()
            .failure()
            .stderr(contains(expected));
        if expected.contains("expects `Number`") {
            let stderr = String::from_utf8_lossy(&failed.get_output().stderr);
            assert!(stderr.contains("locales/main/fr.lgl"), "{stderr}");
            assert!(stderr.contains("locales/main/en.lgl"), "{stderr}");
            assert!(!stderr.contains("related source #"), "{stderr}");
        }
        assert_eq!(fs::read(&manifest).unwrap(), baseline);
    }
}
