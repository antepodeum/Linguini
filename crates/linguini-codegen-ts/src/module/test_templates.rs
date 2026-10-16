//! Substitution helper for frozen, test-only template oracles.

pub fn render_template(template: &str, replacements: &[(&str, String)]) -> String {
    let mut output = template.to_owned();
    for (key, value) in replacements {
        output = output.replace(&format!("{{{{{}}}}}", key), value);
    }
    output
}
