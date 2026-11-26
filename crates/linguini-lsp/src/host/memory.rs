//! Host-provided files for web workers and other runtimes without native filesystem access.
use super::*;
use linguini_config::{
    locale_files_from_paths, parse_config, schema_files_from_paths, DEFAULT_CONFIG_FILE,
};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// A saved workspace file. Open buffers are still synchronized through standard LSP messages.
#[derive(Debug, Clone)]
pub struct WorkspaceFile {
    pub uri: String,
    pub text: String,
}

/// Source roots parsed from a project configuration by the shared Rust configuration parser.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceSourceRoots {
    pub config_uri: String,
    pub schema_uri: String,
    pub locale_uri: String,
}

/// Valid projects can be indexed even when another configuration is invalid.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceSourcePlan {
    pub roots: Vec<WorkspaceSourceRoots>,
    pub errors: Vec<String>,
}

/// A bounded, saved-file snapshot owned by the embedding host.
///
/// The host resolves its workspace URIs to absolute local `file:` URIs before supplying files
/// and protocol messages. Updating saved files does not change open-buffer versions; notify
/// `workspace/didChangeWatchedFiles` after an update to refresh the shared server's index.
#[derive(Debug, Clone, Default)]
pub struct MemoryWorkspace(Arc<RwLock<BTreeMap<PathBuf, Arc<str>>>>);

impl MemoryWorkspace {
    /// Atomically replace the saved-file snapshot, rejecting invalid or oversized input.
    pub fn replace(&self, files: impl IntoIterator<Item = WorkspaceFile>) -> Result<(), String> {
        let mut next = BTreeMap::new();
        let mut bytes = 0usize;
        for file in files {
            let path = file_path(&file.uri)?;
            validate_source(&file.text)?;
            bytes = bytes.saturating_add(file.text.len());
            if next.len() >= MAX_WORKSPACE_ENTRIES || bytes as u64 > MAX_INDEXED_BYTES {
                return Err("saved workspace exceeds its file or byte budget".into());
            }
            if next.insert(path, Arc::from(file.text)).is_some() {
                return Err(format!("duplicate workspace URI: {}", file.uri));
            }
        }
        *self.0.write().map_err(|_| "workspace lock poisoned")? = next;
        Ok(())
    }

    /// Insert, replace, or remove one saved file. Failed updates leave the snapshot intact.
    pub fn update(&self, uri: &str, text: Option<String>) -> Result<(), String> {
        let path = file_path(uri)?;
        let mut files = self.0.write().map_err(|_| "workspace lock poisoned")?;
        if let Some(text) = text {
            validate_source(&text)?;
            let bytes: usize = files.values().map(|source| source.len()).sum();
            let previous = files.get(&path).map_or(0, |source| source.len());
            if (!files.contains_key(&path) && files.len() >= MAX_WORKSPACE_ENTRIES)
                || (bytes - previous).saturating_add(text.len()) as u64 > MAX_INDEXED_BYTES
            {
                return Err("saved workspace exceeds its file or byte budget".into());
            }
            files.insert(path, Arc::from(text));
        } else {
            files.remove(&path);
        }
        Ok(())
    }

    /// Resolve configured schema/locale roots before the host reads their source files.
    /// Configuration parsing, validation, and namespace policy remain entirely in Rust.
    pub fn source_roots(&self, roots: &[String]) -> Result<Vec<WorkspaceSourceRoots>, String> {
        let plan = self.source_plan(roots)?;
        if !plan.errors.is_empty() {
            return Err(plan.errors.join("\n"));
        }
        Ok(plan.roots)
    }

