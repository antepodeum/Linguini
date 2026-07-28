//! Shared, fallible test filesystem helpers.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use tempfile::{Builder, TempDir};

/// Optional runtime override for the directory containing repository fixtures.
pub const FIXTURE_ROOT_ENV: &str = "LINGUINI_FIXTURE_ROOT";

const TEMP_PROJECT_PREFIX: &str = "linguini-";
const MAX_PROJECT_NAME_LEN: usize = 64;

/// Finds the fixture root in a packaged crate or a repository checkout.
///
/// [`FIXTURE_ROOT_ENV`] takes precedence. Without it, a package-local
/// `tests/fixtures` directory is preferred, followed by the nearest ancestor
/// containing `tests/fixtures`.
pub fn fixture_root() -> io::Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let override_root = env::var_os(FIXTURE_ROOT_ENV);
    discover_fixture_root(manifest_dir, override_root.as_deref().map(Path::new))
}

/// Returns a path below [`fixture_root`].
///
/// Absolute paths, parent components, and platform-dependent separators are
/// rejected so a caller cannot escape the fixture root.
pub fn fixture_path(relative: impl AsRef<Path>) -> io::Result<PathBuf> {
    let relative = relative.as_ref();
    validate_fixture_relative_path(relative)?;
    Ok(fixture_root()?.join(relative))
}

/// An isolated temporary project removed when dropped.
#[derive(Debug)]
pub struct TempProject {
    directory: TempDir,
}

impl TempProject {
    /// Returns the temporary project's directory.
    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    /// Removes the temporary project and reports cleanup failures.
    ///
    /// Dropping the project still performs best-effort cleanup. Call `close`
    /// when a test must verify that cleanup succeeded.
    pub fn close(self) -> io::Result<()> {
        self.directory.close()
    }
}

/// Atomically creates a uniquely named temporary project.
///
/// `name` is a diagnostic label, not a path. It must be 1–64 ASCII
/// alphanumeric, `_`, or `-` characters. Creation errors are returned.
pub fn temp_project_dir(name: &str) -> io::Result<TempProject> {
    temp_project_dir_in(env::temp_dir(), name)
}

/// Atomically creates a uniquely named temporary project below `root`.
///
/// The root must already exist. This variant makes failure behavior testable
/// and lets callers select a controlled temporary filesystem.
pub fn temp_project_dir_in(root: impl AsRef<Path>, name: &str) -> io::Result<TempProject> {
    validate_project_name(name)?;
    let prefix = format!("{TEMP_PROJECT_PREFIX}{name}-");
    let directory = Builder::new().prefix(&prefix).tempdir_in(root.as_ref())?;
    Ok(TempProject { directory })
}

fn validate_project_name(name: &str) -> io::Result<()> {
    let valid = !name.is_empty()
        && name.len() <= MAX_PROJECT_NAME_LEN
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));

    if valid {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "temporary project name must be 1-64 ASCII alphanumeric, `_`, or `-` characters",
        ))
    }
}

fn validate_fixture_relative_path(path: &Path) -> io::Result<()> {
    if path.is_absolute() {
        return Err(invalid_fixture_path(path));
    }

    for component in path.components() {
        match component {
            Component::Normal(value) if portable_fixture_component(value) => {}
            _ => return Err(invalid_fixture_path(path)),
        }
    }

    Ok(())
}

fn portable_fixture_component(component: &OsStr) -> bool {
    !component.to_string_lossy().contains('\\')
}

fn invalid_fixture_path(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "fixture path must contain only portable normal components: {}",
            path.display()
        ),
    )
}

fn discover_fixture_root(manifest_dir: &Path, override_root: Option<&Path>) -> io::Result<PathBuf> {
    if let Some(root) = override_root {
        return canonical_fixture_root(root, "fixture root override");
    }

    let local = manifest_dir.join("tests/fixtures");
    if local.is_dir() {
        return canonical_fixture_root(&local, "package-local fixture root");
    }

    for ancestor in manifest_dir.ancestors().skip(1) {
        let candidate = ancestor.join("tests/fixtures");
        if candidate.is_dir() {
            return canonical_fixture_root(&candidate, "ancestor fixture root");
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "fixture root not found; set {FIXTURE_ROOT_ENV} or provide tests/fixtures near {}",
            manifest_dir.display()
        ),
    ))
}

