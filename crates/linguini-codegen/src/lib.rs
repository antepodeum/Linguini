//! Backend-independent models and semantic planning for code generators.
//!
//! Target backends own syntax, identifier escaping, runtime templates, and file emission.

mod dependencies;
mod requirements;
mod types;

pub use dependencies::{
    message_dependency_closure, MessageDependencyClosure, MessageDependencyError,
};
pub use requirements::{
    formatter_requirements, module_uses_inline_functions, plural_required, FormatterRequirements,
};
pub use types::TypeModel;
