//! Shared native-preview comparisons against modules built by the real CLI.

use std::io::Write;
use std::process::{Command, Stdio};

pub(super) fn assert_generated_samples(
    schema: &str,
    source: &str,
    locales: &[&str],
    cases: &[serde_json::Value],
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
  assert.equal(main['sample'+entry.index](entry.value),entry.expected,`${{entry.locale}}/${{entry.index}}/${{entry.value}}`);
}}
console.log('ok');
"#,
        cases = serde_json::to_string(cases).unwrap()
    );
    let mut child = Command::new("node")
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
