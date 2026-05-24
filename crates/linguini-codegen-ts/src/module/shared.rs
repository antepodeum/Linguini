use linguini_ir::IrModule;

use super::emit::{emit_enums, emit_type_aliases};
use super::templates::SHARED_RUNTIME;

pub fn emit_shared(schema: &IrModule, output: &mut String) {
    emit_enums(schema, output);
    emit_type_aliases(schema, output);
    output.push_str(SHARED_RUNTIME);
}
