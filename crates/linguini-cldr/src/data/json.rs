use crate::cache::{CldrCacheError, CldrCacheResult};
use std::fs;
use std::path::Path;

pub fn read_data_file(path: &Path) -> CldrCacheResult<String> {
    fs::read_to_string(path).map_err(|source| CldrCacheError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn required_string(source: &str, key: &str, path: &Path) -> CldrCacheResult<String> {
    optional_string(source, key).ok_or_else(|| CldrCacheError::Parse {
        path: path.to_path_buf(),
        message: format!("missing string field `{key}`"),
    })
}

pub fn optional_string(source: &str, key: &str) -> Option<String> {
    string_pairs(source)
        .into_iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value)
}

pub fn find_json_object<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let quoted_key = format!("\"{key}\"");
    let key_start = source.find(&quoted_key)?;
    let after_key = &source[key_start + quoted_key.len()..];
    let object_start = after_key.find('{')? + key_start + quoted_key.len();
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for index in object_start..source.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[object_start + 1..index]);
                }
            }
            _ => {}
        }
    }

    None
}

pub fn string_pairs(source: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut index = 0;

    while let Some((key, after_key)) = read_json_string(source, index) {
        let Some(colon) = source[after_key..].find(':') else {
            break;
        };
        let value_start = after_key + colon + 1;
        let Some((value, after_value)) = read_json_string(source, value_start) else {
            index = value_start;
            continue;
        };
        pairs.push((key, value));
        index = after_value;
    }

    pairs
}

fn read_json_string(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut index = start;
    while index < bytes.len() && bytes[index] != b'"' {
        index += 1;
    }
    if index == bytes.len() {
        return None;
    }

    index += 1;
    let mut value = String::new();
    let mut escaped = false;
    while index < bytes.len() {
        let character = source[index..].chars().next()?;
        index += character.len_utf8();

        if escaped {
            value.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some((value, index));
        } else {
            value.push(character);
        }
    }

    None
}
