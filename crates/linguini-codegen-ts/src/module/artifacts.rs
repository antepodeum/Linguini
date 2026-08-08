use std::collections::{BTreeMap, BTreeSet};

use super::{
    visible_schema, TypeScriptCodegenError, TypeScriptOptions, ValidatedTypeScriptProject,
};

const MAX_PORTABLE_PATH_BYTES: usize = 240;
const HEX_COMPONENT_BYTES: usize = 48;

/// Stable physical location and public signature metadata for one bundler leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptMessageArtifact {
    pub locale: String,
    pub message: String,
    pub arity: usize,
    pub module_path: String,
    pub source_map_path: String,
    pub output_file_name: String,
    pub shared_import_path: String,
}

pub(super) fn message_artifacts(
    project: &ValidatedTypeScriptProject<'_>,
) -> Result<Vec<TypeScriptMessageArtifact>, TypeScriptCodegenError> {
    let schema = if project.options.tree_shaking && !project.options.included_messages.is_empty() {
        visible_schema(
            project.schema,
            &TypeScriptOptions {
                included_messages: project.options.included_messages.clone(),
                ..TypeScriptOptions::default()
            },
        )
    } else {
        project.schema.clone()
    };
    let messages = schema
        .messages
        .iter()
        .map(|message| (message.name.as_str(), message.parameters.len()))
        .collect::<BTreeSet<_>>();
    let locales = project
        .locales
        .iter()
        .map(|locale| locale.locale.as_str())
        .collect::<BTreeSet<_>>();
    let mut artifacts = Vec::with_capacity(messages.len() * locales.len());

    for (message, arity) in messages {
        let message_path = hex_path(message.as_bytes());
        for locale in &locales {
            let locale = *locale;
            let locale_file_stem = hex_component(locale.as_bytes());
            let module_path = format!("bundler/messages/{message_path}/{locale_file_stem}.ts");
            if module_path.len() > MAX_PORTABLE_PATH_BYTES {
                return Err(TypeScriptCodegenError::InvalidMessageArtifactPath {
                    locale: locale.to_owned(),
                    message: message.to_owned(),
                    reason: "encoded path exceeds 240 bytes",
                });
            }
            let parent_depth = module_path.matches('/').count();
            let shared_import_path = format!("{}shared", "../".repeat(parent_depth));
            artifacts.push(TypeScriptMessageArtifact {
                locale: locale.to_owned(),
                message: message.to_owned(),
                arity,
                source_map_path: format!("{module_path}.map"),
                output_file_name: format!("{locale_file_stem}.ts"),
                module_path,
                shared_import_path,
            });
        }
    }

    validate_unique_paths(&artifacts)?;
    Ok(artifacts)
}

fn hex_path(value: &[u8]) -> String {
    let encoded = hex_component(value);
    encoded
        .as_bytes()
        .chunks(HEX_COMPONENT_BYTES)
        .map(|chunk| std::str::from_utf8(chunk).expect("hex is ASCII"))
        .collect::<Vec<_>>()
        .join("/")
}

fn hex_component(value: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2 + 1);
    encoded.push('x');
    for byte in value {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn validate_unique_paths(
    artifacts: &[TypeScriptMessageArtifact],
) -> Result<(), TypeScriptCodegenError> {
    let mut paths = BTreeMap::<String, (&str, &str)>::new();
    for artifact in artifacts {
        let folded = artifact.module_path.to_ascii_lowercase();
        if let Some((message, locale)) = paths.insert(
            folded,
            (artifact.message.as_str(), artifact.locale.as_str()),
        ) {
            return Err(TypeScriptCodegenError::OutputPathCollision {
                path: artifact.module_path.clone(),
                conflicts_with: format!("message `{message}`, locale `{locale}`"),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{
        TypeScriptLocaleModule, TypeScriptProjectOptions, ValidatedTypeScriptProject,
    };
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale, parse_schema};

    fn project<'a>(
        schema: &'a linguini_ir::IrModule,
        locale: linguini_ir::IrModule,
        included_messages: Vec<String>,
    ) -> ValidatedTypeScriptProject<'a> {
        ValidatedTypeScriptProject::try_new(
            schema,
            &[TypeScriptLocaleModule {
                locale: "en".to_owned(),
                module: locale,
            }],
            &TypeScriptProjectOptions {
                tree_shaking: true,
                included_messages,
                base_locale: Some("en".to_owned()),
                ..TypeScriptProjectOptions::default()
            },
        )
        .expect("validated project")
    }

    #[test]
    fn artifact_paths_are_case_fold_injective_and_tree_shaking_aware() {
        let mut schema =
            lower_schema(&parse_schema("reserved\nupper\nlower\nunicode\n").expect("schema"));
        let mut locale = lower_locale(
            &parse_locale("reserved = Reserved\nupper = Upper\nlower = Lower\nunicode = Unicode\n")
                .expect("locale"),
        );
        for (module, names) in [
            (&mut schema, ["CON", "Foo", "foo", "é"]),
            (&mut locale, ["CON", "Foo", "foo", "é"]),
        ] {
            for ((message, origin), name) in module
                .messages
                .iter_mut()
                .zip(module.origins.iter_mut())
                .zip(names)
            {
                message.name = name.to_owned();
                origin.name = name.to_owned();
            }
        }
        let all_project = project(&schema, locale, Vec::new());

        let artifacts = all_project.message_artifacts().expect("artifacts");
        assert_eq!(artifacts.len(), 4);
        let folded = artifacts
            .iter()
            .map(|artifact| artifact.module_path.to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(folded.len(), artifacts.len());
        assert!(artifacts.iter().all(|artifact| {
            artifact.module_path.is_ascii()
                && artifact.module_path.starts_with("bundler/messages/x")
                && artifact.source_map_path == format!("{}.map", artifact.module_path)
        }));

        let selected = project(
            &schema,
            all_project.locales[0].module.clone(),
            vec!["Foo".to_owned()],
        )
        .message_artifacts()
        .expect("selected artifacts");
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].message, "Foo");
    }

    #[test]
    fn overlong_artifact_path_fails_with_message_and_locale_context() {
        let name = "m".repeat(120);
        let schema = lower_schema(&parse_schema(&format!("{name}\n")).expect("schema"));
        let locale = lower_locale(&parse_locale(&format!("{name} = Long\n")).expect("locale"));
        let error = project(&schema, locale, Vec::new())
            .message_artifacts()
            .expect_err("overlong path");
        let rendered = error.to_string();
        assert!(rendered.contains(&name));
        assert!(rendered.contains("locale `en`"));
        assert!(rendered.contains("exceeds 240 bytes"));
    }
}
