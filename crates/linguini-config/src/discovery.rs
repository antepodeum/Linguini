use crate::error::{ConfigError, ConfigResult};
use crate::model::{validate_locale_tag, validate_relative_path};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

const MAX_DISCOVERY_DEPTH: usize = 128;
const MAX_DISCOVERY_ENTRIES: usize = 100_000;
const APPLICATION_SOURCE_EXTENSIONS: [&str; 11] = [
    "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "svelte", "vue", "astro",
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SchemaFile {
    pub path: PathBuf,
    /// Dot-separated path relative to the schema root, including the file stem.
    pub namespace: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocaleFile {
    pub path: PathBuf,
    /// Validated BCP 47 tag read verbatim from the file stem.
    pub locale: String,
    /// Dot-separated parent path relative to the locale root.
    pub namespace: String,
}

/// Discovers `.lgs` files and derives each filesystem namespace from its relative path.
pub fn discover_schema_files(root: impl AsRef<Path>) -> ConfigResult<Vec<SchemaFile>> {
    let root = prepare_root(root.as_ref())?;
    let mut state = DiscoveryState::new(&root);
    collect_schema_files(&root, &root, 0, &mut state)?;
    state.schema_files.sort_by(|left, right| {
        portable_sort_key(&root, &left.path).cmp(&portable_sort_key(&root, &right.path))
    });
    ensure_case_distinct(
        state
            .schema_files
            .iter()
            .map(|file| (&file.path, file.namespace.as_str())),
        "schema namespace",
    )?;
    Ok(state.schema_files)
}

/// Discovers `.lgl` files, using each parent path as its namespace and stem as its locale.
pub fn discover_locale_files(root: impl AsRef<Path>) -> ConfigResult<Vec<LocaleFile>> {
    let root = prepare_root(root.as_ref())?;
    let mut state = DiscoveryState::new(&root);
    collect_locale_files(&root, &root, 0, &mut state)?;
    state.locale_files.sort_by(|left, right| {
        portable_sort_key(&root, &left.path).cmp(&portable_sort_key(&root, &right.path))
    });
    ensure_case_distinct(
        state
            .locale_files
            .iter()
            .map(|file| (&file.path, format!("{}:{}", file.namespace, file.locale))),
        "locale namespace and tag",
    )?;
    Ok(state.locale_files)
}

/// Discovers configured JavaScript, TypeScript, and component application sources.
///
/// `sources` and `exclude` entries are portable project-relative file or directory paths.
/// Excluded directories are not traversed. Symbolic links are rejected everywhere else.
pub fn discover_application_source_files(
    project_root: impl AsRef<Path>,
    sources: &[String],
    exclude: &[String],
) -> ConfigResult<Vec<PathBuf>> {
    discover_application_source_files_with_fields(
        project_root,
        sources,
        exclude,
        "analysis.unused_messages.sources",
        "analysis.unused_messages.exclude",
    )
}

/// Discovers application sources while attributing configuration diagnostics to the caller's
/// source and exclusion fields.
pub fn discover_application_source_files_with_fields(
    project_root: impl AsRef<Path>,
    sources: &[String],
    exclude: &[String],
    sources_field: &'static str,
    exclude_field: &'static str,
) -> ConfigResult<Vec<PathBuf>> {
    let root = prepare_root(project_root.as_ref())?;
    let sources = configured_paths(sources_field, sources)?;
    let exclude = configured_paths(exclude_field, exclude)?;
    let mut state = DiscoveryState::new(&root);

    for relative in sources {
        if path_is_excluded(&relative, &exclude) {
            return Err(ConfigError::InvalidPath {
                field: sources_field,
                value: relative.to_string_lossy().replace('\\', "/"),
                reason: if exclude_field == "targets.ts.bundler.exclude" {
                    "source is fully covered by targets.ts.bundler.exclude"
                } else {
                    "source is fully covered by analysis.unused_messages.exclude"
                },
            });
        }
        reject_symlink_components(&root, &relative)?;
        let path = root.join(&relative);
        let metadata = fs::symlink_metadata(&path).map_err(|error| io_error(&path, error))?;
        if metadata.file_type().is_symlink() {
            return Err(ConfigError::UnsupportedSymlink(path));
        }
        if metadata.is_dir() {
            collect_application_source_files(&root, &path, 0, &exclude, &mut state)?;
        } else if metadata.is_file() {
            if !is_application_source_file(&path) {
                return Err(ConfigError::Io {
                    path,
                    kind: std::io::ErrorKind::InvalidInput,
                    message: "configured application source file has an unsupported extension"
                        .to_owned(),
                });
            }
            state.count_entry()?;
            state.add_application_file(path)?;
        } else {
            return Err(ConfigError::Io {
                path,
                kind: std::io::ErrorKind::InvalidInput,
                message: "configured application source is neither a file nor a directory"
                    .to_owned(),
            });
        }
    }

    let mut files = state.application_files.into_iter().collect::<Vec<_>>();
    files.sort_by(|left, right| {
        portable_sort_key(&root, left).cmp(&portable_sort_key(&root, right))
    });
    Ok(files)
}

pub fn locale_scope_chain(
    locale_root: impl AsRef<Path>,
    file: impl AsRef<Path>,
) -> ConfigResult<Vec<PathBuf>> {
    let locale_root = locale_root.as_ref();
    let file = file.as_ref();
    let relative = file
        .strip_prefix(locale_root)
        .map_err(|_| ConfigError::PathOutsideRoot {
            root: locale_root.to_path_buf(),
            path: file.to_path_buf(),
        })?;
    if relative.components().any(|component| {
        !matches!(component, Component::Normal(_)) || component.as_os_str().to_str().is_none()
    }) {
        return Err(ConfigError::PathOutsideRoot {
            root: locale_root.to_path_buf(),
            path: file.to_path_buf(),
        });
    }

    let locale_name = relative
        .file_name()
        .ok_or_else(|| ConfigError::PathOutsideRoot {
            root: locale_root.to_path_buf(),
            path: file.to_path_buf(),
        })?;
    let relative_parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let mut paths = Vec::new();
    let mut current = locale_root.to_path_buf();
    paths.push(current.join(locale_name));

    for component in relative_parent.components() {
        current.push(component.as_os_str());
        paths.push(current.join(locale_name));
    }
    Ok(paths)
}

struct DiscoveryState {
    root: PathBuf,
    visited: BTreeSet<PathBuf>,
    entries: usize,
    schema_files: Vec<SchemaFile>,
    locale_files: Vec<LocaleFile>,
    application_files: BTreeSet<PathBuf>,
}

impl DiscoveryState {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            visited: BTreeSet::new(),
            entries: 0,
            schema_files: Vec::new(),
            locale_files: Vec::new(),
            application_files: BTreeSet::new(),
        }
    }

    fn enter(&mut self, directory: &Path, depth: usize) -> ConfigResult<()> {
        if !self.enter_once(directory, depth)? {
            return Err(ConfigError::UnsupportedSymlink(directory.to_path_buf()));
        }
        Ok(())
    }

    fn enter_once(&mut self, directory: &Path, depth: usize) -> ConfigResult<bool> {
        if depth > MAX_DISCOVERY_DEPTH {
            return Err(ConfigError::DiscoveryLimit {
                root: self.root.clone(),
                limit: MAX_DISCOVERY_DEPTH,
            });
        }
        let canonical = fs::canonicalize(directory).map_err(|error| io_error(directory, error))?;
        if !canonical.starts_with(&self.root) {
            return Err(ConfigError::PathOutsideRoot {
                root: self.root.clone(),
                path: canonical,
            });
        }
        Ok(self.visited.insert(canonical))
    }

    fn count_entry(&mut self) -> ConfigResult<()> {
        self.entries += 1;
        if self.entries > MAX_DISCOVERY_ENTRIES {
            return Err(ConfigError::DiscoveryLimit {
                root: self.root.clone(),
                limit: MAX_DISCOVERY_ENTRIES,
            });
        }
        Ok(())
    }

    fn add_application_file(&mut self, path: PathBuf) -> ConfigResult<()> {
        if !is_application_source_file(&path) || self.application_files.contains(&path) {
            return Ok(());
        }
        require_utf8_component(&path)?;
        fs::File::open(&path).map_err(|error| io_error(&path, error))?;
        self.application_files.insert(path);
        Ok(())
    }
}

