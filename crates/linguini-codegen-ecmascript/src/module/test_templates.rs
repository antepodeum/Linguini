//! Substitution helper for frozen, test-only template oracles.

/// Retain frozen declaration oracles, applying only the explicit Node ESM import fix.
pub(super) fn declaration_with_runtime_imports(source: impl AsRef<str>) -> String {
    let mut output = source.as_ref().to_owned();
    for path in ["./index", "./locale", "./web", "../web"] {
        output = output.replace(
            &format!("from \"{path}\";"),
            &format!("from \"{path}.js\";"),
        );
    }
    output
}
