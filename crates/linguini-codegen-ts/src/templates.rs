//! Build-time rendering of shared ECMAScript source templates.

use std::sync::OnceLock;

use minijinja::{context, Environment, UndefinedBehavior, Value};

use crate::ecmascript::EcmaScriptTarget;

fn environment() -> &'static Environment<'static> {
    static ENVIRONMENT: OnceLock<Environment<'static>> = OnceLock::new();
    ENVIRONMENT.get_or_init(|| {
        let mut environment = Environment::new();
        environment.set_keep_trailing_newline(true);
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        for (name, source) in [
            ("functions", include_str!("../templates/functions.jinja")),
            ("shared", include_str!("../templates/shared.jinja")),
            (
                "shared_declarations",
                include_str!("../templates/shared_declarations.jinja"),
            ),
        ] {
            environment
                .add_template(name, source)
                .expect("valid embedded codegen template");
        }
        environment
    })
}

pub(crate) fn render(name: &str, target: EcmaScriptTarget, values: Value) -> String {
    environment()
        .get_template(name)
        .expect("registered codegen template")
        .render(context! { ts => target.is_typescript(), ..values })
        .expect("complete validated codegen template context")
}
