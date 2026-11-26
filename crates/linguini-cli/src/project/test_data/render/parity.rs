//! Shared native-preview comparisons against modules built by the real CLI.

use std::io::Write;
use std::process::{Command, Stdio};

pub(super) fn assert_generated_samples(
    schema: &str,
    source: &str,
    locales: &[&str],
    cases: &[serde_json::Value],
) {
    assert_generated_samples_in_time_zones(schema, source, locales, cases, &[]);
}

pub(super) fn assert_generated_samples_in_time_zones(
    schema: &str,
    source: &str,
    locales: &[&str],
    cases: &[serde_json::Value],
    time_zones: &[&str],
) {
    let project = linguini_test_support::temp_project_dir("preview-parity").unwrap();
    let root = project.path();
    std::fs::create_dir_all(root.join("schema")).unwrap();
    std::fs::create_dir_all(root.join("locales/main")).unwrap();
    std::fs::write(root.join("package.json"), "{\"type\":\"module\"}").unwrap();
    std::fs::write(root.join("linguini.toml"), format!(
        "[project]\nname = \"preview\"\ndefault_locale = \"en\"\nlocales = {}\n[paths]\nschema = \"schema\"\nlocale = \"locales\"\n[targets.ts]\nout = \"generated\"\n",
        serde_json::to_string(locales).unwrap()
    )).unwrap();
    std::fs::write(root.join("schema/main.lgs"), schema).unwrap();
    for locale in locales {
        std::fs::write(root.join(format!("locales/main/{locale}.lgl")), source).unwrap();
    }
    crate::build_project(root).unwrap();
    let script = format!(
        r#"
import assert from 'node:assert/strict';
import {{join}} from 'node:path';
import {{pathToFileURL}} from 'node:url';
Intl.DateTimeFormat=class{{constructor(){{throw Error('no host date formatter');}}}};
Intl.NumberFormat=class{{constructor(){{throw Error('no host number formatter');}}}};
const modules=new Map();
for(const entry of {cases}) {{
  let main=modules.get(entry.locale);
  if(!main) {{
    ({{main}}=await import(pathToFileURL(join(process.argv[2],'locales',entry.locale+'.js')).href));
    modules.set(entry.locale,main);
  }}
  const args=entry.args ?? [entry.value];
  if(entry.invalid) {{
    assert.throws(()=>main['sample'+entry.index](...args),RangeError,JSON.stringify(entry));
    continue;
  }}
  assert.equal(main['sample'+entry.index](...args),entry.expected,`${{entry.locale}}/${{entry.index}}/${{JSON.stringify(args)}}`);
}}
console.log('ok');
"#,
        cases = serde_json::to_string(cases).unwrap()
    );
    let zones = if time_zones.is_empty() {
        vec![None]
    } else {
        time_zones.iter().copied().map(Some).collect()
    };
    for zone in zones {
        let mut command = Command::new("node");
        if let Some(zone) = zone {
            command.env("TZ", zone);
        }
        let mut child = command
            .args(["--input-type=module", "-"])
            .arg(root.join("generated"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "ok");
    }
}
