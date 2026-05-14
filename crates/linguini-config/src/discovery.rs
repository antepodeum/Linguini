use crate::error::{ConfigError, ConfigResult};
use crate::model::validate_locale_tag;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SchemaFile {
    pub path: PathBuf,
    pub namespace: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocaleFile {
    pub path: PathBuf,
    pub locale: String,
    pub namespace: String,
}

pub fn discover_schema_files(root: impl AsRef<Path>) -> ConfigResult<Vec<SchemaFile>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_schema_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

pub fn discover_locale_files(root: impl AsRef<Path>) -> ConfigResult<Vec<LocaleFile>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_locale_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

pub fn locale_scope_chain(locale_root: impl AsRef<Path>, file: impl AsRef<Path>) -> Vec<PathBuf> {
    let locale_root = locale_root.as_ref();
    let file = file.as_ref();
    let locale_name = file.file_name().unwrap_or_default();
    let parent = file.parent().unwrap_or(locale_root);
    let relative_parent = parent.strip_prefix(locale_root).unwrap_or(parent);
    let mut paths = Vec::new();
    let mut current = locale_root.to_path_buf();

    paths.push(current.join(locale_name));

    for component in relative_parent.components() {
        current.push(component.as_os_str());
        paths.push(current.join(locale_name));
    }

    paths
}

fn collect_schema_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<SchemaFile>,
) -> ConfigResult<()> {
    for entry in read_directory(directory)? {
        let path = entry.path();

        if path.is_dir() {
            collect_schema_files(root, &path, files)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("lgs") {
            files.push(SchemaFile {
                namespace: schema_namespace(root, &path),
                path,
            });
        }
    }

    Ok(())
}

fn collect_locale_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<LocaleFile>,
) -> ConfigResult<()> {
    for entry in read_directory(directory)? {
        let path = entry.path();

        if path.is_dir() {
            collect_locale_files(root, &path, files)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("lgl") {
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };

            validate_locale_tag(stem)?;
            files.push(LocaleFile {
                locale: stem.to_owned(),
                namespace: locale_namespace(root, &path),
                path,
            });
        }
    }

    Ok(())
}

fn read_directory(path: &Path) -> ConfigResult<Vec<fs::DirEntry>> {
    fs::read_dir(path)
        .map_err(|_| ConfigError::UnreadableDirectory(path.to_path_buf()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ConfigError::UnreadableDirectory(path.to_path_buf()))
}

fn schema_namespace(root: &Path, path: &Path) -> String {
    namespace_from_path(root, path, true)
}

fn locale_namespace(root: &Path, path: &Path) -> String {
    namespace_from_path(root, path, false)
}

fn namespace_from_path(root: &Path, path: &Path, include_file_stem: bool) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut parts = Vec::new();

    if let Some(parent) = relative.parent() {
        parts.extend(
            parent
                .components()
                .filter_map(|component| component.as_os_str().to_str().map(str::to_owned)),
        );
    }

    if include_file_stem {
        if let Some(stem) = relative.file_stem().and_then(|stem| stem.to_str()) {
            parts.push(stem.to_owned());
        }
    }

    parts.join(".")
}

#[cfg(test)]
mod tests {
    use super::{locale_scope_chain, namespace_from_path};
    use std::path::Path;

    #[test]
    fn derives_schema_namespace_from_path() {
        let namespace = namespace_from_path(
            Path::new("linguini/schema"),
            Path::new("linguini/schema/shop/delivery.lgs"),
            true,
        );

        assert_eq!(namespace, "shop.delivery");
    }

    #[test]
    fn derives_locale_namespace_from_parent_path() {
        let namespace = namespace_from_path(
            Path::new("linguini/locale"),
            Path::new("linguini/locale/shop/forms/fruit/ru.lgl"),
            false,
        );

        assert_eq!(namespace, "shop.forms.fruit");
    }

    #[test]
    fn derives_locale_namespace_matching_schema_file_layout() {
        let namespace = namespace_from_path(
            Path::new("locales"),
            Path::new("locales/shop/ru.lgl"),
            false,
        );

        assert_eq!(namespace, "shop");
    }

    #[test]
    fn builds_top_down_locale_scope_chain() {
        let chain = locale_scope_chain("linguini/locale", "linguini/locale/shop/delivery/ru.lgl");

        assert_eq!(
            chain,
            [
                Path::new("linguini/locale/ru.lgl").to_path_buf(),
                Path::new("linguini/locale/shop/ru.lgl").to_path_buf(),
                Path::new("linguini/locale/shop/delivery/ru.lgl").to_path_buf(),
            ]
        );
    }
}