fn canonical_fixture_root(path: &Path, source: &str) -> io::Result<PathBuf> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("{source} `{}` is unavailable: {error}", path.display()),
        )
    })?;

    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{source} `{}` is not a directory", path.display()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        discover_fixture_root, fixture_path, fixture_root, temp_project_dir, temp_project_dir_in,
    };
    use std::collections::HashSet;
    use std::fs;
    use std::io;
    use std::path::Path;

    #[test]
    fn fixture_path_points_at_existing_repository_fixtures() -> io::Result<()> {
        let path = fixture_path("golden")?;
        assert!(path.is_dir());
        assert!(path.starts_with(fixture_root()?));
        Ok(())
    }

    #[test]
    fn package_local_fixture_root_works_without_monorepo_layout() -> io::Result<()> {
        let package = tempfile::tempdir()?;
        let expected = package.path().join("tests/fixtures");
        fs::create_dir_all(&expected)?;

        let actual = discover_fixture_root(package.path(), None)?;

        assert_eq!(actual, fs::canonicalize(expected)?);
        Ok(())
    }

    #[test]
    fn fixture_root_override_has_precedence() -> io::Result<()> {
        let package = tempfile::tempdir()?;
        let local = package.path().join("tests/fixtures");
        let override_root = tempfile::tempdir()?;
        fs::create_dir_all(local)?;

        let actual = discover_fixture_root(package.path(), Some(override_root.path()))?;

        assert_eq!(actual, fs::canonicalize(override_root.path())?);
        Ok(())
    }

    #[test]
    fn missing_fixture_root_returns_not_found() -> io::Result<()> {
        let package = tempfile::tempdir()?;
        let manifest = package.path().join("isolated/package");
        fs::create_dir_all(&manifest)?;

        let error = discover_fixture_root(&manifest, None).expect_err("missing fixture root");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        Ok(())
    }

    #[test]
    fn fixture_path_rejects_escape_attempts() {
        for path in [
            Path::new("../golden"),
            Path::new("golden/../../outside"),
            Path::new("./golden"),
            Path::new(r"golden\..\outside"),
            Path::new("/tmp/outside"),
        ] {
            let error = fixture_path(path).expect_err("unsafe path must fail");
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput, "{path:?}");
        }
    }

    #[test]
    fn temp_project_dir_creates_directory() -> io::Result<()> {
        let project = temp_project_dir("creates_directory")?;
        assert!(project.path().is_dir());
        assert!(project
            .path()
            .file_name()
            .expect("temporary directory name")
            .to_string_lossy()
            .starts_with("linguini-creates_directory-"));
        Ok(())
    }

    #[test]
    fn same_label_always_creates_isolated_directories() -> io::Result<()> {
        let root = tempfile::tempdir()?;
        let mut projects = Vec::new();
        let mut paths = HashSet::new();

        for _ in 0..128 {
            let project = temp_project_dir_in(root.path(), "collision")?;
            assert!(paths.insert(project.path().to_owned()));
            projects.push(project);
        }

        assert_eq!(paths.len(), 128);
        assert!(paths.iter().all(|path| path.is_dir()));
        drop(projects);
        assert!(paths.iter().all(|path| !path.exists()));
        Ok(())
    }

    #[test]
    fn stale_directory_is_never_reused() -> io::Result<()> {
        let root = tempfile::tempdir()?;
        let stale = root.path().join("linguini-collision-stale");
        fs::create_dir(&stale)?;
        fs::write(stale.join("sentinel"), "keep")?;

        let project = temp_project_dir_in(root.path(), "collision")?;

        assert_ne!(project.path(), stale);
        assert_eq!(fs::read_to_string(stale.join("sentinel"))?, "keep");
        Ok(())
    }

    #[test]
    fn unsafe_project_names_are_rejected_without_filesystem_changes() -> io::Result<()> {
        let root = tempfile::tempdir()?;
        let before = fs::read_dir(root.path())?.count();

        for name in [
            "",
            ".",
            "..",
            "../outside",
            "../../outside",
            "nested/name",
            r"nested\name",
            "/absolute",
            "white space",
            "colon:name",
        ] {
            let error = temp_project_dir_in(root.path(), name).expect_err("unsafe name must fail");
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput, "{name:?}");
        }

        assert_eq!(fs::read_dir(root.path())?.count(), before);
        Ok(())
    }

    #[test]
    fn creation_failures_are_returned() -> io::Result<()> {
        let root = tempfile::tempdir()?;
        let missing = root.path().join("missing");

        let error =
            temp_project_dir_in(&missing, "creation_error").expect_err("missing root must fail");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        Ok(())
    }

    #[test]
    fn drop_removes_project_tree() -> io::Result<()> {
        let path = {
            let project = temp_project_dir("drop_cleanup")?;
            fs::create_dir(project.path().join("nested"))?;
            fs::write(project.path().join("nested/file"), "temporary")?;
            project.path().to_owned()
        };

        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn close_removes_project_tree_and_reports_result() -> io::Result<()> {
        let project = temp_project_dir("explicit_cleanup")?;
        let path = project.path().to_owned();
        fs::write(project.path().join("file"), "temporary")?;

        project.close()?;

        assert!(!path.exists());
        Ok(())
    }
}
