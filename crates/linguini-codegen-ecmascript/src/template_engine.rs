//! Shared ECMAScript template rendering for independently packaged integrations.

use minijinja::{Environment, UndefinedBehavior, Value};

/// An immutable, reusable renderer for one codegen module's embedded templates.
/// ECMAScript literal escaping and strict context validation are shared by all modules.
pub struct EcmaTemplateEngine {
    environment: Environment<'static>,
}
impl EcmaTemplateEngine {
    /// Register a complete static template set, including any included templates.
    /// Panics for invalid embedded templates, which are implementation errors.
    pub fn new(sources: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        let mut environment = Environment::new();
        environment.set_keep_trailing_newline(true);
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment.add_filter("js", crate::ecmascript::string_literal);
        for (name, source) in sources {
            environment
                .add_template(name, source)
                .expect("valid embedded codegen template");
        }
        Self { environment }
    }
    /// Render a registered template from validated semantic context.
    /// Panics for missing templates or context, which are implementation errors.
    pub fn render(&self, name: &str, values: Value) -> String {
        self.environment
            .get_template(name)
            .expect("registered codegen template")
            .render(values)
            .expect("complete validated codegen template context")
    }
}
