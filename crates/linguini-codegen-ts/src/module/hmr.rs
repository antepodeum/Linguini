//! Shared target-specific typing for generated HMR lifecycle registrations.

use super::runtime_code::RuntimeCode;

pub(super) fn register_disposer(code: &mut RuntimeCode, disposer: &str) {
    code.typed("const hot = (import.meta as ImportMeta & {\n  hot?: { dispose(callback: () => void): void };\n}).hot;\n", "const hot = (/** @type {ImportMeta & { hot?: { dispose(callback: () => void): void } }} */ (import.meta)).hot;\n");
    code.generated(format!("hot?.dispose({disposer});\n"));
}
