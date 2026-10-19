//! Reusable Vite artifact planning. Framework adapters supply runtime descriptors explicitly.
use linguini_codegen_ecmascript::{
    compile_bundler_message_artifact_module, compile_bundler_semantic_module, rebase_map_sources,
    EcmaCodegenError, EcmaGeneratedFile, EcmaLocaleRuntimeArtifact, EcmaMessageArtifact,
    EcmaSemanticArtifact, EcmaSource, ValidatedEcmaProject,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct ViteCodegenError(String);
impl std::fmt::Display for ViteCodegenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl std::error::Error for ViteCodegenError {}
impl From<EcmaCodegenError> for ViteCodegenError {
    fn from(error: EcmaCodegenError) -> Self {
        Self(error.to_string())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LocaleLoading {
    Eager,
    Dynamic,
}
impl LocaleLoading {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eager => "eager",
            Self::Dynamic => "dynamic",
        }
    }
}

/// Validated application usage and runtime capabilities supplied by the host compiler/adapters.
pub struct ViteApplicationPlan {
    pub locale_loading: LocaleLoading,
    pub applications: BTreeMap<String, serde_json::Value>,
    pub runtime_helpers: BTreeMap<String, serde_json::Value>,
}
/// One output destination. The semantic artifact plan is reusable across destinations.
pub struct ViteEmissionOptions<'a> {
    pub output_root: &'a str,
    pub sources: &'a [EcmaSource],
    pub configured_locales: &'a [String],
    pub base_locale: &'a str,
    pub applications: Option<&'a ViteApplicationPlan>,
}
/// Exact message, helper, and semantic leaves owned by the reusable bundler integration.
pub struct VitePlan<'project, 'schema> {
    project: &'project ValidatedEcmaProject<'schema>,
    messages: Vec<EcmaMessageArtifact>,
    runtimes: Vec<EcmaLocaleRuntimeArtifact>,
    semantics: Vec<EcmaSemanticArtifact>,
}
impl<'project, 'schema> VitePlan<'project, 'schema> {
    pub fn new(project: &'project ValidatedEcmaProject<'schema>) -> Result<Self, ViteCodegenError> {
        Ok(Self {
            project,
            messages: project.message_artifacts()?,
            runtimes: project.locale_runtime_artifacts()?,
            semantics: project.semantic_artifacts()?,
        })
    }
    pub fn message_arities(&self) -> BTreeMap<String, usize> {
        self.messages
            .iter()
            .map(|artifact| (artifact.message.clone(), artifact.arity))
            .collect()
    }
    pub fn generate_files(
        &self,
        options: &ViteEmissionOptions<'_>,
    ) -> Result<Vec<EcmaGeneratedFile>, ViteCodegenError> {
        let ViteEmissionOptions {
            output_root,
            sources,
            configured_locales,
            base_locale,
            ..
        } = options;
        let mut files = Vec::with_capacity((self.messages.len() + self.semantics.len()) * 2 + 1);
        let mut messages = BTreeMap::<String, (usize, BTreeMap<String, serde_json::Value>)>::new();
        let effective_locales = self
            .project
            .effective_locales()
            .into_iter()
            .collect::<BTreeSet<_>>();

        for artifact in &self.semantics {
            let map_sources = rebase_map_sources(output_root, &artifact.source_map_path, sources)
            .map_err(|reason| {
                ViteCodegenError(format!(
                    "bundler semantic `{}` `{}` locale `{}`: cannot rebase source-map paths: {reason}",
                    artifact.kind.as_str(), artifact.name, artifact.locale
                ))
            })?;
            let compiled = compile_bundler_semantic_module(self.project, artifact, &map_sources)
                .map_err(|error| {
                    ViteCodegenError(format!(
                        "bundler semantic `{}` `{}` locale `{}`: {error}",
                        artifact.kind.as_str(),
                        artifact.name,
                        artifact.locale
                    ))
                })?;
            files.push(EcmaGeneratedFile {
                path: artifact.module_path.clone(),
                contents: compiled.code,
            });
            files.push(EcmaGeneratedFile {
                path: artifact.source_map_path.clone(),
                contents: compiled.source_map,
            });
        }

        for artifact in &self.messages {
            let map_sources = rebase_map_sources(output_root, &artifact.source_map_path, sources)
                .map_err(|reason| {
                ViteCodegenError(format!(
                    "bundler message `{}` locale `{}`: cannot rebase source-map paths: {reason}",
                    artifact.message, artifact.locale
                ))
            })?;
            let compiled =
                compile_bundler_message_artifact_module(self.project, artifact, &map_sources)
                    .map_err(|error| {
                        ViteCodegenError(format!(
                            "bundler message `{}` locale `{}`: {error}",
                            artifact.message, artifact.locale
                        ))
                    })?;
            let locale_entry = serde_json::json!({
                "module": artifact.module_path,
                "source_ids": compiled.source_ids().iter().map(|id| id.0).collect::<Vec<_>>(),
            });
            messages
                .entry(artifact.message.clone())
                .or_insert_with(|| (artifact.arity, BTreeMap::new()))
                .1
                .insert(artifact.locale.clone(), locale_entry);
            files.push(EcmaGeneratedFile {
                path: artifact.module_path.clone(),
                contents: compiled.code,
            });
            files.push(EcmaGeneratedFile {
                path: artifact.source_map_path.clone(),
                contents: compiled.source_map,
            });
        }

        let messages = messages
            .into_iter()
            .map(|(message, (arity, locales))| {
                (
                    message,
                    serde_json::json!({
                        "arity": arity,
                        "locales": locales,
                    }),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let source_table = sources
            .iter()
            .map(|source| {
                serde_json::json!({
                    "id": source.id.0,
                    "path": source.path,
                })
            })
            .collect::<Vec<_>>();
        let mut manifest = serde_json::json!({
            "version": 1,
            "base_locale": base_locale,
            "configured_locales": configured_locales,
            "effective_locales": effective_locales,
            "sources": source_table,
            "messages": messages,
        });
        if let Some(application_plan) = options.applications {
            let manifest = manifest.as_object_mut().expect("manifest is an object");
            manifest.insert("version".to_owned(), serde_json::json!(1));
            manifest.insert(
                "locale_loading".to_owned(),
                serde_json::json!(application_plan.locale_loading.as_str()),
            );
            manifest.insert(
                "applications".to_owned(),
                serde_json::json!(application_plan.applications),
            );
            let message_runtimes = self.runtimes
            .iter()
            .map(|artifact| {
                (
                    artifact.locale.clone(),
                    serde_json::json!({
                        "module": artifact.module_path,
                        "source_ids": artifact.source_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                    }),
                )
            })
            .collect::<BTreeMap<_, _>>();
            manifest.insert(
                "message_runtimes".to_owned(),
                serde_json::json!(message_runtimes),
            );
            let message_semantics = self
                .semantics
                .iter()
                .map(|artifact| {
                    serde_json::json!({
                        "locale": artifact.locale,
                        "kind": artifact.kind.as_str(),
                        "name": artifact.name,
                        "module": artifact.module_path,
                        "source_ids": artifact.source_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                    })
                })
                .collect::<Vec<_>>();
            manifest.insert(
                "message_semantics".to_owned(),
                serde_json::Value::Array(message_semantics),
            );
            manifest.insert(
                "runtime_helpers".to_owned(),
                serde_json::json!(application_plan.runtime_helpers),
            );
        }
        let mut contents = serde_json::to_string_pretty(&manifest).map_err(|error| {
            ViteCodegenError(format!("failed to serialize bundler manifest: {error}"))
        })?;
        contents.push('\n');
        files.push(EcmaGeneratedFile {
            path: "bundler/manifest.json".to_owned(),
            contents,
        });
        Ok(files)
    }
}
