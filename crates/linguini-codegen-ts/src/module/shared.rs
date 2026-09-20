use linguini_ir::IrModule;

use crate::ecmascript::{EcmaModule, EcmaModuleOutput, EcmaStatement};

use super::emit::{emit_enums, emit_type_aliases};
use super::templates::{SHARED_DECLARATIONS, SHARED_RUNTIME};

pub fn generate_shared_module(
    schema: &IrModule,
    output: EcmaModuleOutput,
    declaration: bool,
) -> String {
    let mut module = EcmaModule::new(output);
    let mut types = String::new();
    emit_enums(schema, &mut types);
    emit_type_aliases(schema, &mut types);
    let has_types = !types.trim().is_empty();
    if has_types {
        module.push_statement(EcmaStatement::type_declaration(types, None));
    }
    let runtime = if declaration {
        SHARED_DECLARATIONS
    } else {
        SHARED_RUNTIME
    };
    module.push_statement(EcmaStatement::generated(if has_types {
        format!("\n{runtime}")
    } else {
        runtime.to_owned()
    }));
    module.render_code()
}
