//! Framework integration generation. The ECMAScript backend remains independent.

mod host;
mod svelte_control;
mod svelte_control_types;
mod svelte_effects;
mod svelte_facade;
mod svelte_facade_types;
mod svelte_locale;
mod templates;
#[cfg(test)]
mod test_templates;
#[cfg(test)]
use linguini_codegen_ecmascript_web::EcmaLocaleSource;
use linguini_codegen_ecmascript_web::{EcmaLinkMode, EcmaWebOptions};

pub use host::{SvelteHost, SvelteHostImport};
pub use svelte_control::compile_svelte_control_module;
pub use svelte_effects::compile_svelte_effects_module;
pub use svelte_facade::compile_svelte_module;
pub use svelte_locale::{compile_svelte_locale_module, SvelteLocaleOptions};

/// Selected Svelte artifacts and their external routing capabilities.
pub struct SvelteProjectOptions {
    pub web: Option<EcmaWebOptions>,
    pub declaration: bool,
    pub host: SvelteHost,
}

/// Render Svelte-owned artifacts; base ECMAScript/web artifacts are supplied by their owner.
pub fn generate_project_files(
    options: &SvelteProjectOptions,
    source_maps: bool,
) -> Vec<linguini_codegen_ecmascript::EcmaGeneratedFile> {
    use linguini_codegen_ecmascript::{generated_module_files, EcmaGeneratedFile};
    let mut files = Vec::new();
    if let Some(web) = &options.web {
        files.extend(generated_module_files(
            compile_svelte_effects_module(web, &options.host),
            source_maps,
        ));
        if options.declaration {
            files.push(EcmaGeneratedFile {
                path: "svelte-effects.svelte.d.ts".into(),
                contents: svelte_effects::generate_svelte_effects_declaration(),
            });
        }
        files.extend(generated_module_files(
            compile_svelte_control_module(web, &options.host),
            source_maps,
        ));
        if options.declaration {
            files.push(EcmaGeneratedFile {
                path: "svelte-control.d.ts".into(),
                contents: svelte_control_types::generate_svelte_control_declaration(&options.host),
            });
        }
    }
    let locale = SvelteLocaleOptions {
        initialize: options.web.is_some(),
        request_locale: options.host.request_locale.clone(),
    };
    files.extend(generated_module_files(
        compile_svelte_locale_module(&locale),
        source_maps,
    ));
    if options.declaration {
        files.push(EcmaGeneratedFile {
            path: "svelte-locale.svelte.d.ts".into(),
            contents: svelte_locale::generate_svelte_locale_declaration(&locale),
        });
    }
    files.extend(generated_module_files(
        compile_svelte_module(options.web.is_some()),
        source_maps,
    ));
    if options.declaration {
        files.push(EcmaGeneratedFile {
            path: "svelte.d.ts".into(),
            contents: svelte_facade_types::generate_svelte_declaration(options.web.is_some()),
        });
    }
    files
}
