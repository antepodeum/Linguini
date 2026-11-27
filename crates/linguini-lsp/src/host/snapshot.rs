//! Deterministic snapshots and source budgets shared by native and memory hosts.
use super::*;
use crate::document::prime_semantic_cache;
use linguini_syntax::SourceId;
use std::collections::HashSet;

pub(super) trait WorkspaceSources {
    fn schema_files(&self, root: &Path) -> Result<Vec<linguini_config::SchemaFile>, String>;
    fn locale_files(&self, root: &Path) -> Result<Vec<linguini_config::LocaleFile>, String>;
    fn identity(&self, path: &Path) -> PathBuf;
    fn read(&self, path: &Path) -> Result<String, String>;
}

pub(super) fn index_contexts(
    mut contexts: Vec<ProjectContext>,
    errors: Vec<String>,
    sources: &impl WorkspaceSources,
) -> WorkspaceIndex {
    let mut index = WorkspaceIndex {
        projects: Vec::new(),
        errors,
    };
    contexts.sort_by(|left, right| {
        right
            .root
            .components()
            .count()
            .cmp(&left.root.components().count())
            .then_with(|| left.root.cmp(&right.root))
    });
    if contexts.len() > MAX_WORKSPACE_PROJECTS {
        push_index_error(
            &mut index.errors,
            format!(
                "workspace exposes {} Linguini projects; indexing first {MAX_WORKSPACE_PROJECTS}",
                contexts.len()
            ),
        );
        contexts.truncate(MAX_WORKSPACE_PROJECTS);
    }
    let mut next_source_id = 1u32;
    let mut remaining_files = MAX_INDEXED_FILES;
    let mut remaining_bytes = MAX_INDEXED_BYTES;
    let mut seen_sources = HashSet::new();
    for context in contexts {
        if remaining_files == 0 || remaining_bytes == 0 {
            push_index_error(
                &mut index.errors,
                format!(
                    "workspace index reached its global limit of {MAX_INDEXED_FILES} files or {MAX_INDEXED_BYTES} bytes"
                ),
            );
            break;
        }
        index.projects.push(load_project_snapshot(
            context,
            sources,
            &mut index.errors,
            &mut next_source_id,
            &mut remaining_files,
            &mut remaining_bytes,
            &mut seen_sources,
        ));
    }
    index
        .projects
        .sort_by(|left, right| left.context.root.cmp(&right.context.root));
    index
}

fn load_project_snapshot(
    context: ProjectContext,
    provider: &impl WorkspaceSources,
    errors: &mut Vec<String>,
    next_source_id: &mut u32,
    remaining_files: &mut usize,
    remaining_bytes: &mut u64,
    seen_sources: &mut HashSet<PathBuf>,
) -> ProjectSnapshot {
    let mut sources = Vec::new();
    match provider.schema_files(&context.schema_root) {
        Ok(files) => {
            sources.extend(
                files
                    .into_iter()
                    .map(|file| (file.path, file.namespace, None, "linguini-schema")),
            );
        }
        Err(error) => push_index_error(
            errors,
            format!(
                "cannot discover schema sources for {}: {error}",
                context.root.display()
            ),
        ),
    }
    match provider.locale_files(&context.locale_root) {
        Ok(files) => {
            sources.extend(files.into_iter().map(|file| {
                (
                    file.path,
                    file.namespace,
                    Some(file.locale),
                    "linguini-locale",
                )
            }));
        }
        Err(error) => push_index_error(
            errors,
            format!(
                "cannot discover locale sources for {}: {error}",
                context.root.display()
            ),
        ),
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    sources.dedup_by(|left, right| left.0 == right.0);
    let file_limit = MAX_PROJECT_FILES.min(*remaining_files);
    if sources.len() > file_limit {
        push_index_error(
            errors,
            format!(
                "project {} exceeds its remaining index budget of {file_limit} source files",
                context.root.display(),
            ),
        );
        sources.truncate(file_limit);
    }

    let mut bytes = 0u64;
    let mut documents = Vec::new();
    let policy = std::sync::Arc::clone(&context.locale_policy);
    let mut locale_plan = None;
    let mut locale_fallbacks = std::collections::BTreeMap::new();
    for (path, namespace, locale, language_id) in sources {
        let normalized_path = provider.identity(&path);
        if !seen_sources.insert(normalized_path) {
            continue;
        }
        let source = match provider.read(&path) {
            Ok(source) => source,
            Err(error) => {
                push_index_error(errors, error);
                continue;
            }
        };
        if source.len() as u64 > MAX_DOCUMENT_BYTES {
            push_index_error(
                errors,
                format!(
                    "source {} grew beyond {} bytes while indexing",
                    path.display(),
                    MAX_DOCUMENT_BYTES
                ),
            );
            continue;
        }
        let source_bytes = source.len() as u64;
        if source_bytes > MAX_PROJECT_BYTES.saturating_sub(bytes) {
            push_index_error(
                errors,
                format!(
                    "project {} exceeds {} indexed source bytes",
                    context.root.display(),
                    MAX_PROJECT_BYTES
                ),
            );
            break;
        }
        if source_bytes > *remaining_bytes {
            push_index_error(
                errors,
                format!(
                    "workspace exceeds {MAX_INDEXED_BYTES} indexed source bytes while loading {}",
                    path.display()
                ),
            );
            break;
        }
        let Some(uri) = path_to_uri(&path) else {
            push_index_error(
                errors,
                format!("cannot convert source path to URI: {}", path.display()),
            );
            continue;
        };
        let source_id = SourceId(*next_source_id);
        *next_source_id = next_source_id.saturating_add(1);
        // Resolve only indexed locale names and reuse the chain across their namespaces.
        let fallbacks = locale
            .as_ref()
            .map(|locale| {
                locale_fallbacks
                    .entry(locale.clone())
                    .or_insert_with(|| {
                        if !policy.locales.contains(locale) {
                            return Vec::new();
                        }
                        let plan = locale_plan.get_or_insert_with(|| {
                            linguini_cldr::ProjectLocalePlan::new(
                                policy.locales.iter().map(String::as_str),
                            )
                        });
                        plan.fallback_chain(locale, Some(&policy.default_locale))
                            .into_iter()
                            .filter(|fallback| !fallback.eq_ignore_ascii_case(locale))
                            .collect()
                    })
                    .clone()
            })
            .unwrap_or_default();
        let document = LinguiniDocument::new(uri.to_string(), language_id, source)
            .with_source_identity(namespace, locale)
            .with_locale_fallbacks(fallbacks)
            .with_source_id(source_id);
        if document.is_within_safety_limits() {
            prime_semantic_cache(&document);
        }
        bytes += source_bytes;
        *remaining_bytes -= source_bytes;
        *remaining_files -= 1;
        documents.push(document);
    }

    ProjectSnapshot { context, documents }
}
