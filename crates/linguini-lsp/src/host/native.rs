//! Native filesystem discovery and bounded workspace snapshots.
use super::*;
use linguini_config::{
    discover_locale_files, discover_schema_files, parse_config, DEFAULT_CONFIG_FILE,
};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(crate) struct NativeHost;

impl WorkspaceHost for NativeHost {
    fn default_roots(&self) -> Vec<PathBuf> {
        std::env::current_dir().into_iter().collect()
    }
    fn workspace_index(&self, roots: Vec<PathBuf>) -> IndexFuture {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || build_workspace_index(roots))
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn spawn_after(&self, delay: Duration, task: LspTask) {
        NativeScheduler.spawn_after(delay, task);
    }
}

fn build_workspace_index(mut roots: Vec<PathBuf>) -> WorkspaceIndex {
    let mut index = WorkspaceIndex::default();
    let mut contexts = Vec::new();
    let mut seen_contexts = HashSet::new();
    let mut remaining_entries = MAX_WORKSPACE_ENTRIES;

    roots.sort();
    roots.dedup();
    if roots.len() > MAX_WORKSPACE_ROOTS {
        push_index_error(
            &mut index.errors,
            format!(
                "workspace exposes {} roots; indexing first {MAX_WORKSPACE_ROOTS}",
                roots.len()
            ),
        );
        roots.truncate(MAX_WORKSPACE_ROOTS);
    }
    for root in roots {
        if remaining_entries == 0 || contexts.len() >= MAX_WORKSPACE_PROJECTS {
            break;
        }
        let root = match fs::canonicalize(&root) {
            Ok(root) => root,
            Err(error) => {
                push_index_error(
                    &mut index.errors,
                    format!("cannot index workspace root {}: {error}", root.display()),
                );
                continue;
            }
        };
        let mut state = WorkspaceDiscovery {
            root: root.clone(),
            visited: HashSet::new(),
            entries: 0,
            entry_limit: remaining_entries,
        };
        discover_project_contexts_recursively(
            &root,
            0,
            &mut state,
            &mut seen_contexts,
            &mut contexts,
            &mut index.errors,
        );
        remaining_entries = remaining_entries.saturating_sub(state.entries);
    }

    super::snapshot::index_contexts(contexts, index.errors, &NativeSources)
}

struct WorkspaceDiscovery {
    root: PathBuf,
    visited: HashSet<PathBuf>,
    entries: usize,
    entry_limit: usize,
}

