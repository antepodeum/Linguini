use std::path::{Path, PathBuf};
use std::{env, fs};

pub fn fixture_path(relative: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(relative)
}

pub struct TempProject {
    path: PathBuf,
}

impl TempProject {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn temp_project_dir(name: &str) -> TempProject {
    let mut path = env::temp_dir();
    path.push(format!(
        "linguini-{name}-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&path).expect("create temporary project directory");
    TempProject { path }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::{fixture_path, temp_project_dir};

    #[test]
    fn fixture_path_points_at_repository_fixtures() {
        let path = fixture_path("golden");
        assert!(path.ends_with("tests/fixtures/golden"));
    }

    #[test]
    fn temp_project_dir_creates_directory() {
        let project = temp_project_dir("temp_project_dir_creates_directory");
        assert!(project.path().is_dir());
    }
}
