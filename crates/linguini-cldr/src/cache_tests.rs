use super::{
    cache_root, fetch_cldr_from_dir, fetch_cldr_from_dir_for_locales, inspect_cache,
    require_offline_cache, required_cldr_json_paths, OFFICIAL_CLDR_JSON_REPO,
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
    let extra_dir = project
        .path()
        .join("source/cldr-json/cldr-numbers-full/main/ru");
    fs::write(extra_dir.join("characters.json"), "{}\n").expect("extra data");

    let cache = project.path().join(".linguini/cache");
    fetch_cldr_from_dir_for_locales(project.path().join("source"), &cache, ["ru"]).expect("fetch");

    assert!(cache
        .join("data/cldr-json/cldr-core/supplemental/plurals.json")
        .is_file());
    assert!(cache
        .join("data/cldr-json/cldr-numbers-full/main/ru/numbers.json")
        .is_file());
    assert!(cache
        .join("data/cldr-json/cldr-dates-full/main/ru/ca-gregorian.json")
        .is_file());
    assert!(cache
        .join("data/cldr-json/cldr-misc-full/main/ru/layout.json")
        .is_file());
    assert!(!cache
        .join("data/cldr-json/cldr-numbers-full/main/ru/characters.json")
        .exists());
}

#[test]
fn official_cldr_source_is_unicode_cldr_json_repo() {
    assert_eq!(
        OFFICIAL_CLDR_JSON_REPO,
        "https://github.com/unicode-org/cldr-json"
    );
}

#[test]
fn sparse_fetch_paths_include_only_required_locale_files() {
    let paths = required_cldr_json_paths(&["en", "ru"]);

    assert_eq!(
        paths,
        [
            "cldr-json/cldr-core/supplemental/plurals.json",
            "cldr-json/cldr-numbers-full/main/en/numbers.json",
            "cldr-json/cldr-dates-full/main/en/ca-gregorian.json",
            "cldr-json/cldr-misc-full/main/en/layout.json",
            "cldr-json/cldr-numbers-full/main/ru/numbers.json",
            "cldr-json/cldr-dates-full/main/ru/ca-gregorian.json",
            "cldr-json/cldr-misc-full/main/ru/layout.json",
        ]
    );
}

fn write_required_source_files(root: &std::path::Path, locale: &str) {
    let supplemental = root.join("source/cldr-json/cldr-core/supplemental");
    let numbers = root
        .join("source/cldr-json/cldr-numbers-full/main")
        .join(locale);
    let dates = root
        .join("source/cldr-json/cldr-dates-full/main")
        .join(locale);
    let layout = root
        .join("source/cldr-json/cldr-misc-full/main")
        .join(locale);
    fs::create_dir_all(&supplemental).expect("supplemental dir");
    fs::create_dir_all(&numbers).expect("numbers dir");
    fs::create_dir_all(&dates).expect("dates dir");
    fs::create_dir_all(&layout).expect("layout dir");
    fs::write(supplemental.join("plurals.json"), "{}\n").expect("plural data");
    fs::write(numbers.join("numbers.json"), "{}\n").expect("numbers data");
    fs::write(dates.join("ca-gregorian.json"), "{}\n").expect("calendar data");
    fs::write(
        layout.join("layout.json"),
        format!(
            r#"{{"main":{{"{locale}":{{"layout":{{"orientation":{{"characterOrder":"left-to-right"}}}}}}}}}}"#
        ),
    )
    .expect("layout data");
}
