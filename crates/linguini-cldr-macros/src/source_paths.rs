use crate::sha256::{hex, Sha256};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) const SOURCE_REPOSITORY: &str = "https://github.com/unicode-org/cldr-json.git";
pub(crate) const SOURCE_REF: &str = "48.2.0";
pub(crate) const SOURCE_COMMIT: &str = "bb334e8d6250c9363e957e131bf7e6d08ec72f91";
pub(crate) const SOURCE_GIT_TREE: &str = "b297a9501ae136e59d006f0497204524a5478cc9";
pub(crate) const SOURCE_TREE_SHA256: &str =
    "56a882b24a6f978d514c6940afd18f96d9b9078b7d967b3f6b4d76b5516387b5";
pub(crate) const CLDR_VERSION: &str = "48.2.0";
pub(crate) const UNICODE_VERSION: &str = "16.0.0";
pub(crate) const EXPECTED_LOCALE_COUNT: usize = 766;

const ALIASES_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/aliases.json";
const PARENT_LOCALES_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/parentLocales.json";
const LIKELY_SUBTAGS_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/likelySubtags.json";
const BCP47_RELATIVE_PATH: &str = "cldr-json/cldr-bcp47/bcp47";
const PLURALS_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/plurals.json";
const CURRENCY_DATA_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/currencyData.json";
const LAYOUT_MAIN_RELATIVE_PATH: &str = "cldr-json/cldr-misc-full/main";
const NUMBERS_MAIN_RELATIVE_PATH: &str = "cldr-json/cldr-numbers-full/main";
const DATES_MAIN_RELATIVE_PATH: &str = "cldr-json/cldr-dates-full/main";
const PACKAGE_RELATIVE_PATHS: [&str; 5] = [
    "cldr-json/cldr-bcp47/package.json",
    "cldr-json/cldr-core/package.json",
    "cldr-json/cldr-numbers-full/package.json",
    "cldr-json/cldr-dates-full/package.json",
    "cldr-json/cldr-misc-full/package.json",
];

pub(crate) struct CldrSource {
    root: PathBuf,
    aliases: PathBuf,
    parent_locales: PathBuf,
    likely_subtags: PathBuf,
    bcp47: PathBuf,
    plurals: PathBuf,
    currency_data: PathBuf,
    layout_main: PathBuf,
    numbers_main: PathBuf,
    dates_main: PathBuf,
    locales: Vec<String>,
    input_files: Vec<PathBuf>,
    source_tree_sha256: String,
}

