use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub type CldrCacheResult<T> = Result<T, CldrCacheError>;

const DATA_DIR: &str = "data";
const MANIFEST_FILE: &str = "manifest.txt";
pub(crate) const PLURALS_FILE: &str = "data/common/supplemental/plurals.json";

#[derive(Debug)]
pub enum CldrCacheError {
    Io { path: PathBuf, source: io::Error },
    MissingCache(PathBuf),
    Parse { path: PathBuf, message: String },
}

impl Display for CldrCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::MissingCache(path) => write!(
                f,
                "missing CLDR cache at `{}`; run `linguini cldr fetch <cldr-json-dir>`",
                path.display()
            ),
            Self::Parse { path, message } => write!(f, "{}: {message}", path.display()),
        }
    }
}

impl std::error::Error for CldrCacheError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStatus {
    pub root: PathBuf,
    pub cache_dir_exists: bool,
    pub manifest_exists: bool,
    pub data_dir_exists: bool,
    pub plurals_file_exists: bool,
    pub problems: Vec<String>,
}

impl CacheStatus {
    pub fn is_usable(&self) -> bool {
        self.cache_dir_exists
            && self.manifest_exists
            && self.data_dir_exists
            && self.plurals_file_exists
            && self.problems.is_empty()
    }
}

pub fn cache_root(project_root: &Path, configured_cache_path: &str) -> PathBuf {
    let path = Path::new(configured_cache_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

pub fn inspect_cache(root: impl AsRef<Path>) -> CacheStatus {
    let root = root.as_ref().to_path_buf();
    let manifest = root.join(MANIFEST_FILE);
    let data_dir = root.join(DATA_DIR);
    let plurals_file = root.join(PLURALS_FILE);
    let mut problems = Vec::new();

    if manifest.exists() && fs::metadata(&manifest).is_err() {
        problems.push(format!("manifest is not readable: {}", manifest.display()));
    }

    if plurals_file.exists() && fs::metadata(&plurals_file).is_err() {
        problems.push(format!(
            "plural rules file is not readable: {}",
            plurals_file.display()
        ));
    }

    CacheStatus {
        cache_dir_exists: root.is_dir(),
        manifest_exists: manifest.is_file(),
        data_dir_exists: data_dir.is_dir(),
        plurals_file_exists: plurals_file.is_file(),
        root,
        problems,
    }
}

pub fn require_offline_cache(root: impl AsRef<Path>) -> CldrCacheResult<CacheStatus> {
    let status = inspect_cache(root.as_ref());
    if status.is_usable() {
        Ok(status)
    } else {
        Err(CldrCacheError::MissingCache(status.root))
    }
}

pub fn fetch_cldr_from_dir(
    source_dir: impl AsRef<Path>,
    cache_root: impl AsRef<Path>,
) -> CldrCacheResult<CacheStatus> {
    fetch_cldr_from_dir_for_locales(source_dir, cache_root, std::iter::empty::<&str>())
}

pub fn fetch_cldr_from_dir_for_locales<'a>(
    source_dir: impl AsRef<Path>,
    cache_root: impl AsRef<Path>,
    locales: impl IntoIterator<Item = &'a str>,
) -> CldrCacheResult<CacheStatus> {
    let source_dir = source_dir.as_ref();
    let cache_root = cache_root.as_ref();
    let data_dir = cache_root.join(DATA_DIR);
    let locales: Vec<_> = locales.into_iter().collect();

    create_dir_all(cache_root)?;
    create_dir_all(&data_dir)?;
    copy_required_files(source_dir, &data_dir, &locales)?;

    let manifest = format!(
        "source={}\nfetched_at_unix={}\n",
        source_dir.display(),
        unix_seconds()
    );
    write_file(cache_root.join(MANIFEST_FILE), &manifest)?;

    require_offline_cache(cache_root)
}

fn copy_required_files(source: &Path, destination: &Path, locales: &[&str]) -> CldrCacheResult<()> {
    copy_relative_file(source, destination, "common/supplemental/plurals.json")?;

    if locales.is_empty() {
        copy_required_locale_files_for_all_locales(source, destination)?;
    } else {
        for locale in locales {
            copy_required_locale_files(source, destination, locale)?;
        }
    }

    Ok(())
}

fn copy_required_locale_files_for_all_locales(
    source: &Path,
    destination: &Path,
) -> CldrCacheResult<()> {
    let main = source.join("common/main");
    if !main.exists() {
        return Ok(());
    }

    for entry in read_dir(&main)? {
        let entry = entry.map_err(|error| CldrCacheError::Io {
            path: main.clone(),
            source: error,
        })?;
        if entry.path().is_dir() {
            let locale = entry.file_name().to_string_lossy().into_owned();
            copy_required_locale_files(source, destination, &locale)?;
        }
    }

    Ok(())
}

fn copy_required_locale_files(
    source: &Path,
    destination: &Path,
    locale: &str,
) -> CldrCacheResult<()> {
    copy_relative_file(
        source,
        destination,
        format!("common/main/{locale}/numbers.json"),
    )?;
    copy_relative_file(
        source,
        destination,
        format!("common/main/{locale}/ca-gregorian.json"),
    )
}

fn copy_relative_file(
    source_root: &Path,
    destination_root: &Path,
    relative_path: impl AsRef<Path>,
) -> CldrCacheResult<()> {
    let relative_path = relative_path.as_ref();
    let source = source_root.join(relative_path);
    let destination = destination_root.join(relative_path);

    if let Some(parent) = destination.parent() {
        create_dir_all(parent)?;
    }

    fs::copy(&source, &destination)
        .map(|_| ())
        .map_err(|error| CldrCacheError::Io {
            path: source,
            source: error,
        })
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn read_dir(path: &Path) -> CldrCacheResult<fs::ReadDir> {
    fs::read_dir(path).map_err(|source| CldrCacheError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn create_dir_all(path: impl AsRef<Path>) -> CldrCacheResult<()> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|source| CldrCacheError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: impl AsRef<Path>, contents: &str) -> CldrCacheResult<()> {
    let path = path.as_ref();
    fs::write(path, contents).map_err(|source| CldrCacheError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        cache_root, fetch_cldr_from_dir, fetch_cldr_from_dir_for_locales, inspect_cache,
        require_offline_cache,
    };
    use linguini_test_support::temp_project_dir;
    use std::fs;

    #[test]
    fn cache_root_resolves_relative_path_under_project() {
        let project = temp_project_dir("cldr_cache_root");
        let root = cache_root(project.path(), ".linguini/cache");

        assert_eq!(root, project.path().join(".linguini/cache"));
    }

    #[test]
    fn cache_status_reports_missing_cache() {
        let project = temp_project_dir("cldr_missing_cache");
        let status = inspect_cache(project.path().join(".linguini/cache"));

        assert!(!status.is_usable());
        assert!(!status.cache_dir_exists);
    }

    #[test]
    fn fetch_from_dir_populates_offline_cache() {
        let project = temp_project_dir("cldr_fetch_from_dir");
        write_required_source_files(project.path(), "ru");

        let cache = project.path().join(".linguini/cache");
        let status = fetch_cldr_from_dir(project.path().join("source"), &cache).expect("fetch");

        assert!(status.is_usable());
        assert!(require_offline_cache(&cache).is_ok());
    }

    #[test]
    fn fetch_for_locales_copies_only_required_json_files() {
        let project = temp_project_dir("cldr_fetch_required_only");
        write_required_source_files(project.path(), "ru");
        let extra_dir = project.path().join("source/common/main/ru");
        fs::write(extra_dir.join("characters.json"), "{}\n").expect("extra data");

        let cache = project.path().join(".linguini/cache");
        fetch_cldr_from_dir_for_locales(project.path().join("source"), &cache, ["ru"])
            .expect("fetch");

        assert!(cache
            .join("data/common/supplemental/plurals.json")
            .is_file());
        assert!(cache.join("data/common/main/ru/numbers.json").is_file());
        assert!(cache
            .join("data/common/main/ru/ca-gregorian.json")
            .is_file());
        assert!(!cache.join("data/common/main/ru/characters.json").exists());
    }

    fn write_required_source_files(root: &std::path::Path, locale: &str) {
        let supplemental = root.join("source/common/supplemental");
        let main = root.join("source/common/main").join(locale);
        fs::create_dir_all(&supplemental).expect("supplemental dir");
        fs::create_dir_all(&main).expect("main dir");
        fs::write(supplemental.join("plurals.json"), "{}\n").expect("plural data");
        fs::write(main.join("numbers.json"), "{}\n").expect("numbers data");
        fs::write(main.join("ca-gregorian.json"), "{}\n").expect("calendar data");
    }
}
