use std::path::{Path, PathBuf};

pub fn fixture_path(relative: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(relative)
}

#[cfg(test)]
mod tests {
    use super::fixture_path;

    #[test]
    fn fixture_path_points_at_repository_fixtures() {
        let path = fixture_path("golden");
        assert!(path.ends_with("tests/fixtures/golden"));
    }
}
