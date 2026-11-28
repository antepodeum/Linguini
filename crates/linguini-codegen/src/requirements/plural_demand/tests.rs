use super::*;
use linguini_ir::{lower_locale_typed, lower_schema_typed};
use linguini_syntax::{parse_locale, parse_schema};

#[test]
fn selector_demand_unions_dimensions_and_preserves_observable_categories() {
    for (source, expected) in [
        ("fn A(Plural) {\n one => One\n _ => Rest\n}\n", Some(vec!["one"])),
        ("fn A(Plural) {\n one => One\n _ => Rest\n}\nfn B(Plural) {\n few => Few\n other => Rest\n}\n", Some(vec!["few", "one"])),
        ("fn A(Choice, Plural, payload: String) {\n a {\n one => One\n _ => Rest\n }\n _ => Outer\n}\n", Some(vec!["one"])),
        ("fn A(Plural) {\n one => One\n}\n", None),
        ("fn A(Plural) {\n one => One\n other => Other\n _ => Default\n}\n", None),
        ("let category = {Plural(value)}\n", None),
        ("fn A(value: Number) {\n _ => {Plural(value)}\n}\n", None),
        ("root = {Plural(value)}\n", None),
        ("impl Item {\n item {\n form name(Plural) {\n one => Item\n _ => Items\n }\n }\n}\n", Some(vec!["one"])),
        ("impl Item {\n item {\n one => Item\n _ => Items\n}\n}\n", Some(vec!["one"])),
    ] {
        let schema = lower_schema_typed(&parse_schema("root\n").unwrap());
        let locale = lower_locale_typed(&parse_locale(source).unwrap());
        let expected = expected.map_or(PluralCategoryDemand::Full, |keys| PluralCategoryDemand::Categories(keys.into_iter().map(String::from).collect()));
        assert_eq!(plural_category_demand(&schema, &locale), expected, "{source}");
    }
}

#[test]
fn equal_fallbacks_need_validation_but_no_numeric_category() {
    let locale = lower_locale_typed(&parse_locale("root = same\n").unwrap());
    let text = locale.messages()[0].body.clone().unwrap();
    let plan = Decision::branch(
        (),
        vec![
            ("one".into(), Decision::Return(text.clone())),
            ("_".into(), Decision::Return(text)),
        ],
    );
    let mut demand = PluralCategoryDemand::default();
    demand.record(&plan, &|_| true);
    assert_eq!(demand, PluralCategoryDemand::default());
    demand.merge(PluralCategoryDemand::Full);
    demand.merge(PluralCategoryDemand::default());
    assert_eq!(demand, PluralCategoryDemand::Full);
}
