use crate::{CliError, CliResult};
use linguini_analyzer::{Diagnostic, DiagnosticSeverity, LocaleCoverageOptions};
use linguini_config::{discover_locale_files, discover_schema_files, LinguiniConfig};
use linguini_schema::build_schema_symbols_from_files;
use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::fixes::missing_messages_fix_id;
use super::io::{read_file, render_parse_errors};
use super::util::namespace_display;
use super::{ParsedLocaleSource, ParsedSchemaSource};

pub(crate) fn load_schema_sources(
    root: &Path,
    config: &LinguiniConfig,
) -> CliResult<Vec<ParsedSchemaSource>> {
    let mut parsed = Vec::new();
    for (index, file) in discover_schema_files(root.join(&config.paths.schema))?
        .into_iter()
        .enumerate()
    {
        let source = read_file(&file.path)?;
        let ast = parse_schema_in(&source, project_source_id(SourceKind::Schema, index)?).map_err(
            |errors| {
                CliError::Diagnostics(render_parse_errors(
                    root,
                    &file.path,
                    &source,
                    "schema syntax error",
                    errors,
                ))
            },
        )?;
        parsed.push(ParsedSchemaSource { file, source, ast });
    }
    Ok(parsed)
}

pub(crate) fn load_locale_sources(
    root: &Path,
    config: &LinguiniConfig,
) -> CliResult<Vec<ParsedLocaleSource>> {
    let mut parsed = Vec::new();
    for (index, file) in discover_locale_files(root.join(&config.paths.locale))?
        .into_iter()
        .enumerate()
    {
        let source = read_file(&file.path)?;
        let ast = parse_locale_in(&source, project_source_id(SourceKind::Locale, index)?).map_err(
            |errors| {
                CliError::Diagnostics(render_parse_errors(
                    root,
                    &file.path,
                    &source,
                    "locale syntax error",
                    errors,
                ))
            },
        )?;
        parsed.push(ParsedLocaleSource { file, source, ast });
    }
    Ok(parsed)
}

