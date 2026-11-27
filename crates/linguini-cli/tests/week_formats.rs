use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn year_week_requests_check_build_preview_and_preserve_failed_output() {
    assert_week_family("yw");
}

#[test]
fn month_week_requests_check_build_preview_and_preserve_failed_output() {
    assert_week_family("MMMMW");
}

fn assert_week_family(skeleton: &str) {
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
        "nso",
        "sq",
        "sa",
        "en-GB-u-fw-sun",
        "en-u-fw-mon",
        "en-US-u-rg-gbzzzz",
        "en-US-u-rg-aczzzz",
        "en-u-sd-gbeng",
        "en-US-u-sd-gbeng",
        "en-u-sd-frxxx",
        "en-US-u-ca-iso8601",
        "en-AU-u-ca-iso8601-rg-afzzzz",
        "en-u-ca-iso8601-fw-sun",
        "en-u-rg-gbeng",
        "en-u-rg-fi01",
        "en-u-sd-fra",
        "en-u-sd-cn11",
        "en-u-ca-iso8601-rg-afbal",
        "en-u-sd-gbgbn",
        "en-u-sd-fr75",
        "nso-u-sd-gbgbn",
        "mt-u-rg-usca",
        "sa-u-sd-gbgbn",
    ];
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::create_dir_all(root.join("locales/main")).unwrap();
    fs::write(root.join("package.json"), "{\"type\":\"module\"}").unwrap();
    fs::write(root.join("linguini.toml"), format!(
        "[project]\nname=\"weeks\"\ndefault_locale=\"en-GB\"\nlocales={}\n[paths]\nschema=\"schema\"\nlocale=\"locales\"\n[targets.ts]\nout=\"generated\"\n",
        serde_json::to_string(&locales).unwrap()
    )).unwrap();
    fs::write(root.join("schema/main.lgs"), "type WeekDate = Date @date(skeleton = \"yw\")\nperiod(value: WeekDate)\nforwarded(value: Date)\n".replace("\"yw\"", &format!("{skeleton:?}"))).unwrap();
    let source = "fn Receipt(value: Date) { _ => {value @date(skeleton = \"yw\")} }\nperiod = {value}\nforwarded = {Receipt(value)}\n".replace("\"yw\"", &format!("{skeleton:?}"));
    for locale in locales {
        fs::write(root.join(format!("locales/main/{locale}.lgl")), &source).unwrap();
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
    let script = format!(
        "const skeleton = {skeleton:?};\nconst locales = {};\n",
        serde_json::to_string(&locales).unwrap()
    ) + r#"
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {prepareLinguini,createLinguini} from './generated/index.js';
const manifest=JSON.parse(readFileSync('./generated/bundler/manifest.json','utf8'));
for(const locale of locales) {
  const prepared=await prepareLinguini(locale);
  assert.equal(createLinguini(locale),prepared);
  const {main}=await import('./generated/locales/'+locale+'.js');
  const {message}=await import('./generated/'+manifest.messages['main.period'].locales[locale].module);
  for(const value of ['2021-01-01','2021-01-04','2021-05-30','2021-05-31',0,-1,new Date('0000-01-01T00:00:00Z')]) {
    const result=main.period(value);
    assert.equal(prepared.main.period(value),result);
    assert.equal(main.forwarded({value}),result);
    assert.equal(message(value),result);
  }
  if(['en-GB','en-US-u-rg-gbzzzz','en-u-sd-gbeng','en-US-u-ca-iso8601','en-u-ca-iso8601-fw-sun','en-u-rg-gbeng','en-u-rg-fi01','en-u-sd-fra','en-u-sd-gbgbn','en-u-sd-fr75'].includes(locale)) assert.equal(main.period('2021-01-01'),skeleton==='yw'?'week 53 of 2020':'week 0 of January');
  if(['en','en-US-u-sd-gbeng','en-u-sd-frxxx'].includes(locale)) {
    assert.equal(main.period('2021-01-01'),skeleton==='yw'?'week 1 of 2021':'week 1 of January');
    if(skeleton==='MMMMW') assert.equal(main.period('2021-05-30'),'week 6 of May');
  }
  if(['en-AU-u-ca-iso8601-rg-afzzzz','en-u-ca-iso8601-rg-afbal'].includes(locale)) assert.equal(main.period('2021-01-01'),skeleton==='yw'?'week 52 of 2020':'week 0 of January');
  if(locale==='en-u-sd-cn11') {
    assert.equal(main.period('2021-01-01'),skeleton==='yw'?'week 1 of 2021':'week 1 of January');
    if(skeleton==='MMMMW') assert.equal(main.period('2021-05-30'),'week 5 of May');
  }
  if(locale==='en-US-u-rg-aczzzz') assert.equal(main.period('2021-01-01'),skeleton==='yw'?'week 1 of 2021':'week 1 of January');
  if(locale==='en-GB-u-fw-sun') assert.equal(main.period('2021-01-03'),skeleton==='yw'?'week 1 of 2021':'week 1 of January');
}
console.log('ok');
"#;
    let execution = std::process::Command::new("node")
        .current_dir(root)
        .args(["--input-type=module", "-e", &script])
        .output()
        .unwrap();
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
        .stdout(contains(if skeleton == "yw" {
            "=> week 20 of 2026"
        } else {
            "=> week 2 of May"
        }))
        .stdout(contains(if skeleton == "yw" {
            "=> week 1 of 1970"
        } else {
            "=> week 1 of January"
        }));
    fs::write(
        root.join("locales/main/mt.lgl"),
        source.replace(
            &format!("skeleton = {skeleton:?}"),
            &format!("skeleton = {skeleton:?}, style = \"short\""),
        ),
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