fn collect_schema_files(
    root: &Path,
    directory: &Path,
    depth: usize,
    state: &mut DiscoveryState,
) -> ConfigResult<()> {
    state.enter(directory, depth)?;
    for entry in read_directory(directory)? {
        state.count_entry()?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| io_error(&path, error))?;
        if file_type.is_symlink() {
            return Err(ConfigError::UnsupportedSymlink(path));
        }
        if file_type.is_dir() {
            require_utf8_component(&path)?;
            collect_schema_files(root, &path, depth + 1, state)?;
        } else if file_type.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("lgs")
        {
            state.schema_files.push(SchemaFile {
                namespace: namespace_from_path(root, &path, true)?,
                path,
            });
        }
    }
    Ok(())
}

fn collect_locale_files(
    root: &Path,
    directory: &Path,
    depth: usize,
    state: &mut DiscoveryState,
) -> ConfigResult<()> {
    state.enter(directory, depth)?;
    for entry in read_directory(directory)? {
        state.count_entry()?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| io_error(&path, error))?;
        if file_type.is_symlink() {
            return Err(ConfigError::UnsupportedSymlink(path));
        }
        if file_type.is_dir() {
            require_utf8_component(&path)?;
            collect_locale_files(root, &path, depth + 1, state)?;
        } else if file_type.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("lgl")
        {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| ConfigError::NonUtf8Path(path.clone()))?;
            validate_locale_tag(stem)?;
            state.locale_files.push(LocaleFile {
                locale: stem.to_owned(),
                namespace: namespace_from_path(root, &path, false)?,
                path,
            });
        }
    }
    Ok(())
}

