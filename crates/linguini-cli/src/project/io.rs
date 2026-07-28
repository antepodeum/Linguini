use crate::{CliError, CliResult};
use linguini_analyzer::{render_diagnostics_with_color, Diagnostic};
use linguini_config::{parse_config, DEFAULT_CONFIG_FILE};
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Path;

pub fn init_project(root: &Path) -> CliResult<String> {
    let schema_dir = root.join("schema");
    let locale_dir = root.join("locales");
    let schema_created = ensure_directory(&schema_dir)?;
    let locale_created = ensure_directory(&locale_dir)?;

    let config_path = root.join(DEFAULT_CONFIG_FILE);
    let config_created = if !config_path.exists() {
        write_file(&config_path, default_config())?;
        true
    } else {
        false
    };

    Ok(format!(
        "{} {}\n{} schema\n{} locales\n",
        creation_status(config_created),
        DEFAULT_CONFIG_FILE,
        creation_status(schema_created),
        creation_status(locale_created),
    ))
}

fn ensure_directory(path: &Path) -> CliResult<bool> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(false),
        Ok(_) => Err(CliError::Diagnostics(format!(
            "cannot initialize directory because a file exists at `{}`\n",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_dir_all(path)?;
            Ok(true)
        }
        Err(source) => Err(CliError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn creation_status(created: bool) -> &'static str {
    if created {
        "created"
    } else {
        "kept existing"
    }
}

pub(crate) fn read_project_config(root: &Path) -> CliResult<linguini_config::LinguiniConfig> {
    let config_path = root.join(DEFAULT_CONFIG_FILE);
    let source = read_file(&config_path)?;
    Ok(parse_config(&source)?)
}

fn default_config() -> &'static str {
    r#"[project]
name = "linguini-app"
default_locale = "en"
locales = ["en"]

[paths]
schema = "schema"
locale = "locales"

[targets.ts]
out = "src/generated/linguini"
declaration = true
gitignore = true
"#
}

pub(crate) fn path_for_output(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn create_dir_all(path: &Path) -> CliResult<()> {
    fs::create_dir_all(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn read_file(path: &Path) -> CliResult<String> {
    fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn write_file(path: &Path, contents: &str) -> CliResult<()> {
    atomic_write_file(path, contents.as_bytes())
}

pub(crate) fn atomic_write_file(path: &Path, contents: &[u8]) -> CliResult<()> {
    let parent = path.parent().ok_or_else(|| {
        CliError::Diagnostics(format!(
            "cannot atomically write `{}` without a parent\n",
            path.display()
        ))
    })?;
    create_dir_all(parent)?;
    let existing_permissions = match fs::metadata(path) {
        Ok(metadata) => Some(metadata.permissions()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(source) => {
            return Err(CliError::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };

    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|source| CliError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    temporary
        .write_all(contents)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|source| CliError::Io {
            path: temporary.path().to_path_buf(),
            source,
        })?;
    if let Some(permissions) = existing_permissions {
        fs::set_permissions(temporary.path(), permissions).map_err(|source| CliError::Io {
            path: temporary.path().to_path_buf(),
            source,
        })?;
    }
    temporary
        .persist(path)
        .map(|_| ())
        .map_err(|error| CliError::Io {
            path: path.to_path_buf(),
            source: error.error,
        })
}

pub(crate) fn atomic_write_file_if_unchanged(
    path: &Path,
    expected: &str,
    contents: &str,
) -> CliResult<()> {
    let current = read_file(path)?;
    if current != expected {
        return Err(CliError::Diagnostics(format!(
            "refusing to overwrite concurrently changed file `{}`\n",
            path.display()
        )));
    }
    atomic_write_file(path, contents.as_bytes())
}

pub(crate) fn create_new_file(path: &Path, contents: &str) -> CliResult<()> {
    let parent = path.parent().ok_or_else(|| {
        CliError::Diagnostics(format!(
            "cannot create `{}` without a parent\n",
            path.display()
        ))
    })?;
    create_dir_all(parent)?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|source| CliError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    if let Err(source) = file
        .write_all(contents.as_bytes())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let cleanup = fs::remove_file(path);
        return match cleanup {
            Ok(()) => Err(CliError::Io {
                path: path.to_path_buf(),
                source,
            }),
            Err(cleanup) if cleanup.kind() == ErrorKind::NotFound => Err(CliError::Io {
                path: path.to_path_buf(),
                source,
            }),
            Err(cleanup) => Err(CliError::Diagnostics(format!(
                "{}: {source}\nfailed to remove partial file: {cleanup}\n",
                path.display()
            ))),
        };
    }
    Ok(())
}

pub(crate) fn resolve_project_file(root: &Path, requested: &Path) -> CliResult<std::path::PathBuf> {
    if !requested.is_absolute()
        && requested
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(outside_project_error(requested));
    }

    let canonical_root = fs::canonicalize(root).map_err(|source| CliError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    };
    reject_symlink_components(root, &candidate)?;
    let canonical = fs::canonicalize(&candidate).map_err(|source| CliError::Io {
        path: candidate.clone(),
        source,
    })?;
    if !canonical.starts_with(&canonical_root) || !canonical.is_file() {
        return Err(outside_project_error(requested));
    }
    Ok(canonical)
}

fn reject_symlink_components(root: &Path, candidate: &Path) -> CliResult<()> {
    let relative = candidate
        .strip_prefix(root)
        .map_err(|_| outside_project_error(candidate))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current).map_err(|source| CliError::Io {
            path: current.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(CliError::Diagnostics(format!(
                "refusing to modify symbolic-link target `{}`\n",
                current.display()
            )));
        }
    }
    Ok(())
}

fn outside_project_error(path: &Path) -> CliError {
    CliError::Diagnostics(format!(
        "refusing to modify file outside the project root: `{}`\n",
        path.display()
    ))
}

pub(crate) fn render_parse_errors(
    root: &Path,
    path: &Path,
    source: &str,
    note: &str,
    errors: Vec<linguini_syntax::ParseError>,
) -> String {
    let diagnostics: Vec<_> = errors
        .into_iter()
        .map(|error| Diagnostic::error(error.message, error.span).with_note(note))
        .collect();

    render_file_diagnostics(root, path, source, &diagnostics)
}

pub(crate) fn render_file_diagnostics(
    root: &Path,
    path: &Path,
    source: &str,
    diagnostics: &[Diagnostic],
) -> String {
    let relative_path = path_for_output(root, path);
    render_diagnostics_with_color(&relative_path, source, diagnostics, false).unwrap_or_else(
        |error| format!("failed to render diagnostics for {relative_path}: {error}"),
    )
}
