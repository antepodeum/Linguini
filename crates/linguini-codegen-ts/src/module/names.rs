pub fn function_name(name: &str) -> String {
    name.replace('.', "_")
}

pub fn safe_identifier(name: &str) -> String {
    name.replace('-', "_")
}

pub fn property_key(name: &str) -> String {
    if name
        .bytes()
        .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    {
        name.to_owned()
    } else {
        string_literal(name)
    }
}

pub fn string_literal(value: &str) -> String {
    format!("\"{}\"", escape_string(value))
}

pub fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
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
        other => other.to_owned(),
    }
}
