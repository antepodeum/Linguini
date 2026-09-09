use crate::data::{
    generated_extension_key_alias, generated_extension_type_alias, generated_language_alias,
    generated_likely_subtag, generated_parent_locale, generated_script_alias,
    generated_territory_alias, generated_variant_alias, GENERATED_LOCALE_CANDIDATES,
    GENERATED_NONLIKELY_SCRIPT_PARENT,
};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// CLDR data component whose inheritance rules should be followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleFallbackComponent {
    /// Main locale data, including messages, number/date formats, and layout.
    Main,
    /// Cardinal plural rules. CLDR 48 has no component-specific parent overrides.
    Plurals,
}

/// Error returned when a locale is not a well-formed ASCII BCP 47 tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleError {
    locale: String,
}

impl LocaleError {
    fn invalid(locale: &str) -> Self {
        Self {
            locale: locale.to_owned(),
        }
    }
}

impl fmt::Display for LocaleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid BCP 47 locale tag `{}`", self.locale)
    }
}

impl Error for LocaleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocaleId {
    language: String,
    extlangs: Vec<String>,
    script: Option<String>,
    region: Option<String>,
    variants: Vec<String>,
    extensions: Vec<Vec<String>>,
    private_use: Option<Vec<String>>,
    private_only: bool,
}

/// Canonicalizes BCP 47 syntax and applies CLDR language, script, territory,
/// and variant aliases from the pinned data artifact.
pub fn canonicalize_locale(locale: &str) -> Result<String, LocaleError> {
    canonical_locale_id(locale).map(|locale| locale.to_tag())
}

/// Adds the most likely missing language, script, and region using pinned CLDR data.
pub fn maximize_locale(locale: &str) -> Result<String, LocaleError> {
    let locale = canonical_locale_id(locale)?;
    if locale.private_only {
        return Ok(locale.to_tag());
    }
    Ok(maximize_id(&locale).unwrap_or(locale).to_tag())
}

/// Returns CLDR main-data inheritance order, from the requested locale to `und`.
pub fn locale_fallback_chain(locale: &str) -> Result<Vec<String>, LocaleError> {
    locale_fallback_chain_for(locale, LocaleFallbackComponent::Main)
}

/// Locale tags represented by the pinned CLDR supplemental resolution graph.
pub fn locale_resolution_candidates() -> &'static [&'static str] {
    GENERATED_LOCALE_CANDIDATES
}

/// Returns component-aware CLDR inheritance order, from most to least specific.
pub fn locale_fallback_chain_for(
    locale: &str,
    component: LocaleFallbackComponent,
) -> Result<Vec<String>, LocaleError> {
    let mut canonical = canonical_locale_id(locale)?;
    if canonical.private_only {
        return Ok(vec![canonical.to_tag()]);
    }
    normalize_placeholder_subtags(&mut canonical);

    let mut chain = Vec::new();
    push_unique(&mut chain, canonical.to_tag());

    let mut current = canonical.without_extensions();
    push_unique(&mut chain, current.to_tag());

    if parent_for(&current, component).is_none() {
        if let Some(with_script) = inferred_non_default_script(&current) {
            current = with_script;
            push_unique(&mut chain, current.to_tag());
        }
    }

    for _ in 0..32 {
        if current.language == "und"
            && current.extlangs.is_empty()
            && current.script.is_none()
            && current.region.is_none()
            && current.variants.is_empty()
        {
            break;
        }
        let Some(parent) = parent_for(&current, component).or_else(|| structural_parent(&current))
        else {
            break;
        };
        current = parent;
        push_unique(&mut chain, current.to_tag());
    }

    Ok(chain)
}

