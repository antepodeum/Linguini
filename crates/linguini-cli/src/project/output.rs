use crate::{CliError, CliResult};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MANIFEST_NAME: &str = ".linguini-generated-manifest";
const MANIFEST_HEADER: &str = "linguini-generated-v1";
static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) struct GeneratedFile<'a> {
    pub(crate) path: PathBuf,
    pub(crate) contents: &'a str,
}

#[derive(Debug)]
pub(crate) struct SafeOutputRoot {
    project_root: PathBuf,
    output_root: PathBuf,
}

impl SafeOutputRoot {
    pub(crate) fn new(
        project_root: &Path,
        output: &Path,
        protected_roots: &[&Path],
    ) -> CliResult<Self> {
        let project_root = fs::canonicalize(project_root).map_err(|source| CliError::Io {
            path: project_root.to_path_buf(),
            source,
        })?;
        if !project_root.is_dir() {
            return Err(unsafe_output(output, "project root is not a directory"));
        }

        let output_root = resolve_project_path(&project_root, output, true)?;
        if output_root == project_root {
            return Err(unsafe_output(
                output,
                "generated output cannot be the project root",
            ));
        }

        for protected in protected_roots {
            let protected = resolve_project_path(&project_root, protected, false)?;
            if paths_overlap(&output_root, &protected) {
                return Err(unsafe_output(
                    output,
                    "generated output overlaps a source root",
                ));
            }
        }

        Ok(Self {
            project_root,
            output_root,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.output_root
    }

    fn prepare(&self) -> CliResult<()> {
        create_directory_chain(&self.project_root, &self.output_root)?;
        let canonical = fs::canonicalize(&self.output_root).map_err(|source| CliError::Io {
            path: self.output_root.clone(),
            source,
        })?;
        if canonical != self.output_root || !canonical.starts_with(&self.project_root) {
            return Err(unsafe_output(
                &self.output_root,
                "generated output escaped the canonical project root",
            ));
        }
        Ok(())
    }
}

pub(crate) fn replace_owned_files(
    output: &SafeOutputRoot,
    files: &[GeneratedFile<'_>],
) -> CliResult<()> {
    output.prepare()?;

    let mut new_paths = BTreeSet::new();
    for file in files {
        validate_relative_path(&file.path)?;
        if file.path == Path::new(MANIFEST_NAME) {
            return Err(unsafe_generated_path(
                &file.path,
                "path is reserved for the ownership manifest",
            ));
        }
        if !new_paths.insert(file.path.clone()) {
            return Err(unsafe_generated_path(
                &file.path,
                "codegen returned a duplicate path",
            ));
        }
    }

    let old_paths = read_manifest(output.path())?;
    validate_existing_ownership(output.path(), &old_paths, &new_paths)?;

    let transaction = TransactionDir::create(
        output
            .path()
            .parent()
            .expect("validated output has a parent"),
    )?;
    let staged_root = transaction.path().join("new");
    let backup_root = transaction.path().join("backup");
    fs::create_dir(&staged_root).map_err(|source| CliError::Io {
        path: staged_root.clone(),
        source,
    })?;
    fs::create_dir(&backup_root).map_err(|source| CliError::Io {
        path: backup_root.clone(),
        source,
    })?;

    for file in files {
        write_staged_file(&staged_root.join(&file.path), file.contents.as_bytes())?;
    }
    let manifest = render_manifest(&new_paths);
    write_staged_file(&staged_root.join(MANIFEST_NAME), manifest.as_bytes())?;

    let mut affected = old_paths
        .union(&new_paths)
        .cloned()
        .collect::<BTreeSet<_>>();
    affected.insert(PathBuf::from(MANIFEST_NAME));

    let mut backed_up = Vec::new();
    for relative in &affected {
        let current = output.path().join(relative);
        if !path_exists(&current)? {
            continue;
        }
        let backup = backup_root.join(relative);
        create_parent_directories(&backup)?;
        if let Err(source) = fs::rename(&current, &backup) {
            let primary = CliError::Io {
                path: current,
                source,
            };
            return Err(rollback(
                output.path(),
                &backup_root,
                &[],
                &backed_up,
                primary,
            ));
        }
        backed_up.push(relative.clone());
    }

    let mut installed = Vec::new();
    let mut install_paths = new_paths.iter().cloned().collect::<Vec<_>>();
    install_paths.push(PathBuf::from(MANIFEST_NAME));
    for relative in install_paths {
        let staged = staged_root.join(&relative);
        let destination = output.path().join(&relative);
        if let Err(error) = create_parent_directories(&destination) {
            return Err(rollback(
                output.path(),
                &backup_root,
                &installed,
                &backed_up,
                error,
            ));
        }
        if let Err(source) = fs::rename(&staged, &destination) {
            let primary = CliError::Io {
                path: destination,
                source,
            };
            return Err(rollback(
                output.path(),
                &backup_root,
                &installed,
                &backed_up,
                primary,
            ));
        }
        installed.push(relative);
    }

    prune_empty_owned_directories(output.path(), &old_paths, &new_paths);
    Ok(())
}

fn resolve_project_path(
    project_root: &Path,
    relative: &Path,
    reject_symlinks: bool,
) -> CliResult<PathBuf> {
    validate_relative_path(relative)?;
    let mut resolved = project_root.to_path_buf();

    for component in relative.components() {
        let Component::Normal(component) = component else {
            continue;
        };
        resolved.push(component);
        match fs::symlink_metadata(&resolved) {
            Ok(metadata) if metadata.file_type().is_symlink() && reject_symlinks => {
                return Err(unsafe_output(
                    relative,
                    "generated output path contains a symbolic link",
                ));
            }
            Ok(_) => {
                let canonical = fs::canonicalize(&resolved).map_err(|source| CliError::Io {
                    path: resolved.clone(),
                    source,
                })?;
                if !canonical.starts_with(project_root) {
                    return Err(unsafe_output(
                        relative,
                        "path resolves outside the canonical project root",
                    ));
                }
                resolved = canonical;
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(source) => {
                return Err(CliError::Io {
                    path: resolved,
                    source,
                });
            }
        }
    }

    if !resolved.starts_with(project_root) {
        return Err(unsafe_output(
            relative,
            "path resolves outside the canonical project root",
        ));
    }
    Ok(resolved)
}

fn validate_relative_path(path: &Path) -> CliResult<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(unsafe_generated_path(
            path,
            "path must be a non-empty project-relative path without parent traversal",
        ));
    }

    let original = path.to_string_lossy();
    let portable = original.replace('\\', "/");
    if portable != original {
        return Err(unsafe_generated_path(
            path,
            "backslash path separators are not portable",
        ));
    }
    let windows_prefix = portable
        .as_bytes()
        .get(1)
        .is_some_and(|character| *character == b':');
    if portable.starts_with('/')
        || windows_prefix
        || portable.split('/').any(|segment| segment == "..")
        || portable
            .split('/')
            .all(|segment| segment.is_empty() || segment == ".")
    {
        return Err(unsafe_generated_path(
            path,
            "path is not portable or resolves outside its root",
        ));
    }
    Ok(())
}

fn create_directory_chain(project_root: &Path, output_root: &Path) -> CliResult<()> {
    let relative = output_root
        .strip_prefix(project_root)
        .map_err(|_| unsafe_output(output_root, "output is not below the project root"))?;
    let mut current = project_root.to_path_buf();

    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(unsafe_output(
                    output_root,
                    "generated output path contains a symbolic link",
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(unsafe_output(
                    output_root,
                    "generated output parent is not a directory",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|source| CliError::Io {
                    path: current.clone(),
                    source,
                })?;
            }
            Err(source) => {
                return Err(CliError::Io {
                    path: current,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn read_manifest(output_root: &Path) -> CliResult<BTreeSet<PathBuf>> {
    let manifest_path = output_root.join(MANIFEST_NAME);
    let source = match fs::read_to_string(&manifest_path) {
        Ok(source) => source,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let is_empty = fs::read_dir(output_root)
                .map_err(|source| CliError::Io {
                    path: output_root.to_path_buf(),
                    source,
                })?
                .next()
                .is_none();
            if is_empty {
                return Ok(BTreeSet::new());
            }
            return Err(unsafe_output(
                output_root,
                "existing output is not owned by Linguini (manifest missing)",
            ));
        }
        Err(source) => {
            return Err(CliError::Io {
                path: manifest_path,
                source,
            });
        }
    };

    let metadata = fs::symlink_metadata(&manifest_path).map_err(|source| CliError::Io {
        path: manifest_path.clone(),
        source,
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(unsafe_output(
            output_root,
            "ownership manifest is not a regular file",
        ));
    }

    let mut lines = source.lines();
    if lines.next() != Some(MANIFEST_HEADER) {
        return Err(unsafe_output(
            output_root,
            "ownership manifest has an unsupported format",
        ));
    }

    let mut paths = BTreeSet::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let path = PathBuf::from(line);
        validate_relative_path(&path)?;
        if path == Path::new(MANIFEST_NAME) || !paths.insert(path.clone()) {
            return Err(unsafe_output(
                output_root,
                "ownership manifest contains an invalid or duplicate path",
            ));
        }
    }
    Ok(paths)
}

fn validate_existing_ownership(
    output_root: &Path,
    old_paths: &BTreeSet<PathBuf>,
    new_paths: &BTreeSet<PathBuf>,
) -> CliResult<()> {
    for relative in old_paths.union(new_paths) {
        reject_symlink_components(output_root, relative)?;
        let path = output_root.join(relative);
        let exists = path_exists(&path)?;
        if exists && !old_paths.contains(relative) {
            return Err(unsafe_output(
                &path,
                "generated file would overwrite an unowned path",
            ));
        }
        if exists {
            let metadata = fs::symlink_metadata(&path).map_err(|source| CliError::Io {
                path: path.clone(),
                source,
            })?;
            if !metadata.is_file() {
                return Err(unsafe_output(
                    &path,
                    "owned generated path is not a regular file",
                ));
            }
        }
    }
    reject_symlink_components(output_root, Path::new(MANIFEST_NAME))
}

fn reject_symlink_components(root: &Path, relative: &Path) -> CliResult<()> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(unsafe_output(
                    &current,
                    "generated path contains a symbolic link",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => break,
            Err(source) => {
                return Err(CliError::Io {
                    path: current,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn render_manifest(paths: &BTreeSet<PathBuf>) -> String {
    let mut manifest = format!("{MANIFEST_HEADER}\n");
    for path in paths {
        manifest.push_str(&path.to_string_lossy().replace('\\', "/"));
        manifest.push('\n');
    }
    manifest
}

fn write_staged_file(path: &Path, contents: &[u8]) -> CliResult<()> {
    create_parent_directories(path)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|source| CliError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    file.write_all(contents).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.sync_all().map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn create_parent_directories(path: &Path) -> CliResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn rollback(
    output_root: &Path,
    backup_root: &Path,
    installed: &[PathBuf],
    backed_up: &[PathBuf],
    primary: CliError,
) -> CliError {
    let mut failures = Vec::new();
    for relative in installed.iter().rev() {
        let path = output_root.join(relative);
        if let Err(error) = fs::remove_file(&path) {
            if error.kind() != ErrorKind::NotFound {
                failures.push(format!("remove {}: {error}", path.display()));
            }
        }
    }
    for relative in backed_up.iter().rev() {
        let backup = backup_root.join(relative);
        let destination = output_root.join(relative);
        if let Err(error) = create_parent_directories(&destination).and_then(|()| {
            fs::rename(&backup, &destination).map_err(|source| CliError::Io {
                path: destination.clone(),
                source,
            })
        }) {
            failures.push(error.to_string());
        }
    }

    if failures.is_empty() {
        primary
    } else {
        CliError::Diagnostics(format!(
            "{primary}\nrollback also failed:\n{}\n",
            failures.join("\n")
        ))
    }
}

fn prune_empty_owned_directories(
    output_root: &Path,
    old_paths: &BTreeSet<PathBuf>,
    new_paths: &BTreeSet<PathBuf>,
) {
    let mut directories = BTreeSet::new();
    for relative in old_paths.difference(new_paths) {
        let mut current = relative.parent();
        while let Some(parent) = current {
            if parent.as_os_str().is_empty() {
                break;
            }
            directories.insert(output_root.join(parent));
            current = parent.parent();
        }
    }
    let mut directories = directories.into_iter().collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        let _ = fs::remove_dir(directory);
    }
}

fn path_exists(path: &Path) -> CliResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(source) => Err(CliError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn unsafe_output(path: &Path, reason: &str) -> CliError {
    CliError::Diagnostics(format!(
        "unsafe generated output `{}`: {reason}\n",
        path.display()
    ))
}

fn unsafe_generated_path(path: &Path, reason: &str) -> CliError {
    CliError::Diagnostics(format!(
        "codegen returned unsafe output path `{}`: {reason}\n",
        path.display()
    ))
}

struct TransactionDir {
    path: PathBuf,
}

impl TransactionDir {
    fn create(parent: &Path) -> CliResult<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        for _ in 0..128 {
            let id = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(
                ".linguini-transaction-{}-{timestamp}-{id}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(source) => return Err(CliError::Io { path, source }),
            }
        }

        Err(CliError::Diagnostics(
            "could not allocate a unique Linguini output transaction directory\n".to_owned(),
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TransactionDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::{replace_owned_files, GeneratedFile, SafeOutputRoot, MANIFEST_NAME};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn roots(project: &TempDir) -> (PathBuf, PathBuf) {
        let schema = project.path().join("schema");
        let locale = project.path().join("locales");
        fs::create_dir_all(&schema).expect("schema root");
        fs::create_dir_all(&locale).expect("locale root");
        (schema, locale)
    }

    fn output(project: &TempDir, relative: &str) -> SafeOutputRoot {
        let (schema, locale) = roots(project);
        let schema = schema
            .strip_prefix(project.path())
            .expect("relative schema");
        let locale = locale
            .strip_prefix(project.path())
            .expect("relative locale");
        SafeOutputRoot::new(project.path(), Path::new(relative), &[schema, locale])
            .expect("safe output")
    }

    #[test]
    fn rejects_absolute_parent_and_source_overlapping_roots() {
        let project = TempDir::new().expect("project");
        let (schema, locale) = roots(&project);
        let protected = [
            schema.strip_prefix(project.path()).expect("schema"),
            locale.strip_prefix(project.path()).expect("locale"),
        ];

        for candidate in [
            "/tmp/generated",
            "../generated",
            r"generated\outside",
            ".",
            "schema/out",
            "locales",
        ] {
            assert!(
                SafeOutputRoot::new(project.path(), Path::new(candidate), &protected).is_err(),
                "{candidate}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let project = TempDir::new().expect("project");
        let outside = TempDir::new().expect("outside");
        let (schema, locale) = roots(&project);
        symlink(outside.path(), project.path().join("generated")).expect("symlink");

        let result = SafeOutputRoot::new(
            project.path(),
            Path::new("generated"),
            &[
                schema.strip_prefix(project.path()).expect("schema"),
                locale.strip_prefix(project.path()).expect("locale"),
            ],
        );

        assert!(result.is_err());
        assert!(!outside.path().join(MANIFEST_NAME).exists());
    }

    #[test]
    fn refuses_unowned_existing_output() {
        let project = TempDir::new().expect("project");
        let output = output(&project, "generated");
        fs::create_dir_all(output.path()).expect("output root");
        fs::write(output.path().join("user.txt"), "keep").expect("user file");

        let result = replace_owned_files(
            &output,
            &[GeneratedFile {
                path: PathBuf::from("index.js"),
                contents: "export {};\n",
            }],
        );

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(output.path().join("user.txt")).expect("user file"),
            "keep"
        );
    }

    #[test]
    fn replaces_only_manifest_owned_files() {
        let project = TempDir::new().expect("project");
        let output = output(&project, "generated");
        replace_owned_files(
            &output,
            &[
                GeneratedFile {
                    path: PathBuf::from("index.js"),
                    contents: "export const version = 1;\n",
                },
                GeneratedFile {
                    path: PathBuf::from("old/stale.js"),
                    contents: "stale\n",
                },
            ],
        )
        .expect("initial generation");
        fs::write(output.path().join("user.txt"), "keep").expect("user file");

        replace_owned_files(
            &output,
            &[GeneratedFile {
                path: PathBuf::from("index.js"),
                contents: "export const version = 2;\n",
            }],
        )
        .expect("replacement");

        assert_eq!(
            fs::read_to_string(output.path().join("index.js")).expect("index"),
            "export const version = 2;\n"
        );
        assert!(!output.path().join("old/stale.js").exists());
        assert_eq!(
            fs::read_to_string(output.path().join("user.txt")).expect("user file"),
            "keep"
        );
        assert!(output.path().join(MANIFEST_NAME).is_file());
    }

    #[test]
    fn refuses_collision_with_unowned_file() {
        let project = TempDir::new().expect("project");
        let output = output(&project, "generated");
        replace_owned_files(
            &output,
            &[GeneratedFile {
                path: PathBuf::from("index.js"),
                contents: "export {};\n",
            }],
        )
        .expect("initial generation");
        fs::create_dir_all(output.path().join("nested")).expect("nested");
        fs::write(output.path().join("nested/user.js"), "keep").expect("user file");

        let result = replace_owned_files(
            &output,
            &[
                GeneratedFile {
                    path: PathBuf::from("index.js"),
                    contents: "export const changed = true;\n",
                },
                GeneratedFile {
                    path: PathBuf::from("nested/user.js"),
                    contents: "overwrite\n",
                },
            ],
        );

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(output.path().join("index.js")).expect("old index"),
            "export {};\n"
        );
        assert_eq!(
            fs::read_to_string(output.path().join("nested/user.js")).expect("user file"),
            "keep"
        );
    }
}
