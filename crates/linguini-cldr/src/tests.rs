use super::{built_in_plural_rules, parse_plural_rule};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn crate_exports_plural_parser() {
    assert!(parse_plural_rule("i = 1 and v = 0").is_ok());
}

#[test]
fn pinned_cldr_cardinals_match_icu_intl_plural_rules() {
    let locales = ["en", "fr", "ru", "ar", "pl", "cs", "sl", "cy"];
    let samples = [
        "0", "1", "2", "3", "4", "5", "10", "11", "12", "20", "21", "22", "23", "100", "101",
        "1.1", "1.2", "2.1", "5.5",
    ];
    let script = format!(
        r#"
const locales = {locales:?};
const samples = {samples:?};
for (const locale of locales) {{
  const rules = new Intl.PluralRules(locale, {{ type: "cardinal" }});
  for (const sample of samples) console.log(`${{locale}}\t${{sample}}\t${{rules.select(Number(sample))}}`);
}}
"#
    );
    let mut child = Command::new("node")
        .args(["--input-type=module", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node.js with ICU is required for plural differential tests");
    child
        .stdin
        .take()
        .expect("Node stdin")
        .write_all(script.as_bytes())
        .expect("write ICU differential script");
    let output = child.wait_with_output().expect("wait for Node.js");
    assert!(
        output.status.success(),
        "ICU differential process failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut compared = 0;
    for line in String::from_utf8(output.stdout)
        .expect("ICU output is UTF-8")
        .lines()
    {
        let mut fields = line.split('\t');
        let locale = fields.next().expect("locale field");
        let sample = fields.next().expect("sample field");
        let expected = fields.next().expect("category field");
        assert!(fields.next().is_none(), "unexpected ICU output: {line}");
        let rules = built_in_plural_rules(locale).expect("compiled locale plural rules");
        assert_eq!(
            rules.category_for(sample).expect("valid plural sample"),
            expected,
            "ICU/CLDR mismatch for {locale} sample {sample}"
        );
        compared += 1;
    }
    assert_eq!(compared, locales.len() * samples.len());
}