fn canonical_locale_id(locale: &str) -> Result<LocaleId, LocaleError> {
    if locale.is_empty() || !locale.is_ascii() || locale.contains('_') {
        return Err(LocaleError::invalid(locale));
    }

    let mut source = if locale.eq_ignore_ascii_case("root") {
        "und".to_owned()
    } else {
        locale.to_owned()
    };
    for _ in 0..8 {
        let Some(replacement) = generated_language_alias(&source.to_ascii_lowercase()) else {
            break;
        };
        if replacement.eq_ignore_ascii_case(&source) {
            break;
        }
        source = replacement.to_owned();
    }

    let mut parsed = parse_syntax(&source).map_err(|_| LocaleError::invalid(locale))?;
    if parsed.private_only {
        return Ok(parsed);
    }
    if parsed.language == "root" {
        parsed.language = "und".to_owned();
    }

    for _ in 0..8 {
        let mut changed = false;

        if let Some((source_alias, replacement)) = find_language_alias(&parsed) {
            let updated = apply_language_alias(&parsed, &source_alias, replacement)
                .map_err(|_| LocaleError::invalid(locale))?;
            changed |= updated != parsed;
            parsed = updated;
        }

        if let Some(script) = parsed.script.as_deref() {
            if let Some(replacement) = generated_script_alias(script) {
                let replacement = canonical_script(replacement);
                changed |= parsed.script.as_deref() != Some(replacement.as_str());
                parsed.script = Some(replacement);
            }
        }

        if let Some(region) = parsed.region.as_deref() {
            if let Some(replacement) = generated_territory_alias(region) {
                let replacement = choose_territory_replacement(&parsed, replacement);
                changed |= parsed.region.as_deref() != Some(replacement.as_str());
                parsed.region = Some(replacement);
            }
        }

        let mut variants = Vec::with_capacity(parsed.variants.len());
        for variant in &parsed.variants {
            if let Some(replacement) = generated_variant_alias(variant) {
                variants.extend(
                    replacement
                        .split_whitespace()
                        .map(|value| value.to_ascii_lowercase()),
                );
                changed = true;
            } else {
                variants.push(variant.clone());
            }
        }
        variants.sort();
        variants.dedup();
        parsed.variants = variants;

        if !changed {
            break;
        }
    }

    parsed.variants.sort();
    parsed.variants.dedup();
    canonicalize_extensions(&mut parsed.extensions).map_err(|_| LocaleError::invalid(locale))?;
    Ok(parsed)
}

fn parse_syntax(locale: &str) -> Result<LocaleId, ()> {
    let subtags = locale.split('-').collect::<Vec<_>>();
    if subtags.is_empty() || subtags.iter().any(|subtag| subtag.is_empty()) {
        return Err(());
    }

    if subtags[0].eq_ignore_ascii_case("x") {
        if subtags.len() < 2
            || !subtags[1..]
                .iter()
                .all(|subtag| valid_alphanumeric(subtag, 1, 8))
        {
            return Err(());
        }
        return Ok(LocaleId {
            language: String::new(),
            extlangs: Vec::new(),
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: Vec::new(),
            private_use: Some(
                subtags[1..]
                    .iter()
                    .map(|subtag| subtag.to_ascii_lowercase())
                    .collect(),
            ),
            private_only: true,
        });
    }

    if !valid_alpha(subtags[0], 2, 8) {
        return Err(());
    }
    let language = subtags[0].to_ascii_lowercase();
    let mut index = 1;
    let mut extlangs = Vec::new();
    if language.len() <= 3 {
        while index < subtags.len()
            && extlangs.len() < 3
            && valid_alpha(subtags[index], 3, 3)
            && generated_territory_alias(&subtags[index].to_ascii_uppercase()).is_none()
        {
            extlangs.push(subtags[index].to_ascii_lowercase());
            index += 1;
        }
    }

    let script = (index < subtags.len() && valid_alpha(subtags[index], 4, 4)).then(|| {
        let script = canonical_script(subtags[index]);
        index += 1;
        script
    });
    let region = (index < subtags.len()
        && (valid_alpha(subtags[index], 2, 2)
            || valid_numeric(subtags[index], 3, 3)
            || (valid_alpha(subtags[index], 3, 3)
                && generated_territory_alias(&subtags[index].to_ascii_uppercase()).is_some())))
    .then(|| {
        let region = subtags[index].to_ascii_uppercase();
        index += 1;
        region
    });

    let mut variants = Vec::new();
    let mut seen_variants = BTreeSet::new();
    while index < subtags.len() && valid_variant(subtags[index]) {
        let variant = subtags[index].to_ascii_lowercase();
        if !seen_variants.insert(variant.clone()) {
            return Err(());
        }
        variants.push(variant);
        index += 1;
    }

    let mut extensions = Vec::new();
    let mut seen_extensions = BTreeSet::new();
    while index < subtags.len()
        && subtags[index].len() == 1
        && subtags[index]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
        && !subtags[index].eq_ignore_ascii_case("x")
    {
        let singleton = subtags[index].to_ascii_lowercase();
        if !seen_extensions.insert(singleton.clone()) {
            return Err(());
        }
        index += 1;
        let start = index;
        let mut extension = vec![singleton];
        while index < subtags.len() && valid_alphanumeric(subtags[index], 2, 8) {
            extension.push(subtags[index].to_ascii_lowercase());
            index += 1;
        }
        if index == start {
            return Err(());
        }
        extensions.push(extension);
    }

    let private_use = if index < subtags.len() && subtags[index].eq_ignore_ascii_case("x") {
        index += 1;
        let start = index;
        let private_use = subtags[index..]
            .iter()
            .take_while(|subtag| valid_alphanumeric(subtag, 1, 8))
            .map(|subtag| subtag.to_ascii_lowercase())
            .collect::<Vec<_>>();
        index += private_use.len();
        if index == start {
            return Err(());
        }
        Some(private_use)
    } else {
        None
    };

    if index != subtags.len() {
        return Err(());
    }

    variants.sort();
    canonicalize_extensions(&mut extensions)?;
    Ok(LocaleId {
        language,
        extlangs,
        script,
        region,
        variants,
        extensions,
        private_use,
        private_only: false,
    })
}