impl CldrSource {
    pub(crate) fn open(root: &Path) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|error| format!("{}: {error}", root.display()))?;
        if !root.is_dir() {
            return Err(format!(
                "CLDR source root is not a directory: {}",
                root.display()
            ));
        }

        verify_checkout_identity(&root)?;
        for package in PACKAGE_RELATIVE_PATHS {
            verify_package_identity(&root.join(package))?;
        }

        let aliases = checked_file(&root, ALIASES_RELATIVE_PATH)?;
        let parent_locales = checked_file(&root, PARENT_LOCALES_RELATIVE_PATH)?;
        let likely_subtags = checked_file(&root, LIKELY_SUBTAGS_RELATIVE_PATH)?;
        let bcp47 = checked_dir(&root, BCP47_RELATIVE_PATH)?;
        let plurals = checked_file(&root, PLURALS_RELATIVE_PATH)?;
        let currency_data = checked_file(&root, CURRENCY_DATA_RELATIVE_PATH)?;
        let layout_main = checked_dir(&root, LAYOUT_MAIN_RELATIVE_PATH)?;
        let numbers_main = checked_dir(&root, NUMBERS_MAIN_RELATIVE_PATH)?;
        let dates_main = checked_dir(&root, DATES_MAIN_RELATIVE_PATH)?;

        let number_locales = locale_directories(&numbers_main)?;
        let date_locales = locale_directories(&dates_main)?;
        let layout_locales = locale_directories(&layout_main)?;
        if number_locales != date_locales || number_locales != layout_locales {
            return Err(locale_set_mismatch(
                &number_locales,
                &date_locales,
                &layout_locales,
            ));
        }
        if number_locales.len() != EXPECTED_LOCALE_COUNT {
            return Err(format!(
                "CLDR locale manifest has {} locales, expected {EXPECTED_LOCALE_COUNT}",
                number_locales.len()
            ));
        }
        let locales: Vec<_> = number_locales.into_iter().collect();

        let mut input_files = Vec::with_capacity(25 + locales.len() * 3);
        input_files.push(aliases.clone());
        input_files.push(parent_locales.clone());
        input_files.push(likely_subtags.clone());
        input_files.extend(json_files(&bcp47)?);
        input_files.push(plurals.clone());
        input_files.push(currency_data.clone());
        for package in PACKAGE_RELATIVE_PATHS {
            input_files.push(checked_file(&root, package)?);
        }
        for locale in &locales {
            input_files.push(checked_file(
                &root,
                &format!("{NUMBERS_MAIN_RELATIVE_PATH}/{locale}/numbers.json"),
            )?);
            input_files.push(checked_file(
                &root,
                &format!("{DATES_MAIN_RELATIVE_PATH}/{locale}/ca-gregorian.json"),
            )?);
            input_files.push(checked_file(
                &root,
                &format!("{LAYOUT_MAIN_RELATIVE_PATH}/{locale}/layout.json"),
            )?);
        }
        input_files.sort();

        let source_tree_sha256 = hash_source_tree(&root, &input_files)?;
        if source_tree_sha256 != SOURCE_TREE_SHA256 {
            return Err(format!(
                "CLDR source tree SHA-256 mismatch: got {source_tree_sha256}, expected {SOURCE_TREE_SHA256}"
            ));
        }

        Ok(Self {
            root,
            aliases,
            parent_locales,
            likely_subtags,
            bcp47,
            plurals,
            currency_data,
            layout_main,
            numbers_main,
            dates_main,
            locales,
            input_files,
            source_tree_sha256,
        })
    }

    pub(crate) fn aliases(&self) -> &Path {
        &self.aliases
    }

    pub(crate) fn parent_locales(&self) -> &Path {
        &self.parent_locales
    }

    pub(crate) fn likely_subtags(&self) -> &Path {
        &self.likely_subtags
    }

    pub(crate) fn bcp47(&self) -> &Path {
        &self.bcp47
    }

    pub(crate) fn plurals(&self) -> &Path {
        &self.plurals
    }

    pub(crate) fn currency_data(&self) -> &Path {
        &self.currency_data
    }

    pub(crate) fn layout_main(&self) -> &Path {
        &self.layout_main
    }

    pub(crate) fn numbers_main(&self) -> &Path {
        &self.numbers_main
    }

    pub(crate) fn dates_main(&self) -> &Path {
        &self.dates_main
    }

    pub(crate) fn locales(&self) -> &[String] {
        &self.locales
    }

    pub(crate) fn input_file_count(&self) -> usize {
        self.input_files.len()
    }

    pub(crate) fn input_bytes(&self) -> Result<u64, String> {
        self.input_files.iter().try_fold(0u64, |total, path| {
            let bytes = fs::metadata(path)
                .map_err(|error| format!("{}: {error}", path.display()))?
                .len();
            total
                .checked_add(bytes)
                .ok_or_else(|| "CLDR input size exceeds u64".to_owned())
        })
    }

    pub(crate) fn source_tree_sha256(&self) -> &str {
        &self.source_tree_sha256
    }

    pub(crate) fn verify_unchanged(&self) -> Result<(), String> {
        let actual = hash_source_tree(&self.root, &self.input_files)?;
        if actual == self.source_tree_sha256 {
            Ok(())
        } else {
            Err(format!(
                "CLDR source tree changed during generation: got {actual}, started with {}",
                self.source_tree_sha256
            ))
        }
    }
}

fn verify_checkout_identity(root: &Path) -> Result<(), String> {
    let git_metadata = root.join(".git");
    let git_dir = if git_metadata.is_dir() {
        git_metadata
    } else {
        let source = fs::read_to_string(&git_metadata)
            .map_err(|error| format!("{}: {error}", git_metadata.display()))?;
        let relative = source
            .trim()
            .strip_prefix("gitdir: ")
            .ok_or_else(|| format!("{}: invalid gitdir file", git_metadata.display()))?;
        root.join(relative)
    };
    let head_path = git_dir.join("HEAD");
    let head = fs::read_to_string(&head_path)
        .map_err(|error| format!("{}: {error}", head_path.display()))?;
    let head = head.trim();
    if head != SOURCE_COMMIT {
        return Err(format!(
            "CLDR checkout HEAD is `{head}`, expected full commit `{SOURCE_COMMIT}`"
        ));
    }
    if head.len() != 40 || !head.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("CLDR checkout HEAD is not a full 40-digit commit identity".to_owned());
    }
    Ok(())
}

fn verify_package_identity(path: &Path) -> Result<(), String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let value: Value =
        serde_json::from_str(&source).map_err(|error| format!("{}: {error}", path.display()))?;
    require_json_string(&value, "version", CLDR_VERSION, path)?;
    require_json_string(&value, "cldrVersion", "48", path)?;
    require_json_string(&value, "unicodeVersion", UNICODE_VERSION, path)?;
    Ok(())
}

fn require_json_string(
    value: &Value,
    key: &str,
    expected: &str,
    path: &Path,
) -> Result<(), String> {
    let actual = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{}: missing string `{key}`", path.display()))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{}: `{key}` is `{actual}`, expected `{expected}`",
            path.display()
        ))
    }
}

fn checked_file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    checked_path(root, relative, true)
}

