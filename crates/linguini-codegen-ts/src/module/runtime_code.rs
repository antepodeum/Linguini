//! Shared runtime statements with explicit target-specific type annotations.

use crate::ecmascript::EcmaScriptTarget;

#[derive(Default)]
pub(super) struct RuntimeCode {
    fragments: Vec<RuntimeFragment>,
}

enum RuntimeFragment {
    Shared(&'static str),
    Generated(String),
    Typed {
        typescript: &'static str,
        javascript: &'static str,
    },
}

impl RuntimeCode {
    pub(super) fn generated(&mut self, code: impl Into<String>) {
        self.fragments.push(RuntimeFragment::Generated(code.into()));
    }
    pub(super) fn shared(&mut self, code: &'static str) {
        self.fragments.push(RuntimeFragment::Shared(code));
    }

    pub(super) fn typed(&mut self, typescript: &'static str, javascript: &'static str) {
        self.fragments.push(RuntimeFragment::Typed {
            typescript,
            javascript,
        });
    }

    pub(super) fn render(&self, target: EcmaScriptTarget) -> String {
        self.fragments
            .iter()
            .map(|fragment| match fragment {
                RuntimeFragment::Shared(code) => *code,
                RuntimeFragment::Generated(code) => code.as_str(),
                RuntimeFragment::Typed {
                    typescript,
                    javascript,
                } => match target {
                    EcmaScriptTarget::TypeScript => *typescript,
                    EcmaScriptTarget::JavaScript => *javascript,
                },
            })
            .collect()
    }
}