fn find_language_alias(locale: &LocaleId) -> Option<(String, &'static str)> {
    let mut candidates = Vec::new();
    push_unique(&mut candidates, locale.base_tag());

    let mut core = vec![locale.language.clone()];
    core.extend(locale.extlangs.iter().cloned());
    if let Some(script) = &locale.script {
        core.push(script.clone());
    }
    if let Some(region) = &locale.region {
        core.push(region.clone());
    }
    push_unique(&mut candidates, core.join("-"));

    if locale.script.is_some() && locale.region.is_some() {
        let mut without_region = vec![locale.language.clone()];
        without_region.extend(locale.extlangs.iter().cloned());
        without_region.push(locale.script.clone().expect("checked script"));
        push_unique(&mut candidates, without_region.join("-"));
    }
    if locale.region.is_some() {
        let mut with_region = vec![locale.language.clone()];
        with_region.extend(locale.extlangs.iter().cloned());
        with_region.push(locale.region.clone().expect("checked region"));
        push_unique(&mut candidates, with_region.join("-"));
    }
    let mut language = vec![locale.language.clone()];
    language.extend(locale.extlangs.iter().cloned());
    push_unique(&mut candidates, language.join("-"));
    push_unique(&mut candidates, locale.language.clone());

    if let Some(script) = &locale.script {
        push_unique(&mut candidates, format!("und-{script}"));
    }
    for subset in variant_subsets(&locale.variants) {
        push_unique(&mut candidates, format!("und-{}", subset.join("-")));
    }

    candidates.into_iter().find_map(|candidate| {
        let key = candidate.to_ascii_lowercase();
        generated_language_alias(&key).map(|replacement| (candidate, replacement))
    })
}

