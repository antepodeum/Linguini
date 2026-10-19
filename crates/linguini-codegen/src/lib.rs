//! Backend-independent models and semantic planning for code generators.
//!
//! Target backends own syntax, identifier escaping, runtime templates, and file emission.

mod decision;
mod dependencies;
mod formatters;
mod inline;
mod paths;
mod requirements;
mod types;

pub use decision::{Decision, DecisionArm};
pub use dependencies::{
    message_dependency_closure, MessageDependencyClosure, MessageDependencyError,
};
pub use requirements::{
    formatter_requirements, module_uses_inline_functions, plural_required, FormatterRequirements,
};
pub use types::TypeModel;

pub use formatters::{
    default_type_formatters, formatter_requests, parameter_formatters, FormatterRequest,
};

pub use inline::lower_inline_functions;

pub use paths::lexical_relative_path;