fn collect_application_source_files(
    root: &Path,
    directory: &Path,
    depth: usize,
    exclude: &[PathBuf],
    state: &mut DiscoveryState,
) -> ConfigResult<()> {
    if !state.enter_once(directory, depth)? {
        return Ok(());
    }
    for entry in read_directory(directory)? {
        state.count_entry()?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| ConfigError::PathOutsideRoot {
                root: root.to_path_buf(),
                path: path.clone(),
            })?;
        if path_is_excluded(relative, exclude) {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| io_error(&path, error))?;
        if file_type.is_symlink() {
            return Err(ConfigError::UnsupportedSymlink(path));
        }
        if file_type.is_dir() {
            require_utf8_component(&path)?;
            collect_application_source_files(root, &path, depth + 1, exclude, state)?;
        } else if file_type.is_file() {
            state.add_application_file(path)?;
        }
    }
    Ok(())
}

fn configured_paths(field: &'static str, values: &[String]) -> ConfigResult<Vec<PathBuf>> {
    let mut paths = BTreeSet::new();
    for value in values {
        validate_relative_path(field, value)?;
        paths.insert(
            value
                .trim()
                .split('/')
                .filter(|component| !component.is_empty() && *component != ".")
                .collect::<PathBuf>(),
        );
    }
    let mut paths = paths.into_iter().collect::<Vec<_>>();
    paths.sort_by(|left, right| {
        let left = left.to_string_lossy().replace('\\', "/");
        let right = right.to_string_lossy().replace('\\', "/");
        (left.to_ascii_lowercase(), &left).cmp(&(right.to_ascii_lowercase(), &right))
    });
    Ok(paths)
}

fn path_is_excluded(relative: &Path, exclude: &[PathBuf]) -> bool {
    exclude
        .iter()
        .any(|excluded| portable_path_starts_with(relative, excluded))
}

fn portable_path_starts_with(path: &Path, prefix: &Path) -> bool {
    let mut path_components = path.components();
    prefix.components().all(|prefix_component| {
        let Some(path_component) = path_components.next() else {
            return false;
        };
        let (Component::Normal(path_component), Component::Normal(prefix_component)) =
            (path_component, prefix_component)
        else {
            return false;
        };
        path_component
            .to_str()
            .zip(prefix_component.to_str())
            .is_some_and(|(path_component, prefix_component)| {
                path_component.eq_ignore_ascii_case(prefix_component)
            })
    })
}

fn reject_symlink_components(root: &Path, relative: &Path) -> ConfigResult<()> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(ConfigError::PathOutsideRoot {
                root: root.to_path_buf(),
                path: root.join(relative),
            });
        };
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|error| io_error(&current, error))?;
        if metadata.file_type().is_symlink() {
            return Err(ConfigError::UnsupportedSymlink(current));
        }
    }
    Ok(())
}

fn is_application_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| APPLICATION_SOURCE_EXTENSIONS.contains(&extension))
}

