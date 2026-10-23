//! Native module registry. Option semantics stay in the owning generator.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use linguini_codegen::ModulePlan;
use linguini_codegen_ecmascript_web::EcmaWebOptions;

use crate::{CliError, CliResult};

const MAX_PLAN_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum Framework {
    Svelte,
    SvelteKit,
}

pub(super) struct ModuleCapabilities {
    pub web: Option<EcmaWebOptions>,
    pub framework: Option<Framework>,
}

pub(crate) fn read_module_plan(
    root: &Path,
    path: Option<&Path>,
    stdin: bool,
) -> CliResult<ModulePlan> {
    let mut source = String::new();
    if stdin {
        io::stdin()
            .take(MAX_PLAN_BYTES + 1)
            .read_to_string(&mut source)
            .map_err(|source| CliError::Io {
                path: "<module plan stdin>".into(),
                source,
            })?;
    } else {
        let selected = path
            .map(|path| root.join(path))
            .unwrap_or_else(|| root.join("linguini.modules.json"));
        let file = match File::open(&selected) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound && path.is_none() => {
                if root.join("linguini.config.mjs").exists() {
                    return Err(CliError::Diagnostics("linguini.config.mjs requires the JavaScript build frontend or an explicit --module-plan; native codegen does not evaluate JavaScript\n".to_owned()));
                }
                return Ok(ModulePlan::empty());
            }
            Err(source) => {
                return Err(CliError::Io {
                    path: selected,
                    source,
                })
            }
        };
        if path.is_none() && root.join("linguini.config.mjs").exists() {
            return Err(CliError::Diagnostics("choose one module configuration source: linguini.config.mjs or linguini.modules.json\n".to_owned()));
        }
        file.take(MAX_PLAN_BYTES + 1)
            .read_to_string(&mut source)
            .map_err(|source| CliError::Io {
                path: selected,
                source,
            })?;
    }
    if source.len() as u64 > MAX_PLAN_BYTES {
        return Err(CliError::Diagnostics(
            "module plan exceeds the 4 MiB limit\n".to_owned(),
        ));
    }
    ModulePlan::parse(&source).map_err(|error| CliError::Diagnostics(format!("{error}\n")))
}

impl ModuleCapabilities {
    pub fn compile(plan: &ModulePlan, project_name: &str) -> CliResult<Self> {
        let mut capabilities = Self {
            web: None,
            framework: None,
        };
        for module in plan.modules() {
            match module.id() {
                "ecmascript.web" => {
                    capabilities.web = Some(
                        linguini_codegen_ecmascript_web::parse_module_options(
                            module.options(),
                            project_name,
                        )
                        .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?,
                    );
                }
                "svelte" | "sveltekit" => {
                    if !module
                        .options()
                        .as_object()
                        .is_some_and(|options| options.is_empty())
                    {
                        return Err(CliError::Diagnostics(format!(
                            "module `{}` accepts no options\n",
                            module.id()
                        )));
                    }
                    if capabilities.framework.is_some() {
                        return Err(CliError::Diagnostics("SvelteKit composes Svelte; configure either the svelte or sveltekit module, not both\n".to_owned()));
                    }
                    capabilities.framework = Some(if module.id() == "svelte" {
                        Framework::Svelte
                    } else {
                        Framework::SvelteKit
                    });
                }
                id => {
                    return Err(CliError::Diagnostics(format!(
                        "unsupported codegen module `{id}`\n"
                    )))
                }
            }
        }
        if capabilities.framework == Some(Framework::SvelteKit) {
            linguini_codegen_sveltekit::validate_module_sources(capabilities.web.as_ref())
                .map_err(|error| CliError::Diagnostics(format!("{error}\n")))?;
        }
        Ok(capabilities)
    }
}
