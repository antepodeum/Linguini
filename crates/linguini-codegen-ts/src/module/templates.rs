pub const SHARED_RUNTIME: &str = include_str!("templates/shared.runtime.ts");
pub const SHARED_DECLARATIONS: &str = include_str!("templates/shared.runtime.d.ts");
#[cfg(test)]
pub const SINGLE_INDEX_RUNTIME: &str = include_str!("templates/single-index.runtime.ts");
#[cfg(test)]
pub const SINGLE_INDEX_DECLARATIONS: &str = include_str!("templates/single-index.runtime.d.ts");
#[cfg(test)]
pub fn render_template(template: &str, replacements: &[(&str, String)]) -> String {
    let mut output = template.to_owned();
    for (key, value) in replacements {
        output = output.replace(&format!("{{{{{}}}}}", key), value);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_runtime_templates_are_non_empty() {
        assert!(SHARED_RUNTIME.contains("selectBranch"));
        assert!(SHARED_RUNTIME.contains("normalizeMessageArgs"));
        assert!(SHARED_RUNTIME.contains("Reflect.ownKeys"));
        assert!(SHARED_RUNTIME.contains("hasOwnProperty.call(branches, key)"));
        assert!(SHARED_RUNTIME.contains("throw new Error"));
        assert!(!SHARED_RUNTIME.contains("?? \"\""));
        assert!(SHARED_DECLARATIONS.contains("selectBranch"));
        assert!(SHARED_DECLARATIONS.contains("normalizeMessageArgs"));
        assert!(SINGLE_INDEX_RUNTIME.contains("createLinguini"));
        assert!(SINGLE_INDEX_DECLARATIONS.contains("createLinguini"));
    }

    #[test]
    fn render_template_replaces_double_brace_tokens() {
        let rendered = render_template("before {{VALUE}} after", &[("VALUE", "ok".to_owned())]);
        assert_eq!(rendered, "before ok after");
    }
}