fn variant_subsets(variants: &[String]) -> Vec<Vec<String>> {
    if variants.is_empty() {
        return Vec::new();
    }
    if variants.len() > 8 {
        return variants
            .iter()
            .cloned()
            .map(|variant| vec![variant])
            .collect();
    }
    let mut subsets = (1usize..(1usize << variants.len()))
        .map(|mask| {
            variants
                .iter()
                .enumerate()
                .filter(|(index, _)| mask & (1 << index) != 0)
                .map(|(_, variant)| variant.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    subsets.sort_by_key(|right| std::cmp::Reverse(right.len()));
    subsets
}

fn apply_language_alias(
    locale: &LocaleId,
    source: &str,
    replacement: &str,
) -> Result<LocaleId, ()> {
    let source = parse_syntax(source)?;
    let replacement = parse_syntax(replacement)?;
    let mut result = locale.clone();

    if source.language != "und" || replacement.language != "und" {
        result.language = replacement.language;
        result.extlangs = replacement.extlangs;
    }

    if source.script.is_some() || result.script.is_none() {
        result.script = replacement.script.clone();
    }
    if source.region.is_some() || result.region.is_none() {
        result.region = replacement.region.clone();
    }

    result
        .variants
        .retain(|variant| !source.variants.contains(variant));
    result.variants.extend(replacement.variants);
    result.variants.sort();
    result.variants.dedup();
    Ok(result)
}

fn choose_territory_replacement(locale: &LocaleId, replacements: &str) -> String {
    let replacements = replacements
        .split_whitespace()
        .map(|replacement| replacement.to_ascii_uppercase())
        .collect::<Vec<_>>();
    if replacements.len() == 1 {
        return replacements[0].clone();
    }

    let mut without_region = locale.clone();
    without_region.region = None;
    if let Some(likely) = likely_target(&without_region) {
        if let Some(region) = likely.region {
            if replacements.contains(&region) {
                return region;
            }
        }
    }
    replacements[0].clone()
}

fn maximize_id(locale: &LocaleId) -> Option<LocaleId> {
    let mut normalized = locale.clone();
    normalize_placeholder_subtags(&mut normalized);
    let likely = likely_target(&normalized)?;
    let mut maximized = normalized;
    if maximized.language == "und" {
        maximized.language = likely.language;
        maximized.extlangs = likely.extlangs;
    }
    if maximized.script.is_none() {
        maximized.script = likely.script;
    }
    if maximized.region.is_none() {
        maximized.region = likely.region;
    }
    Some(maximized)
}

fn normalize_placeholder_subtags(locale: &mut LocaleId) {
    if locale.script.as_deref() == Some("Zzzz") {
        locale.script = None;
    }
    if locale.region.as_deref() == Some("ZZ") {
        locale.region = None;
    }
}

fn likely_target(locale: &LocaleId) -> Option<LocaleId> {
    let mut candidates = Vec::new();
    let language = locale.language.as_str();
    if let (Some(script), Some(region)) = (&locale.script, &locale.region) {
        push_unique(&mut candidates, format!("{language}-{script}-{region}"));
    }
    if let Some(script) = &locale.script {
        push_unique(&mut candidates, format!("{language}-{script}"));
    }
    if let Some(region) = &locale.region {
        push_unique(&mut candidates, format!("{language}-{region}"));
    }
    push_unique(&mut candidates, language.to_owned());
    if let Some(script) = &locale.script {
        push_unique(&mut candidates, format!("und-{script}"));
    }
    if let Some(region) = &locale.region {
        push_unique(&mut candidates, format!("und-{region}"));
    }
    push_unique(&mut candidates, "und".to_owned());

    candidates.into_iter().find_map(|candidate| {
        generated_likely_subtag(&candidate).and_then(|target| parse_syntax(target).ok())
    })
}

fn inferred_non_default_script(locale: &LocaleId) -> Option<LocaleId> {
    if locale.script.is_some() || locale.region.is_none() {
        return None;
    }
    let maximized = maximize_id(locale)?;
    let requested_script = maximized.script.as_deref()?;

    let language_only = LocaleId::language_only(&locale.language);
    let default_script = maximize_id(&language_only)?.script?;
    if requested_script == default_script {
        return None;
    }

    let mut inferred = locale.clone();
    inferred.script = Some(requested_script.to_owned());
    Some(inferred)
}

fn parent_for(locale: &LocaleId, component: LocaleFallbackComponent) -> Option<LocaleId> {
    if component != LocaleFallbackComponent::Main {
        return None;
    }
    if let Some(parent) = generated_parent_locale(&locale.base_tag()) {
        return parse_syntax(parent).ok();
    }
    if locale.region.is_none()
        && locale.variants.is_empty()
        && locale.extlangs.is_empty()
        && locale.script.is_some()
    {
        let default_script = maximize_id(&LocaleId::language_only(&locale.language))?.script?;
        if locale.script.as_ref() != Some(&default_script) {
            let parent = if GENERATED_NONLIKELY_SCRIPT_PARENT == "root" {
                "und"
            } else {
                GENERATED_NONLIKELY_SCRIPT_PARENT
            };
            return Some(LocaleId::language_only(parent));
        }
    }
    None
}

fn canonicalize_extensions(extensions: &mut [Vec<String>]) -> Result<(), ()> {
    for extension in extensions.iter_mut() {
        match extension.first().map(String::as_str) {
            Some("u") => canonicalize_unicode_extension(extension)?,
            Some("t") => canonicalize_transform_extension(extension)?,
            _ => {}
        }
    }
    extensions.sort_by(|left, right| left[0].cmp(&right[0]));
    Ok(())
}

fn canonicalize_unicode_extension(extension: &mut Vec<String>) -> Result<(), ()> {
    let mut index = 1;
    let mut attributes = Vec::new();
    while index < extension.len() && extension[index].len() >= 3 {
        attributes.push(extension[index].clone());
        index += 1;
    }
    attributes.sort();
    if attributes.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(());
    }

    let mut fields = Vec::new();
    let mut keys = BTreeSet::new();
    while index < extension.len() {
        let raw_key = extension[index].clone();
        let key = generated_extension_key_alias("u", &raw_key)
            .unwrap_or(&raw_key)
            .to_owned();
        if raw_key.len() != 2 || !keys.insert(key.clone()) {
            return Err(());
        }
        index += 1;
        let mut values = Vec::new();
        while index < extension.len() && extension[index].len() >= 3 {
            values.push(extension[index].clone());
            index += 1;
        }
        apply_extension_type_alias("u", &key, &mut values);
        if values == ["true"] {
            values.clear();
        }
        fields.push((key, values));
    }
    fields.sort_by(|left, right| left.0.cmp(&right.0));

    extension.truncate(1);
    extension.extend(attributes);
    for (key, values) in fields {
        extension.push(key);
        extension.extend(values);
    }
    Ok(())
}

fn canonicalize_transform_extension(extension: &mut Vec<String>) -> Result<(), ()> {
    let first_field = extension[1..]
        .iter()
        .position(|subtag| is_transform_key(subtag))
        .map_or(extension.len(), |position| position + 1);
    let mut language = extension[1..first_field].to_vec();
    if !language.is_empty() {
        let canonical = canonical_locale_id(&language.join("-")).map_err(|_| ())?;
        language = canonical
            .to_tag()
            .to_ascii_lowercase()
            .split('-')
            .map(str::to_owned)
            .collect();
    }
    let mut index = first_field;
    let mut fields = Vec::new();
    let mut keys = BTreeSet::new();
    while index < extension.len() {
        let raw_key = extension[index].clone();
        let key = generated_extension_key_alias("t", &raw_key)
            .unwrap_or(&raw_key)
            .to_owned();
        if !is_transform_key(&raw_key) || !keys.insert(key.clone()) {
            return Err(());
        }
        index += 1;
        let start = index;
        let mut values = Vec::new();
        while index < extension.len() && !is_transform_key(&extension[index]) {
            values.push(extension[index].clone());
            index += 1;
        }
        if index == start {
            return Err(());
        }
        apply_extension_type_alias("t", &key, &mut values);
        fields.push((key, values));
    }
    fields.sort_by(|left, right| left.0.cmp(&right.0));

    extension.truncate(1);
    extension.extend(language);
    for (key, values) in fields {
        extension.push(key);
        extension.extend(values);
    }
    Ok(())
}

fn apply_extension_type_alias(extension: &str, key: &str, values: &mut Vec<String>) {
    for _ in 0..4 {
        let source = values.join("-");
        let Some(replacement) = generated_extension_type_alias(extension, key, &source) else {
            break;
        };
        if replacement == source {
            break;
        }
        *values = replacement.split('-').map(str::to_owned).collect();
    }
}

fn is_transform_key(value: &str) -> bool {
    value.len() == 2
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic())
        && value
            .as_bytes()
            .get(1)
            .is_some_and(|byte| byte.is_ascii_digit())
}

fn structural_parent(locale: &LocaleId) -> Option<LocaleId> {
    let mut parent = locale.without_extensions();
    if parent.variants.pop().is_some() {
        return Some(parent);
    }
    if parent.region.take().is_some() {
        return Some(parent);
    }
    if parent.script.take().is_some() {
        return Some(parent);
    }
    if parent.extlangs.pop().is_some() {
        return Some(parent);
    }
    if parent.language != "und" {
        return Some(LocaleId::language_only("und"));
    }
    None
}

impl LocaleId {
    fn language_only(language: &str) -> Self {
        Self {
            language: language.to_owned(),
            extlangs: Vec::new(),
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: Vec::new(),
            private_use: None,
            private_only: false,
        }
    }

    fn without_extensions(&self) -> Self {
        let mut locale = self.clone();
        locale.extensions.clear();
        locale.private_use = None;
        locale
    }

    fn base_tag(&self) -> String {
        let mut subtags = vec![self.language.clone()];
        subtags.extend(self.extlangs.iter().cloned());
        subtags.extend(self.script.iter().cloned());
        subtags.extend(self.region.iter().cloned());
        subtags.extend(self.variants.iter().cloned());
        subtags.join("-")
    }

    fn to_tag(&self) -> String {
        if self.private_only {
            let mut subtags = vec!["x".to_owned()];
            subtags.extend(self.private_use.iter().flatten().cloned());
            return subtags.join("-");
        }
        let mut subtags = vec![self.base_tag()];
        for extension in &self.extensions {
            subtags.push(extension.join("-"));
        }
        if let Some(private_use) = &self.private_use {
            subtags.push(format!("x-{}", private_use.join("-")));
        }
        subtags.join("-")
    }
}

fn canonical_script(value: &str) -> String {
    let lowercase = value.to_ascii_lowercase();
    format!("{}{}", lowercase[..1].to_ascii_uppercase(), &lowercase[1..])
}

fn valid_alpha(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_alphabetic())
}

