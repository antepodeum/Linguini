use crate::{EcmaLocaleSource, EcmaWebOptions, WebCodegenError};

pub fn validate_web_options(options: &EcmaWebOptions) -> Result<(), WebCodegenError> {
    for (index, source) in options.sources.iter().enumerate() {
        if options.sources[..index].contains(source) {
            return Err(WebCodegenError::DuplicateWebSource {
                source: source.as_str().to_owned(),
            });
        }
    }

    let expected = |source: EcmaLocaleSource| options.sources.contains(&source);
    let checks = [
        (EcmaLocaleSource::Path, options.locale_switch.writes_path),
        (
            EcmaLocaleSource::Cookie,
            options.locale_switch.writes_cookie,
        ),
        (
            EcmaLocaleSource::LocalStorage,
            options.locale_switch.writes_local_storage,
        ),
    ];
    for (source, actual) in checks {
        let expected = expected(source);
        if actual != expected {
            return Err(WebCodegenError::WebLocaleSwitchPlanMismatch {
                source: source.as_str(),
                expected,
                actual,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_project_files, EcmaLocaleSwitchPlan, WebProjectOptions};

    #[test]
    fn project_emission_rejects_invalid_closed_web_features() {
        let mut options = WebProjectOptions {
            web: EcmaWebOptions {
                sources: vec![EcmaLocaleSource::Path, EcmaLocaleSource::Path],
                ..Default::default()
            },
            declaration: true,
        };
        assert!(matches!(
            generate_project_files(&options, false),
            Err(WebCodegenError::DuplicateWebSource { .. })
        ));
        options.web = EcmaWebOptions {
            sources: vec![EcmaLocaleSource::Path],
            locale_switch: EcmaLocaleSwitchPlan {
                writes_path: false,
                writes_cookie: false,
                writes_local_storage: false,
            },
            ..Default::default()
        };
        assert!(matches!(
            generate_project_files(&options, false),
            Err(WebCodegenError::WebLocaleSwitchPlanMismatch {
                source: "path",
                expected: true,
                actual: false
            })
        ));
    }
}