fn discover_project_contexts_recursively(
    directory: &Path,
    depth: usize,
    state: &mut WorkspaceDiscovery,
    seen_contexts: &mut HashSet<PathBuf>,
    contexts: &mut Vec<ProjectContext>,
    errors: &mut Vec<String>,
) {
    if depth > MAX_WORKSPACE_DEPTH {
        push_index_error(
            errors,
            format!(
                "workspace discovery depth exceeds {MAX_WORKSPACE_DEPTH} under {}",
                state.root.display()
            ),
        );
        return;
    }
    let metadata = match fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) => {
            push_index_error(
                errors,
                format!("cannot inspect {}: {error}", directory.display()),
            );
            return;
        }
    };
    if metadata.file_type().is_symlink() {
        push_index_error(
            errors,
            format!(
                "workspace discovery skipped symlink {}",
                directory.display()
            ),
        );
        return;
    }
    let canonical = match fs::canonicalize(directory) {
        Ok(canonical) => canonical,
        Err(error) => {
            push_index_error(
                errors,
                format!("cannot canonicalize {}: {error}", directory.display()),
            );
            return;
        }
    };
    if !canonical.starts_with(&state.root) || !state.visited.insert(canonical) {
        return;
    }
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            push_index_error(
                errors,
                format!("cannot read directory {}: {error}", directory.display()),
            );
            return;
        }
    };

    for entry in entries {
        if state.entries >= state.entry_limit {
            push_index_error(
                errors,
                format!(
                    "workspace discovery reached its global limit of {MAX_WORKSPACE_ENTRIES} entries"
                ),
            );
            return;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                push_index_error(errors, format!("workspace directory entry failed: {error}"));
                continue;
            }
        };
        state.entries += 1;
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                push_index_error(
                    errors,
                    format!("cannot inspect workspace entry {}: {error}", path.display()),
                );
                continue;
            }
        };
        if file_type.is_symlink() {
            push_index_error(
                errors,
                format!("workspace discovery skipped symlink {}", path.display()),
            );
            continue;
        }
        if file_type.is_dir() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                push_index_error(
                    errors,
                    format!(
                        "workspace discovery skipped non-UTF-8 directory {}",
                        path.display()
                    ),
                );
                continue;
            };
            if WORKSPACE_IGNORED_DIRECTORIES.contains(&name) {
                continue;
            }
            discover_project_contexts_recursively(
                &path,
                depth + 1,
                state,
                seen_contexts,
                contexts,
                errors,
            );
            continue;
        }

        if !file_type.is_file()
            || path.file_name().and_then(|name| name.to_str()) != Some(DEFAULT_CONFIG_FILE)
        {
            continue;
        }
        let Some(root) = path.parent() else {
            continue;
        };
        let root = normalize_existing_path(root);
        if !seen_contexts.insert(root.clone()) {
            continue;
        }
        if contexts.len() >= MAX_WORKSPACE_PROJECTS {
            push_index_error(
                errors,
                format!(
                    "workspace discovery reached its global limit of {MAX_WORKSPACE_PROJECTS} projects"
                ),
            );
            return;
        }
        match load_project_context(&root) {
            Ok(context) => contexts.push(context),
            Err(error) => push_index_error(errors, error),
        }
    }
}

fn load_project_context(root: &Path) -> std::result::Result<ProjectContext, String> {
    let config_path = root.join(DEFAULT_CONFIG_FILE);
    let source = fs::read_to_string(&config_path)
        .map_err(|error| format!("cannot read {}: {error}", config_path.display()))?;
    let config = parse_config(&source)
        .map_err(|error| format!("invalid {}: {error}", config_path.display()))?;
    Ok(ProjectContext {
        root: normalize_existing_path(root),
        schema_root: normalize_existing_path(root.join(config.paths().schema())),
        locale_root: normalize_existing_path(root.join(config.paths().locale())),
        locale_policy: super::configured_locale_policy(&config),
    })
}

struct NativeSources;
impl super::snapshot::WorkspaceSources for NativeSources {
    fn schema_files(&self, root: &Path) -> Result<Vec<linguini_config::SchemaFile>, String> {
        discover_schema_files(root).map_err(|error| error.to_string())
    }
    fn locale_files(&self, root: &Path) -> Result<Vec<linguini_config::LocaleFile>, String> {
        discover_locale_files(root).map_err(|error| error.to_string())
    }
    fn identity(&self, path: &Path) -> PathBuf {
        normalize_existing_path(path)
    }
    fn read(&self, path: &Path) -> Result<String, String> {
        let metadata = fs::metadata(path)
            .map_err(|error| format!("cannot inspect source {}: {error}", path.display()))?;
        if metadata.len() > MAX_DOCUMENT_BYTES {
            return Err(format!(
                "source {} exceeds {MAX_DOCUMENT_BYTES} bytes",
                path.display()
            ));
        }
        let mut source = String::new();
        fs::File::open(path)
            .and_then(|file| {
                file.take(MAX_DOCUMENT_BYTES + 1)
                    .read_to_string(&mut source)
            })
            .map_err(|error| format!("cannot read source {}: {error}", path.display()))?;
        Ok(source)
    }
}

fn normalize_existing_path(path: impl AsRef<Path>) -> PathBuf {
    let path = fs::canonicalize(path.as_ref()).unwrap_or_else(|_| path.as_ref().to_path_buf());
    // Canonical Windows paths carry a verbatim prefix, but LSP file URIs decode
    // to ordinary drive/UNC paths. Contexts must use the same identity as buffers.
    path_to_uri(&path)
        .and_then(|uri| uri_to_file_path(&uri))
        .unwrap_or(path)
}