pub(crate) fn locale_index(
    locale_files: &[ParsedLocaleSource],
) -> CliResult<BTreeMap<(String, String), &ParsedLocaleSource>> {
    let mut index = BTreeMap::new();
    for file in locale_files {
        let key = (file.file.namespace.clone(), file.file.locale.clone());
        if let Some(previous) = index.insert(key.clone(), file) {
            return Err(CliError::Diagnostics(format!(
                "duplicate locale source for namespace `{}` and locale `{}`:\n- {}\n- {}\n",
                namespace_display(&key.0),
                key.1,
                previous.file.path.display(),
                file.file.path.display()
            )));
        }
    }
    Ok(index)
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SourceKind {
    Schema,
    Locale,
}

pub(crate) fn project_source_id(kind: SourceKind, index: usize) -> CliResult<SourceId> {
    let kind_offset = match kind {
        SourceKind::Schema => 1,
        SourceKind::Locale => 2,
    };
    let index = u32::try_from(index)
        .ok()
        .and_then(|index| index.checked_mul(2))
        .and_then(|index| index.checked_add(kind_offset))
        .ok_or_else(|| {
            CliError::Diagnostics(
                "project contains too many localization source files\n".to_owned(),
            )
        })?;
    Ok(SourceId(index))
}

pub(crate) fn schema_project_diagnostics(schema_files: &[ParsedSchemaSource]) -> Vec<Diagnostic> {
    let mut namespaces = BTreeMap::<&str, Vec<linguini_syntax::SchemaFile>>::new();
    for source in schema_files {
        namespaces
            .entry(&source.file.namespace)
            .or_default()
            .push(source.ast.clone());
    }

    namespaces
        .into_values()
        .flat_map(|schemas| build_schema_symbols_from_files(&schemas).1)
        .collect()
}

pub(crate) fn expected_locale_path(
    root: &Path,
    config: &LinguiniConfig,
    namespace: &str,
    locale: &str,
) -> PathBuf {
    let mut path = root.join(&config.paths.locale);
    for part in namespace.split('.').filter(|part| !part.is_empty()) {
        path.push(part);
    }
    path.join(format!("{locale}.lgl"))
}

pub(crate) fn coverage_options(
    config: &LinguiniConfig,
    namespace: &str,
    locale: &str,
) -> LocaleCoverageOptions {
    LocaleCoverageOptions {
        missing_message_severity: if locale == config.project.default_locale {
            DiagnosticSeverity::Error
        } else {
            DiagnosticSeverity::Warning
        },
        subject: format!(
            "locale `{locale}` for schema namespace `{}`",
            namespace_display(namespace)
        ),
        quick_fix_id: Some(missing_messages_fix_id(namespace, locale)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        load_locale_sources, load_schema_sources, locale_index, schema_project_diagnostics,
        ParsedLocaleSource, ParsedSchemaSource,
    };
    use linguini_config::{parse_config, LocaleFile, SchemaFile};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn loaders_assign_stable_distinct_nonzero_source_ids() {
        let root = TempDir::new().expect("project");
        fs::create_dir_all(root.path().join("schema")).expect("schema directory");
        fs::create_dir_all(root.path().join("locale/a")).expect("first locale directory");
        fs::create_dir_all(root.path().join("locale/b")).expect("second locale directory");
        fs::write(root.path().join("schema/a.lgs"), "hello\n").expect("first schema");
        fs::write(root.path().join("schema/b.lgs"), "goodbye\n").expect("second schema");
        fs::write(root.path().join("locale/a/en.lgl"), "hello = Hello\n").expect("first locale");
        fs::write(root.path().join("locale/b/en.lgl"), "goodbye = Goodbye\n")
            .expect("second locale");
        let config = parse_config(
            r#"
            [project]
            name = "source-ids"
            default_locale = "en"
            locales = ["en"]

            [paths]
            schema = "schema"
            locale = "locale"
            "#,
        )
        .expect("config");

        let first_schema = load_schema_sources(root.path(), &config).expect("first schema load");
        let locales = load_locale_sources(root.path(), &config).expect("locale load");
        let second_schema = load_schema_sources(root.path(), &config).expect("second schema load");

        let first_schema_ids = first_schema
            .iter()
            .map(|source| source.ast.span.source)
            .collect::<Vec<_>>();
        let second_schema_ids = second_schema
            .iter()
            .map(|source| source.ast.span.source)
            .collect::<Vec<_>>();
        let all_ids = first_schema_ids
            .iter()
            .copied()
            .chain(locales.iter().map(|source| source.ast.span.source))
            .collect::<Vec<_>>();

        assert_eq!(first_schema_ids, second_schema_ids);
        assert!(all_ids.iter().all(|source_id| *source_id != SourceId(0)));
        assert_eq!(
            all_ids.iter().copied().collect::<BTreeSet<_>>().len(),
            all_ids.len()
        );
    }

    #[test]
    fn locale_index_rejects_duplicate_namespace_and_locale() {
        let first = parsed_locale("locales/shop/en.lgl", "shop", "en", SourceId(2));
        let second = parsed_locale("other/shop/en.lgl", "shop", "en", SourceId(4));

        let error = locale_index(&[first, second]).expect_err("duplicate must fail");
        let message = error.to_string();

        assert!(message.contains("duplicate locale source"));
        assert!(message.contains("namespace `shop`"));
        assert!(message.contains("locale `en`"));
        assert!(message.contains("locales/shop/en.lgl"));
        assert!(message.contains("other/shop/en.lgl"));
    }

    #[test]
    fn schema_semantics_reject_duplicates_within_one_namespace_only() {
        let first = parsed_schema("schema/shop/first.lgs", "shop", "hello\n", SourceId(1));
        let duplicate = parsed_schema("schema/shop/second.lgs", "shop", "hello\n", SourceId(3));
        let other = parsed_schema("schema/admin.lgs", "admin", "hello\n", SourceId(5));

        let diagnostics = schema_project_diagnostics(&[first, duplicate, other]);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].span.source, SourceId(3));
        assert_eq!(diagnostics[0].related[0].span.source, SourceId(1));
    }

    fn parsed_locale(
        path: &str,
        namespace: &str,
        locale: &str,
        source_id: SourceId,
    ) -> ParsedLocaleSource {
        let source = "hello = Hello\n".to_owned();
        ParsedLocaleSource {
            file: LocaleFile {
                path: PathBuf::from(path),
                locale: locale.to_owned(),
                namespace: namespace.to_owned(),
            },
            ast: parse_locale_in(&source, source_id).expect("locale"),
            source,
        }
    }

    fn parsed_schema(
        path: &str,
        namespace: &str,
        source: &str,
        source_id: SourceId,
    ) -> ParsedSchemaSource {
        ParsedSchemaSource {
            file: SchemaFile {
                path: PathBuf::from(path),
                namespace: namespace.to_owned(),
            },
            source: source.to_owned(),
            ast: parse_schema_in(source, source_id).expect("schema"),
        }
    }
}