fn prepare_root(path: &Path) -> ConfigResult<PathBuf> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error(path, error))?;
    if metadata.file_type().is_symlink() {
        return Err(ConfigError::UnsupportedSymlink(path.to_path_buf()));
    }
    if !metadata.is_dir() {
        return Err(ConfigError::Io {
            path: path.to_path_buf(),
            kind: std::io::ErrorKind::InvalidInput,
            message: "source root is not a directory".to_owned(),
        });
    }
    fs::canonicalize(path).map_err(|error| io_error(path, error))
}

fn read_directory(path: &Path) -> ConfigResult<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| io_error(path, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error(path, error))?;
    entries.sort_by(|left, right| {
        left.file_name()
            .to_string_lossy()
            .cmp(&right.file_name().to_string_lossy())
    });
    Ok(entries)
}

fn namespace_from_path(root: &Path, path: &Path, include_file_stem: bool) -> ConfigResult<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| ConfigError::PathOutsideRoot {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })?;
    let mut parts = Vec::new();

    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let Component::Normal(component) = component else {
                return Err(ConfigError::PathOutsideRoot {
                    root: root.to_path_buf(),
                    path: path.to_path_buf(),
                });
            };
            parts.push(
                component
                    .to_str()
                    .ok_or_else(|| ConfigError::NonUtf8Path(path.to_path_buf()))?
                    .to_owned(),
            );
        }
    }
    if include_file_stem {
        parts.push(
            relative
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| ConfigError::NonUtf8Path(path.to_path_buf()))?
                .to_owned(),
        );
    }
    Ok(parts.join("."))
}

fn require_utf8_component(path: &Path) -> ConfigResult<()> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|_| ())
        .ok_or_else(|| ConfigError::NonUtf8Path(path.to_path_buf()))
}

fn portable_sort_key(root: &Path, path: &Path) -> (String, String) {
    let original = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    (original.to_ascii_lowercase(), original)
}

