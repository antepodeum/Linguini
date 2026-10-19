//! Server cookie persistence delegates to the shared web policy.

use crate::ecmascript::{EcmaModuleKind, EcmaModuleOutput, RenderedEcmaModule};

pub fn generate_web_server_cookie_declaration() -> String {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::Declaration, "web/server-cookie.d.ts", None),
        "web/server-cookie_declarations",
        minijinja::context! {},
    )
    .render_code()
}

/// Compile server cookie policy delegation as checked JavaScript.
pub fn compile_web_server_cookie_module() -> RenderedEcmaModule {
    crate::templates::module(
        EcmaModuleOutput::new(EcmaModuleKind::JavaScript, "web/server-cookie.js", None),
        "web/server-cookie",
        minijinja::context! {},
    )
    .render(&[])
}
