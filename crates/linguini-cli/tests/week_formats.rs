use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn year_week_requests_check_build_preview_and_preserve_failed_output() {
    let project = TempDir::new().unwrap();
    let root = project.path();
    let locales = [
        "en-GB",
        "en",
        "fo",
        "fa",
        "ar-EG",
        "ff-Adlm",
        "mt",
        "fil",
        "hy",
        "en-GB-u-fw-sun",
        "en-u-fw-mon",
    ];
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::create_dir_all(root.join("locales/main")).unwrap();
    fs::write(root.join("package.json"), "{\"type\":\"module\"}").unwrap();
    fs::write(root.join("linguini.toml"), format!(
        "[project]\nname=\"weeks\"\ndefault_locale=\"en-GB\"\nlocales={}\n[paths]\nschema=\"schema\"\nlocale=\"locales\"\n[targets.ts]\nout=\"generated\"\n",
        serde_json::to_string(&locales).unwrap()
    )).unwrap();
    fs::write(root.join("schema/main.lgs"), "type WeekDate = Date @date(skeleton = \"yw\")\nperiod(value: WeekDate)\nforwarded(value: Date)\n").unwrap();
    let source = "fn Receipt(value: Date) { _ => {value @date(skeleton = \"yw\")} }\nperiod = {value}\nforwarded = {Receipt(value)}\n";
    for locale in locales {
        fs::write(root.join(format!("locales/main/{locale}.lgl")), source).unwrap();
    }
    for action in ["check", "build"] {
        assert_cmd::cargo::cargo_bin_cmd!("linguini")
            .current_dir(root)
            .arg(action)
            .assert()
            .success();
    }
    let paths = [
        "generated/index.js",
        "generated/bundler/manifest.json",
        "generated/locales/en-GB/_runtime.js",
        "generated/locales/mt/_runtime.js",
    ];
    let baseline = paths.map(|path| fs::read(root.join(path)).unwrap());
    assert_cmd::cargo::cargo_bin_cmd!("linguini")
        .current_dir(root)
        .arg("build")
        .assert()
        .success();
    assert_eq!(
        paths.map(|path| fs::read(root.join(path)).unwrap()),
        baseline
    );
    let execution = std::process::Command::new("node").current_dir(root)
        .args(["--input-type=module", "-e", r#"
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const manifest=JSON.parse(readFileSync('./generated/bundler/manifest.json','utf8'));
for(const locale of ['en-GB','en','fo','fa','ar-EG','ff-Adlm','mt','fil','hy','en-GB-u-fw-sun','en-u-fw-mon']) {
  const {main}=await import('./generated/locales/'+locale+'.js');
  const {message}=await import('./generated/'+manifest.messages['main.period'].locales[locale].module);
  for(const value of ['2021-01-01','2021-01-04',0,-1,new Date('0000-01-01T00:00:00Z')]) {
    const result=main.period(value);
    assert.equal(main.forwarded({value}),result);
    assert.equal(message(value),result);
  }
  if(locale==='en-GB') assert.equal(main.period('2021-01-01'),'week 53 of 2020');
  if(locale==='en') assert.equal(main.period('2021-01-01'),'week 1 of 2021');
  if(locale==='en-GB-u-fw-sun') assert.equal(main.period('2021-01-03'),'week 1 of 2021');
}
console.log('ok');
"#]).output().unwrap();
    assert!(
        execution.status.success(),
        "{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), "ok");
    assert_cmd::cargo::cargo_bin_cmd!("linguini")
        .current_dir(root)
        .arg("generate")
        .assert()
        .success()
        .stdout(contains("=> week 20 of 2026"))
        .stdout(contains("=> week 1 of 1970"));
    fs::write(
        root.join("locales/main/mt.lgl"),
        source.replace("skeleton = \"yw\"", "skeleton = \"yw\", style = \"short\""),
    )
    .unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("linguini")
        .current_dir(root)
        .arg("build")
        .assert()
        .failure()
        .stderr(contains("cannot be combined"));
    assert_eq!(
        paths.map(|path| fs::read(root.join(path)).unwrap()),
        baseline
    );
}