fn checked_dir(root: &Path, relative: &str) -> Result<PathBuf, String> {
    checked_path(root, relative, false)
}

fn checked_path(root: &Path, relative: &str, file: bool) -> Result<PathBuf, String> {
    let path = root.join(relative);
    let metadata =
        fs::symlink_metadata(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "CLDR input must not be a symlink: {}",
            path.display()
        ));
    }
    if file && !metadata.is_file() {
        return Err(format!("CLDR input is not a file: {}", path.display()));
    }
    if !file && !metadata.is_dir() {
        return Err(format!("CLDR input is not a directory: {}", path.display()));
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if !canonical.starts_with(root) {
        return Err(format!(
            "CLDR input escapes source root: {}",
            path.display()
        ));
    }
    Ok(canonical)
}

fn locale_directories(main: &Path) -> Result<BTreeSet<String>, String> {
    let mut locales = BTreeSet::new();
    for entry in fs::read_dir(main).map_err(|error| format!("{}: {error}", main.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", main.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "CLDR locale entry must not be a symlink: {}",
                entry.path().display()
            ));
        }
        if !file_type.is_dir() {
            continue;
        }
        let locale = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("non-UTF-8 CLDR locale in {}", main.display()))?;
        locales.insert(locale);
    }
    Ok(locales)
}

fn json_files(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "CLDR input must not be a symlink: {}",
                entry.path().display()
            ));
        }
        if file_type.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|value| value == "json")
        {
            files.push(
                entry
                    .path()
                    .canonicalize()
                    .map_err(|error| format!("{}: {error}", entry.path().display()))?,
            );
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!(
            "no CLDR BCP 47 JSON files in {}",
            directory.display()
        ));
    }
    Ok(files)
}

fn locale_set_mismatch(
    numbers: &BTreeSet<String>,
    dates: &BTreeSet<String>,
    layouts: &BTreeSet<String>,
) -> String {
    format!(
        "CLDR locale manifests differ: numbers-only={:?}, dates-only={:?}, layout-only={:?}",
        numbers
            .difference(dates)
            .chain(numbers.difference(layouts))
            .collect::<Vec<_>>(),
        dates
            .difference(numbers)
            .chain(dates.difference(layouts))
            .collect::<Vec<_>>(),
        layouts
            .difference(numbers)
            .chain(layouts.difference(dates))
            .collect::<Vec<_>>()
    )
}

fn hash_source_tree(root: &Path, files: &[PathBuf]) -> Result<String, String> {
    let mut hasher = Sha256::new();
    hasher.update(b"linguini-cldr-source-tree-v1\0");
    let mut buffer = [0u8; 64 * 1024];
    for path in files {
        let relative = path.strip_prefix(root).map_err(|_| {
            format!(
                "CLDR input {} is outside source root {}",
                path.display(),
                root.display()
            )
        })?;
        let relative = relative
            .to_str()
            .ok_or_else(|| format!("non-UTF-8 CLDR input path: {}", path.display()))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let relative_bytes = relative.as_bytes();
        hasher.update(&(relative_bytes.len() as u64).to_be_bytes());
        hasher.update(relative_bytes);

        let metadata =
            fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
        hasher.update(&metadata.len().to_be_bytes());
        let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
    }
    Ok(hex(&hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::{hash_source_tree, verify_checkout_identity, SOURCE_COMMIT};
    use linguini_test_support::temp_project_dir;
    use std::fs;

    #[test]
    fn checkout_identity_requires_exact_full_commit() {
        let project = temp_project_dir("cldr_full_commit").expect("temporary project");
        let git = project.path().join(".git");
        fs::create_dir(&git).expect("git metadata");
        fs::write(git.join("HEAD"), format!("{SOURCE_COMMIT}\n")).expect("full HEAD");
        verify_checkout_identity(project.path()).expect("full commit accepted");

        fs::write(git.join("HEAD"), &SOURCE_COMMIT[..7]).expect("short HEAD");
        let error = verify_checkout_identity(project.path()).expect_err("short commit rejected");
        assert!(error.contains("expected full commit"));
    }

    #[test]
    fn source_tree_hash_detects_content_tampering() {
        let project = temp_project_dir("cldr_tree_hash").expect("temporary project");
        let first = project.path().join("a.json");
        let second = project.path().join("b.json");
        fs::write(&first, b"first").expect("first input");
        fs::write(&second, b"second").expect("second input");
        let root = project.path().canonicalize().expect("canonical root");
        let files = vec![
            first.canonicalize().expect("canonical first"),
            second.canonicalize().expect("canonical second"),
        ];
        let original = hash_source_tree(&root, &files).expect("original hash");

        fs::write(&second, b"tamper").expect("tampered input");
        let tampered = hash_source_tree(&root, &files).expect("tampered hash");

        assert_ne!(tampered, original);
    }
}