    /// Plan valid source roots and retain configuration errors for host reporting.
    pub fn source_plan(&self, roots: &[String]) -> Result<WorkspaceSourcePlan, String> {
        let roots = roots
            .iter()
            .map(|uri| file_path(uri))
            .collect::<Result<Vec<_>, _>>()?;
        let files = self.0.read().map_err(|_| "workspace lock poisoned")?;
        let (mut contexts, mut errors) = contexts(&files, &roots);
        if contexts.len() > MAX_WORKSPACE_PROJECTS {
            push_index_error(
                &mut errors,
                format!("workspace exceeds {MAX_WORKSPACE_PROJECTS} projects"),
            );
            contexts.sort_by(|left, right| {
                right
                    .root
                    .components()
                    .count()
                    .cmp(&left.root.components().count())
                    .then_with(|| left.root.cmp(&right.root))
            });
            contexts.truncate(MAX_WORKSPACE_PROJECTS);
        }
        let roots = contexts
            .into_iter()
            .map(|context| {
                Ok(WorkspaceSourceRoots {
                    config_uri: uri(&context.root.join(DEFAULT_CONFIG_FILE))?,
                    schema_uri: uri(&context.schema_root)?,
                    locale_uri: uri(&context.locale_root)?,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(WorkspaceSourcePlan { roots, errors })
    }

    /// Create the same protocol/semantic service used by the native stdio server.
    pub fn service(
        &self,
        scheduler: Arc<dyn LspScheduler>,
    ) -> (
        tower_lsp_server::LspService<impl tower_lsp_server::LanguageServer>,
        tower_lsp_server::ClientSocket,
    ) {
        crate::server::service(Arc::new(MemoryHost {
            workspace: self.clone(),
            scheduler,
        }))
    }
}

fn file_path(uri: &str) -> Result<PathBuf, String> {
    uri.parse::<Uri>()
        .ok()
        .as_ref()
        .and_then(uri_to_file_path)
        .ok_or_else(|| format!("expected an absolute local file URI: {uri}"))
}

fn uri(path: &Path) -> Result<String, String> {
    path_to_uri(path)
        .map(|uri| uri.to_string())
        .ok_or_else(|| format!("invalid source path: {}", path.display()))
}

fn validate_source(text: &str) -> Result<(), String> {
    if text.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(format!("workspace file exceeds {MAX_DOCUMENT_BYTES} bytes"));
    }
    Ok(())
}

fn contexts(
    files: &BTreeMap<PathBuf, Arc<str>>,
    roots: &[PathBuf],
) -> (Vec<ProjectContext>, Vec<String>) {
    let mut contexts = Vec::new();
    let mut errors = Vec::new();
    let mut roots = roots.to_vec();
    roots.sort();
    roots.dedup();
    if roots.len() > MAX_WORKSPACE_ROOTS {
        push_index_error(
            &mut errors,
            format!("workspace exceeds {MAX_WORKSPACE_ROOTS} roots"),
        );
        roots.truncate(MAX_WORKSPACE_ROOTS);
    }
    for (path, source) in files {
        if path.file_name().and_then(|name| name.to_str()) != Some(DEFAULT_CONFIG_FILE) {
            continue;
        }
        let visible = roots.iter().any(|root| {
            path.strip_prefix(root).ok().is_some_and(|relative| {
                relative.components().count() <= MAX_WORKSPACE_DEPTH + 1
                    && !relative.components().any(|part| {
                        part.as_os_str()
                            .to_str()
                            .is_some_and(|name| WORKSPACE_IGNORED_DIRECTORIES.contains(&name))
                    })
            })
        });
        if !visible {
            continue;
        }
        let Some(root) = path.parent() else {
            continue;
        };
        match parse_config(source) {
            Ok(config) => contexts.push(ProjectContext {
                root: root.to_path_buf(),
                schema_root: root.join(config.paths().schema()),
                locale_root: root.join(config.paths().locale()),
            }),
            Err(error) => {
                push_index_error(&mut errors, format!("invalid {}: {error}", path.display()))
            }
        }
    }
    (contexts, errors)
}

#[derive(Debug)]
struct MemoryHost {
    workspace: MemoryWorkspace,
    scheduler: Arc<dyn LspScheduler>,
}

impl WorkspaceHost for MemoryHost {
    fn workspace_index(&self, roots: Vec<PathBuf>) -> IndexFuture {
        // Clone only Arc handles, so request handlers never hold a workspace lock while awaiting.
        let result = self
            .workspace
            .0
            .read()
            .map(|files| files.clone())
            .map_err(|_| "workspace lock poisoned".to_owned());
        Box::pin(async move {
            let files = result?;
            let (contexts, errors) = contexts(&files, &roots);
            Ok(super::snapshot::index_contexts(
                contexts,
                errors,
                &MemorySources(files),
            ))
        })
    }
    fn spawn_after(&self, delay: Duration, task: LspTask) {
        self.scheduler.spawn_after(delay, task);
    }
    fn dynamic_watching(&self) -> bool {
        false
    }
}

struct MemorySources(BTreeMap<PathBuf, Arc<str>>);
impl super::snapshot::WorkspaceSources for MemorySources {
    fn schema_files(&self, root: &Path) -> Result<Vec<linguini_config::SchemaFile>, String> {
        schema_files_from_paths(root, self.paths(root, "lgs")).map_err(|error| error.to_string())
    }
    fn locale_files(&self, root: &Path) -> Result<Vec<linguini_config::LocaleFile>, String> {
        locale_files_from_paths(root, self.paths(root, "lgl")).map_err(|error| error.to_string())
    }
    fn identity(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }
    fn read(&self, path: &Path) -> Result<String, String> {
        self.0
            .get(path)
            .map(|source| source.to_string())
            .ok_or_else(|| format!("missing source: {}", path.display()))
    }
}
impl MemorySources {
    fn paths(&self, root: &Path, extension: &str) -> Vec<PathBuf> {
        self.0
            .keys()
            .filter(|path| {
                path.starts_with(root)
                    && path.extension().and_then(|ext| ext.to_str()) == Some(extension)
            })
            .cloned()
            .collect()
    }
}
