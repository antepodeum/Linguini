use super::templates::SHARED_RUNTIME;

pub fn emit_shared(output: &mut String) {
    output.push_str(SHARED_RUNTIME);
}
