pub fn function_name(name: &str) -> String {
    safe_identifier(name)
}

pub fn safe_identifier(name: &str) -> String {
    if is_safe_identifier(name) && !name.starts_with("__lgl_") {
        return name.to_owned();
    }

    encoded_name("__lgl_name_", name)
}

pub fn safe_file_stem(name: &str) -> String {
    if portable_path_component_error(name).is_none() && !name.starts_with("__lgl_path_") {
        return name.to_owned();
    }

    encoded_name("__lgl_path_", name)
}

pub fn portable_path_component_error(name: &str) -> Option<&'static str> {
    if name.is_empty() {
        return Some("component is empty");
    }
    if name.len() > 240 {
        return Some("component exceeds the portable 240-byte limit");
    }
    if matches!(name, "." | "..") {
        return Some("component is a traversal segment");
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Some("component ends with a dot or space");
    }
    if name.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
        return Some(
            "component contains a separator, control character, or platform-invalid character",
        );
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Some("component contains non-portable characters");
    }
    if is_windows_reserved_stem(name) {
        return Some("component is a reserved Windows device name");
    }
    None
}

fn encoded_name(prefix: &str, name: &str) -> String {
    let mut output = String::from(prefix);
    for byte in name.as_bytes() {
        use std::fmt::Write;

        write!(output, "{byte:02X}").expect("writing to a String cannot fail");
    }
    output
}

pub fn form_binding_name(name: &str) -> String {
    encoded_name("__lgl_form_", name)
}

pub fn property_access(name: &str) -> String {
    if is_safe_identifier(name) {
        format!(".{name}")
    } else {
        format!("[{}]", string_literal(name))
    }
}

pub fn path_expression(path: &[String]) -> String {
    let Some((root, properties)) = path.split_first() else {
        return String::new();
    };
    let mut output = safe_identifier(root);
    for property in properties {
        output.push_str(&property_access(property));
    }
    output
}

pub fn property_key(name: &str) -> String {
    if is_safe_identifier(name) {
        name.to_owned()
    } else {
        string_literal(name)
    }
}

pub fn string_literal(value: &str) -> String {
    format!("\"{}\"", escape_string(value))
}

pub fn escape_string(value: &str) -> String {
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

pub fn escape_comment(value: &str) -> String {
    value.replace("*/", "* /")
}

pub fn ts_type(name: &str) -> String {
    match name {
        "String" => "string".to_owned(),
        "Date" => "Date | number | string".to_owned(),
        "Number" | "Decimal" => "number".to_owned(),
        "Boolean" => "boolean".to_owned(),
        other => safe_identifier(other),
    }
}

fn is_safe_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    matches!(first, '_' | '$' | 'a'..='z' | 'A'..='Z')
        && characters
            .all(|character| matches!(character, '_' | '$' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
        && !is_reserved_word(name)
}

fn is_windows_reserved_stem(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name);
    matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn is_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "arguments"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "eval"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        escape_string, form_binding_name, function_name, path_expression,
        portable_path_component_error, property_access, property_key, safe_file_stem,
        safe_identifier, string_literal,
    };

    #[test]
    fn identifiers_are_valid_ascii_injective_and_avoid_reserved_words() {
        assert_eq!(safe_identifier("message"), "message");
        assert_eq!(
            safe_identifier("message-name"),
            "__lgl_name_6D6573736167652D6E616D65"
        );
        assert_eq!(safe_identifier("9lives"), "__lgl_name_396C69766573");
        assert_eq!(safe_identifier("class"), "__lgl_name_636C617373");
        assert_eq!(
            safe_identifier("has space!"),
            "__lgl_name_68617320737061636521"
        );
        assert_eq!(safe_identifier("日本語"), "__lgl_name_E697A5E69CACE8AA9E");
        assert_eq!(safe_identifier("\u{0000}"), "__lgl_name_00");
        assert_eq!(safe_identifier(""), "__lgl_name_");
        assert_eq!(
            function_name("account.delete"),
            "__lgl_name_6163636F756E742E64656C657465"
        );

        assert_ne!(
            safe_identifier("message-name"),
            safe_identifier("message_name")
        );
        assert_ne!(safe_identifier("9lives"), safe_identifier("_9lives"));
        assert_ne!(
            safe_identifier("class"),
            safe_identifier("__lgl_name_636C617373")
        );
        assert_ne!(
            form_binding_name("Fruit"),
            safe_identifier("__lgl_form_4672756974")
        );
    }

    #[test]
    fn file_stems_are_portable_injective_and_avoid_windows_devices() {
        assert_eq!(safe_file_stem("checkout"), "checkout");
        assert_eq!(safe_file_stem("en-US"), "en-US");
        assert_eq!(safe_file_stem("../escape"), "__lgl_path_2E2E2F657363617065");
        assert_eq!(safe_file_stem("CON"), "__lgl_path_434F4E");
        assert_eq!(safe_file_stem("日本語"), "__lgl_path_E697A5E69CACE8AA9E");
        assert_eq!(
            safe_file_stem("__lgl_path_434F4E"),
            "__lgl_path_5F5F6C676C5F706174685F343334463445"
        );
        assert_ne!(safe_file_stem("CON"), safe_file_stem("__lgl_path_434F4E"));
    }

    #[test]
    fn path_component_validation_rejects_cross_platform_hazards() {
        for component in [
            "",
            ".",
            "..",
            "../en",
            "en/us",
            r"en\us",
            "name.",
            "name ",
            "a:b",
            "NUL",
            "COM9",
            "日本語",
            "has space",
        ] {
            assert!(
                portable_path_component_error(component).is_some(),
                "{component:?} must be rejected"
            );
        }
        for component in ["en", "en-US", "zh-Hant-TW", "locale_1"] {
            assert_eq!(
                portable_path_component_error(component),
                None,
                "{component}"
            );
        }
        assert_eq!(
            portable_path_component_error(&"a".repeat(241)),
            Some("component exceeds the portable 240-byte limit")
        );
    }

    #[test]
    fn expression_paths_use_safe_bindings_and_property_access() {
        assert_eq!(property_access("label"), ".label");
        assert_eq!(property_access("delete"), "[\"delete\"]");
        assert_eq!(
            path_expression(&["class".to_owned(), "delete".to_owned()]),
            "__lgl_name_636C617373[\"delete\"]"
        );
    }

    #[test]
    fn unsafe_property_names_are_quoted_and_escaped() {
        assert_eq!(property_key("message"), "message");
        assert_eq!(property_key("$message_2"), "$message_2");
        assert_eq!(property_key("class"), "\"class\"");
        assert_eq!(property_key("2fa"), "\"2fa\"");
        assert_eq!(property_key("with-dash"), "\"with-dash\"");
        assert_eq!(property_key("日本語"), "\"日本語\"");
        assert_eq!(property_key("line\nbreak"), "\"line\\nbreak\"");
    }

    #[test]
    fn strings_escape_javascript_line_breaks_and_controls() {
        let value = "\"\\\u{0000}\u{0008}\t\n\u{000B}\u{000C}\r\u{001F}\u{007F}\u{2028}\u{2029}";
        assert_eq!(
            escape_string(value),
            "\\\"\\\\\\u0000\\b\\t\\n\\u000B\\f\\r\\u001F\\u007F\\u2028\\u2029"
        );
        assert_eq!(
            string_literal(value),
            format!("\"{}\"", escape_string(value))
        );
    }
}
