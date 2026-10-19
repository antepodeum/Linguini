//! Portable project-relative paths for generated artifact ownership and provenance.

/// Compute a lexical relative path between validated project-relative paths.
pub fn lexical_relative_path(from_directory: &str, target: &str) -> Result<String, &'static str> {
    let from = portable_relative_components(from_directory)?;
    let target = portable_relative_components(target)?;
    let common = from
        .iter()
        .zip(&target)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = vec![".."; from.len() - common];
    relative.extend(target[common..].iter().copied());
    if relative.is_empty() {
        return Err("source path resolves to the map directory");
    }
    Ok(relative.join("/"))
}

fn portable_relative_components(value: &str) -> Result<Vec<&str>, &'static str> {
    let has_windows_prefix = value
        .as_bytes()
        .get(1)
        .is_some_and(|character| *character == b':');
    if value.is_empty() || value.starts_with('/') || value.contains('\\') || has_windows_prefix {
        return Err("path must be non-empty, project-relative, and use `/` separators");
    }
    let components = value
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect::<Vec<_>>();
    if components.is_empty() || components.contains(&"..") {
        return Err("path must not contain parent traversal or resolve to the project root");
    }
    Ok(components)
}
