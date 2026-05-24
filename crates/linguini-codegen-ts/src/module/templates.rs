pub const INDEX_RUNTIME: &str = include_str!("templates/index.runtime.ts");
pub const INDEX_RUNTIME_DECLARATIONS: &str = include_str!("templates/index.runtime.d.ts");
pub const PROJECT_INDEX_ENTRY: &str = include_str!("templates/project-index.entry.ts");
pub const PROJECT_INDEX_DECLARATIONS: &str = include_str!("templates/project-index.entry.d.ts");
pub const SHARED_RUNTIME: &str = include_str!("templates/shared.runtime.ts");
pub const SHARED_DECLARATIONS: &str = include_str!("templates/shared.runtime.d.ts");
pub const SINGLE_INDEX_RUNTIME: &str = include_str!("templates/single-index.runtime.ts");
pub const SINGLE_INDEX_DECLARATIONS: &str = include_str!("templates/single-index.runtime.d.ts");
pub const SVELTE_RUNTIME: &str = include_str!("templates/svelte.runtime.ts");
pub const SVELTE_DECLARATIONS: &str = include_str!("templates/svelte.runtime.d.ts");
pub const SVELTEKIT_RUNTIME: &str = include_str!("templates/sveltekit.runtime.ts");
pub const SVELTEKIT_DECLARATIONS: &str = include_str!("templates/sveltekit.runtime.d.ts");
pub const WEB_RUNTIME: &str = include_str!("templates/web.runtime.ts");
pub const WEB_DECLARATIONS: &str = include_str!("templates/web.runtime.d.ts");

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
        assert!(INDEX_RUNTIME.contains("createLinguini"));
        assert!(INDEX_RUNTIME_DECLARATIONS.contains("createLinguini"));
        assert!(PROJECT_INDEX_ENTRY.contains("{{INDEX_RUNTIME}}"));
        assert!(PROJECT_INDEX_DECLARATIONS.contains("{{INDEX_RUNTIME_DECLARATIONS}}"));
        assert!(SHARED_RUNTIME.contains("selectBranch"));
        assert!(SHARED_DECLARATIONS.contains("selectBranch"));
        assert!(SINGLE_INDEX_RUNTIME.contains("createLinguini"));
        assert!(SINGLE_INDEX_DECLARATIONS.contains("createLinguini"));
        assert!(WEB_RUNTIME.contains("createWebI18n"));
        assert!(WEB_DECLARATIONS.contains("LinguiniRequestContext"));
        assert!(SVELTE_RUNTIME.contains("createLinguiniRune"));
        assert!(SVELTE_DECLARATIONS.contains("LinguiniRune"));
        assert!(SVELTEKIT_RUNTIME.contains("createHandle"));
        assert!(SVELTEKIT_DECLARATIONS.contains("SerializedLinguiniContext"));
    }

    #[test]
    fn render_template_replaces_double_brace_tokens() {
        let rendered = render_template("before {{VALUE}} after", &[("VALUE", "ok".to_owned())]);
        assert_eq!(rendered, "before ok after");
    }
}