fn valid_numeric(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_alphanumeric(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn valid_variant(value: &str) -> bool {
    valid_alphanumeric(value, 5, 8)
        || (value.len() == 4
            && value
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_digit())
            && value.bytes().all(|byte| byte.is_ascii_alphanumeric()))
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_locale, locale_fallback_chain, locale_fallback_chain_for, maximize_locale,
        LocaleFallbackComponent,
    };

    #[test]
    fn canonicalization_applies_cldr_aliases() {
        for (input, expected) in [
            ("IW-il", "he-IL"),
            ("sh-BA", "sr-Latn-BA"),
            ("cnr-BA", "sr-BA"),
            ("hy-SU", "hy-AM"),
            ("ja-Latn-hepburn-heploc", "ja-Latn-alalc97"),
            ("en-gb-oed", "en-GB-oxendict"),
            ("eng-USA", "en-US"),
            ("root", "und"),
        ] {
            assert_eq!(canonicalize_locale(input).expect(input), expected);
        }
        assert_eq!(
            canonicalize_locale("en-u-foo-bar-nu-thai-ca-buddhist-kk-true")
                .expect("Unicode extension"),
            "en-u-bar-foo-ca-buddhist-kk-nu-thai"
        );
        assert_eq!(
            canonicalize_locale("iw-u-ms-imperial").expect("extension type alias"),
            "he-u-ms-uksystem"
        );
        assert_eq!(
            canonicalize_locale("en-t-iw").expect("transform language alias"),
            "en-t-he"
        );
        assert_eq!(
            canonicalize_locale("root-u-cu-usd").expect("root with extension"),
            "und-u-cu-usd"
        );
    }

    #[test]
    fn likely_subtags_fill_missing_script_and_region() {
        assert_eq!(
            maximize_locale("zh-TW").expect("Chinese locale"),
            "zh-Hant-TW"
        );
        assert_eq!(
            maximize_locale("sr-ME").expect("Serbian locale"),
            "sr-Latn-ME"
        );
        assert_eq!(
            maximize_locale("zh-Zzzz-SG").expect("placeholder script"),
            "zh-Hans-SG"
        );
    }

    #[test]
    fn main_fallback_uses_parent_locales_and_non_default_scripts() {
        assert_eq!(
            locale_fallback_chain("en-AU").expect("English locale"),
            ["en-AU", "en-001", "en", "und"]
        );
        assert_eq!(
            locale_fallback_chain("es-AR").expect("Spanish locale"),
            ["es-AR", "es-419", "es", "und"]
        );
        assert_eq!(
            locale_fallback_chain("zh-TW").expect("Chinese locale"),
            ["zh-TW", "zh-Hant-TW", "zh-Hant", "und"]
        );
        assert_eq!(
            locale_fallback_chain("ru-Latn").expect("non-likely script"),
            ["ru-Latn", "und"]
        );
        assert_eq!(
            locale_fallback_chain("zh-Zzzz-SG").expect("placeholder script"),
            ["zh-SG", "zh", "und"]
        );
    }

    #[test]
    fn plural_fallback_uses_component_specific_truncation() {
        assert_eq!(
            locale_fallback_chain_for("sh", LocaleFallbackComponent::Plurals)
                .expect("Serbo-Croatian alias"),
            ["sr-Latn", "sr", "und"]
        );
        assert_eq!(
            locale_fallback_chain("sh").expect("Serbo-Croatian alias"),
            ["sr-Latn", "und"]
        );
    }

    #[test]
    fn extensions_do_not_pollute_resource_inheritance() {
        assert_eq!(
            locale_fallback_chain("EN-au-u-NU-latn").expect("extended locale"),
            ["en-AU-u-nu-latn", "en-AU", "en-001", "en", "und"]
        );
    }

    #[test]
    fn invalid_tags_are_rejected() {
        for locale in ["", "e", "en_US", "en--US", "en-u", "x"] {
            assert!(canonicalize_locale(locale).is_err(), "{locale}");
        }
    }
}
