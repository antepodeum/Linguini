//! Historical integration scenarios composed through independent production module APIs.
pub use linguini_codegen_ecmascript::*;
#[derive(Clone, Copy)]
pub enum EcmaFramework {
    Svelte,
    SvelteKit,
}
pub struct EcmaProjectOptions {
    pub declaration: bool,
    pub gitignore: bool,
    pub tree_shaking: bool,
    pub included_messages: Vec<String>,
    pub base_locale: Option<String>,
    pub web: Option<EcmaWebOptions>,
    pub framework: Option<EcmaFramework>,
}
impl Default for EcmaProjectOptions {
    fn default() -> Self {
        let base = linguini_codegen_ecmascript::EcmaProjectOptions::default();
        Self {
            declaration: base.declaration,
            gitignore: base.gitignore,
            tree_shaking: base.tree_shaking,
            included_messages: base.included_messages,
            base_locale: base.base_locale,
            web: base.web,
            framework: None,
        }
    }
}
pub fn project_options(locale: &str) -> EcmaProjectOptions {
    EcmaProjectOptions {
        declaration: true,
        gitignore: true,
        base_locale: Some(locale.into()),
        ..Default::default()
    }
}
pub fn generate_project_files(
    schema: &linguini_ir::SchemaIr,
    locales: &[EcmaLocaleModule],
    options: &EcmaProjectOptions,
) -> Result<Vec<EcmaGeneratedFile>, EcmaCodegenError> {
    let base = linguini_codegen_ecmascript::EcmaProjectOptions {
        declaration: options.declaration,
        gitignore: options.gitignore,
        tree_shaking: options.tree_shaking,
        included_messages: options.included_messages.clone(),
        base_locale: options.base_locale.clone(),
        web: options.web.clone(),
    };
    let project = ValidatedEcmaProject::try_new(schema, locales, &base)?;
    let mut files = linguini_codegen_ecmascript::generate_project_files(&project)?;
    match options.framework {
        Some(EcmaFramework::Svelte) => {
            files.extend(linguini_codegen_svelte::generate_project_files(
                &linguini_codegen_svelte::SvelteProjectOptions {
                    web: options.web.clone(),
                    declaration: options.declaration,
                    host: Default::default(),
                },
                false,
            ))
        }
        Some(EcmaFramework::SvelteKit) => files.extend(super::generate_project_files(
            &super::SvelteKitProjectOptions {
                web: options.web.clone(),
                declaration: options.declaration,
            },
            false,
        )),
        None => {}
    }
    Ok(files)
}
