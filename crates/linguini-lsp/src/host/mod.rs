//! Host services around the shared language-server protocol and semantic implementation.
use crate::LinguiniDocument;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;
mod memory;
mod snapshot;
pub use memory::{MemoryWorkspace, WorkspaceFile, WorkspaceSourcePlan, WorkspaceSourceRoots};

use tower_lsp_server::lsp_types::Url as Uri;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
mod native_runtime;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::NativeHost;
#[cfg(not(target_arch = "wasm32"))]
pub use native_runtime::{run_stdio, run_stdio_blocking, try_run_stdio_blocking, NativeScheduler};

/// A detached protocol task, scheduled by the owning native or browser runtime.
pub type LspTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// The runtime owns its clock and executor; language features stay in the shared server.
pub trait LspScheduler: std::fmt::Debug + Send + Sync + 'static {
    fn spawn_after(&self, delay: Duration, task: LspTask);
}

type IndexFuture = Pin<Box<dyn Future<Output = Result<WorkspaceIndex, String>> + Send + 'static>>;
pub(crate) trait WorkspaceHost: std::fmt::Debug + Send + Sync + 'static {
    fn workspace_index(&self, roots: Vec<PathBuf>) -> IndexFuture;
    fn spawn_after(&self, delay: Duration, task: LspTask);
    fn default_roots(&self) -> Vec<PathBuf> {
        Vec::new()
    }
    fn dynamic_watching(&self) -> bool {
        true
    }
}

pub(crate) const MAX_WORKSPACE_DEPTH: usize = 64;
pub(crate) const MAX_WORKSPACE_ENTRIES: usize = 50_000;
pub(crate) const MAX_WORKSPACE_ROOTS: usize = 32;
pub(crate) const MAX_WORKSPACE_PROJECTS: usize = 256;
pub(crate) const MAX_INDEXED_FILES: usize = 50_000;
pub(crate) const MAX_INDEXED_BYTES: u64 = 128 * 1024 * 1024;
pub(crate) const MAX_PROJECT_FILES: usize = 20_000;
pub(crate) const MAX_PROJECT_BYTES: u64 = 64 * 1024 * 1024;
pub(crate) const MAX_DOCUMENT_BYTES: u64 = 4 * 1024 * 1024;

/// Bounds shared with hosts that enumerate and read saved workspace files.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceLimits {
    pub max_entries: usize,
    pub max_depth: usize,
    pub max_roots: usize,
    pub max_file_bytes: u64,
    pub max_bytes: u64,
}
pub const WORKSPACE_LIMITS: WorkspaceLimits = WorkspaceLimits {
    max_entries: MAX_WORKSPACE_ENTRIES,
    max_depth: MAX_WORKSPACE_DEPTH,
    max_roots: MAX_WORKSPACE_ROOTS,
    max_file_bytes: MAX_DOCUMENT_BYTES,
    max_bytes: MAX_INDEXED_BYTES,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProjectContext {
    pub(crate) root: PathBuf,
    pub(crate) schema_root: PathBuf,
    pub(crate) locale_root: PathBuf,
    pub(crate) locale_policy: std::sync::Arc<ProjectLocalePolicy>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProjectLocalePolicy {
    default_locale: String,
    locales: Vec<String>,
}

fn configured_locale_policy(
    config: &linguini_config::LinguiniConfig,
) -> std::sync::Arc<ProjectLocalePolicy> {
    std::sync::Arc::new(ProjectLocalePolicy {
        default_locale: config.project().default_locale().to_owned(),
        locales: config.project().locales().to_vec(),
    })
}

#[derive(Debug, Clone, Default)]
pub(crate) struct WorkspaceIndex {
    pub(crate) projects: Vec<ProjectSnapshot>,
    pub(crate) errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectSnapshot {
    pub(crate) context: ProjectContext,
    pub(crate) documents: Vec<LinguiniDocument>,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn uri_to_file_path(uri: &Uri) -> Option<PathBuf> {
    if uri.scheme() != "file" || uri.query().is_some() || uri.fragment().is_some() {
        return None;
    }
    uri.to_file_path().ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn path_to_uri(path: &Path) -> Option<Uri> {
    Uri::from_file_path(path).ok()
}

fn push_index_error(errors: &mut Vec<String>, error: String) {
    if errors.len() < 100 {
        errors.push(error);
    }
}

/// Directories omitted when discovering projects; configured source roots are indexed separately.
pub const WORKSPACE_IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".svelte-kit",
    ".next",
    ".nuxt",
    ".turbo",
    "node_modules",
    "target",
    "dist",
    "build",
    "coverage",
    "vendor",
];

#[cfg(target_arch = "wasm32")]
pub(crate) fn uri_to_file_path(uri: &Uri) -> Option<PathBuf> {
    if uri.scheme() != "file"
        || uri.host_str().is_some()
        || uri.query().is_some()
        || uri.fragment().is_some()
    {
        return None;
    }
    let path = percent_encoding::percent_decode_str(uri.path())
        .decode_utf8()
        .ok()?;
    if path.contains('\0') || path.contains('\\') {
        return None;
    }
    Some(PathBuf::from(path.as_ref()))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn path_to_uri(path: &Path) -> Option<Uri> {
    let path = path.to_str()?;
    if !path.starts_with('/') || path.contains('\0') || path.contains('\\') {
        return None;
    }
    let mut uri = Uri::parse("file:///").ok()?;
    uri.set_path(path);
    Some(uri)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
