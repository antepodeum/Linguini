//! Declaration and JSDoc companions share the web template's static type model.

use linguini_codegen_ecmascript::{EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaStatement};

pub(super) fn append(module: &mut EcmaModule) {
    module.push_statement(EcmaStatement::documentation(
        crate::templates::render("web/type_jsdoc", minijinja::context! {}),
        None,
    ));
    module.push_statement(EcmaStatement::documentation("", None));
}

pub fn generate_web_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "web.d.ts",
        None,
    ));
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render("web/type_declaration", minijinja::context! {}),
        None,
    ));
    module.render_code()
}
