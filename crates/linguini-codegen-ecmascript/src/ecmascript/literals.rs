//! Shared ECMAScript/JSON string escaping for modules, templates, and source maps.

pub(crate) fn string_literal(value: &str) -> String {
    format!("\"{}\"", escape_string(value))
}

pub(crate) fn escape_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000C}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{0000}'..='\u{001F}' | '\u{007F}' => {
                use std::fmt::Write;

                write!(output, "\\u{:04X}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            '\u{2028}' => output.push_str("\\u2028"),
            '\u{2029}' => output.push_str("\\u2029"),
            _ => output.push(character),
        }
    }
    output
}
