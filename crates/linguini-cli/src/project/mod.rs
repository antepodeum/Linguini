mod check;
mod codegen;
mod fixes;
mod io;
mod sources;
mod test_data;
mod util;

use linguini_config::{LocaleFile as DiscoveredLocaleFile, SchemaFile as DiscoveredSchemaFile};
use linguini_syntax::{LocaleFile as LocaleAst, SchemaFile as SchemaAst};

pub use check::check_project;
pub use codegen::build_project;
pub(crate) use fixes::fix_project;
pub use io::init_project;
pub(crate) use test_data::generate_project_data;

#[derive(Debug, Clone)]
pub(crate) struct ParsedSchemaSource {
    pub(crate) file: DiscoveredSchemaFile,
    pub(crate) source: String,
    pub(crate) ast: SchemaAst,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedLocaleSource {
    pub(crate) file: DiscoveredLocaleFile,
    pub(crate) source: String,
    pub(crate) ast: LocaleAst,
}