fn ensure_case_distinct<'a, V: AsRef<str> + 'a>(
    entries: impl IntoIterator<Item = (&'a PathBuf, V)>,
    kind: &str,
) -> ConfigResult<()> {
    let mut seen = BTreeSet::new();
    for (path, value) in entries {
        let value = value.as_ref();
        if !seen.insert(value.to_ascii_lowercase()) {
            return Err(ConfigError::DuplicateKey(format!(
                "{kind} `{value}` derived from {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn io_error(path: &Path, error: std::io::Error) -> ConfigError {
    ConfigError::Io {
        path: path.to_path_buf(),
        kind: error.kind(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        discover_application_source_files, discover_locale_files, discover_schema_files,
        locale_scope_chain, namespace_from_path,
    };
    use crate::ConfigError;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn derives_schema_namespace_from_path() {
        let namespace = namespace_from_path(
            Path::new("linguini/schema"),
            Path::new("linguini/schema/shop/delivery.lgs"),
            true,
        )
        .expect("namespace");
        assert_eq!(namespace, "shop.delivery");
    }

    #[test]
    fn derives_locale_namespace_from_parent_path() {
        let namespace = namespace_from_path(
            Path::new("linguini/locale"),
            Path::new("linguini/locale/shop/forms/fruit/ru.lgl"),
            false,
        )
        .expect("namespace");
        assert_eq!(namespace, "shop.forms.fruit");
    }

    #[test]
    fn rejects_paths_outside_namespace_root() {
        let error = namespace_from_path(
            Path::new("linguini/schema"),
            Path::new("elsewhere/shop.lgs"),
            true,
        )
        .expect_err("outside path");
        assert!(matches!(error, ConfigError::PathOutsideRoot { .. }));
    }

    #[test]
    fn builds_top_down_locale_scope_chain() {
        let chain = locale_scope_chain("linguini/locale", "linguini/locale/shop/delivery/ru.lgl")
            .expect("scope chain");
        assert_eq!(
            chain,
            [
                Path::new("linguini/locale/ru.lgl").to_path_buf(),
                Path::new("linguini/locale/shop/ru.lgl").to_path_buf(),
                Path::new("linguini/locale/shop/delivery/ru.lgl").to_path_buf(),
            ]
        );
    }

    #[test]
    fn locale_scope_chain_rejects_outside_path() {
        assert!(locale_scope_chain("locales", "../outside/en.lgl").is_err());
    }

    #[test]
    fn discovers_project_structure_namespaces_and_locales() {
        let root = TempDir::new().expect("root");
        let schema_root = root.path().join("schema");
        let locale_root = root.path().join("locales");
        fs::create_dir_all(schema_root.join("shop/forms")).expect("schema dirs");
        fs::create_dir_all(locale_root.join("shop/forms/cart")).expect("locale dirs");
        fs::write(schema_root.join("shop/forms/cart.lgs"), "cart()\n").expect("schema file");
        fs::write(
            locale_root.join("shop/forms/cart/en-US.lgl"),
            "cart = Cart\n",
        )
        .expect("locale file");

        let schemas = discover_schema_files(&schema_root).expect("schema discovery");
        let locales = discover_locale_files(&locale_root).expect("locale discovery");

        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].namespace, "shop.forms.cart");
        assert_eq!(locales.len(), 1);
        assert_eq!(locales[0].locale, "en-US");
        assert_eq!(locales[0].namespace, "shop.forms.cart");
    }

    #[test]
    fn discovers_supported_application_sources_deterministically() {
        let root = TempDir::new().expect("root");
        fs::create_dir_all(root.path().join("src/components/generated")).expect("source dirs");
        for extension in [
            "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "svelte", "vue", "astro",
        ] {
            fs::write(
                root.path()
                    .join("src/components")
                    .join(format!("source.{extension}")),
                "l.main.title();\n",
            )
            .expect("application source");
        }
        fs::write(root.path().join("src/components/ignored.css"), "body {}")
            .expect("unsupported source");
        fs::write(
            root.path().join("src/components/generated/output.ts"),
            "generated();\n",
        )
        .expect("excluded source");

        let files = discover_application_source_files(
            root.path(),
            &["src/components".to_owned(), "src".to_owned()],
            &["src/components/generated".to_owned()],
        )
        .expect("application discovery");
        let relative = files
            .iter()
            .map(|path| {
                path.strip_prefix(root.path())
                    .expect("project-relative path")
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            relative,
            [
                "src/components/source.astro",
                "src/components/source.cjs",
                "src/components/source.cts",
                "src/components/source.js",
                "src/components/source.jsx",
                "src/components/source.mjs",
                "src/components/source.mts",
                "src/components/source.svelte",
                "src/components/source.ts",
                "src/components/source.tsx",
                "src/components/source.vue",
            ]
        );
    }

    #[test]
    fn matches_application_excludes_with_portable_ascii_case_folding() {
        let root = TempDir::new().expect("root");
        fs::create_dir_all(root.path().join("src/generated")).expect("source dirs");
        fs::write(root.path().join("src/keep.ts"), "keep();\n").expect("kept source");
        fs::write(root.path().join("src/generated/drop.ts"), "generated();\n")
            .expect("excluded source");

        let files = discover_application_source_files(
            root.path(),
            &["src".to_owned()],
            &["SRC/GENERATED".to_owned()],
        )
        .expect("case-folded exclusion");

        assert_eq!(
            files,
            [fs::canonicalize(root.path().join("src/keep.ts")).unwrap()]
        );
    }

    #[test]
    fn accepts_file_sources_and_exact_file_excludes() {
        let root = TempDir::new().expect("root");
        fs::create_dir(root.path().join("src")).expect("source dir");
        fs::write(root.path().join("src/keep.ts"), "keep();\n").expect("kept source");
        fs::write(root.path().join("src/drop.ts"), "drop();\n").expect("excluded source");

        let files =
            discover_application_source_files(root.path(), &["src/keep.ts".to_owned()], &[])
                .expect("explicit application source");

        assert_eq!(
            files,
            [fs::canonicalize(root.path().join("src/keep.ts")).unwrap()]
        );

        let files = discover_application_source_files(
            root.path(),
            &["src".to_owned()],
            &["src/drop.ts".to_owned()],
        )
        .expect("exact file exclusion");
        assert_eq!(
            files,
            [fs::canonicalize(root.path().join("src/keep.ts")).unwrap()]
        );
    }

    #[test]
    fn rejects_application_source_fully_covered_by_exclude() {
        let root = TempDir::new().expect("root");

        let error = discover_application_source_files(
            root.path(),
            &["Missing/Source".to_owned()],
            &["missing".to_owned()],
        )
        .expect_err("fully excluded source rejected");

        assert!(matches!(error, ConfigError::InvalidPath { .. }));
    }

    #[test]
    fn attributes_application_discovery_errors_to_requested_fields() {
        let root = TempDir::new().expect("root");
        let error = super::discover_application_source_files_with_fields(
            root.path(),
            &["src/app".to_owned()],
            &["src".to_owned()],
            "targets.ts.bundler.sources",
            "targets.ts.bundler.exclude",
        )
        .expect_err("covered source");
        let message = error.to_string();
        assert!(message.contains("targets.ts.bundler.sources"), "{message}");
        assert!(message.contains("targets.ts.bundler.exclude"), "{message}");
        assert!(!message.contains("analysis.unused_messages"), "{message}");
    }

    #[test]
    fn rejects_explicit_files_with_unsupported_extensions() {
        let root = TempDir::new().expect("root");
        fs::write(root.path().join("styles.css"), "body {}\n").expect("unsupported source");

        let error = discover_application_source_files(root.path(), &["styles.css".to_owned()], &[])
            .expect_err("unsupported explicit file rejected");

        assert!(matches!(
            error,
            ConfigError::Io {
                kind: std::io::ErrorKind::InvalidInput,
                ..
            }
        ));
    }

    #[test]
    fn reports_missing_application_source_paths() {
        let root = TempDir::new().expect("root");

        let error = discover_application_source_files(root.path(), &["missing.ts".to_owned()], &[])
            .expect_err("missing source rejected");

        assert!(matches!(
            error,
            ConfigError::Io {
                kind: std::io::ErrorKind::NotFound,
                ..
            }
        ));
    }

    #[test]
    fn rejects_unsafe_application_source_paths() {
        let root = TempDir::new().expect("root");

        let error =
            discover_application_source_files(root.path(), &["../outside.ts".to_owned()], &[])
                .expect_err("outside source rejected");

        assert!(matches!(error, ConfigError::InvalidPath { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_application_source_symlinks() {
        use std::os::unix::fs::symlink;

        let root = TempDir::new().expect("root");
        let outside = TempDir::new().expect("outside");
        fs::write(outside.path().join("hidden.ts"), "hidden();\n").expect("outside source");
        symlink(outside.path(), root.path().join("linked")).expect("symlink");

        let error =
            discover_application_source_files(root.path(), &["linked/hidden.ts".to_owned()], &[])
                .expect_err("symlink rejected");

        assert!(matches!(error, ConfigError::UnsupportedSymlink(_)));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_directory_symlinks_without_following_them() {
        use std::os::unix::fs::symlink;

        let root = TempDir::new().expect("root");
        let outside = TempDir::new().expect("outside");
        fs::write(outside.path().join("hidden.lgs"), "hidden\n").expect("outside source");
        symlink(outside.path(), root.path().join("linked")).expect("symlink");

        let error = discover_schema_files(root.path()).expect_err("symlink rejected");

        assert!(matches!(error, ConfigError::UnsupportedSymlink(_)));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_namespace_components() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let root = TempDir::new().expect("root");
        let invalid = OsString::from_vec(vec![b'n', 0xff]);
        let directory = root.path().join(invalid);
        fs::create_dir(&directory).expect("invalid directory");
        fs::write(directory.join("message.lgs"), "message\n").expect("source");

        let error = discover_schema_files(root.path()).expect_err("non-UTF-8 rejected");

        assert!(matches!(error, ConfigError::NonUtf8Path(_)));
    }

    #[test]
    fn rejects_case_insensitive_namespace_collisions() {
        let root = TempDir::new().expect("root");
        fs::write(root.path().join("Shop.lgs"), "first\n").expect("first");
        fs::write(root.path().join("shop.lgs"), "second\n").expect("second");

        assert!(discover_schema_files(root.path()).is_err());
    }

    #[test]
    fn rejects_flattened_namespace_collisions() {
        let root = TempDir::new().expect("root");
        fs::create_dir(root.path().join("shop")).expect("nested directory");
        fs::write(root.path().join("shop.checkout.lgs"), "first\n").expect("flat source");
        fs::write(root.path().join("shop/checkout.lgs"), "second\n").expect("nested source");

        let error = discover_schema_files(root.path()).expect_err("namespace collision");
        assert!(error
            .to_string()
            .contains("schema namespace `shop.checkout`"));
    }
}
