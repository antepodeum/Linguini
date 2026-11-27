use super::*;
use std::fs;

#[test]
fn month_week_formats_match_raw_patterns_and_independent_month_anchors() {
    let mut locales = std::collections::BTreeSet::from([
        "en", "en-GB", "ru", "ar-EG", "fa", "ff-Adlm", "fi", "zh", "vi", "st", "sa",
    ]);
    locales.extend([
        "en-GB-u-fw-sun",
        "en-u-fw-mon",
        "en-GB-u-fw-tue",
        "en-GB-u-fw-wed",
        "en-GB-u-fw-thu",
        "en-GB-u-fw-fri",
        "en-GB-u-fw-sat",
    ]);
    for locale in linguini_cldr::locale_resolution_candidates() {
        let Some(calendar) = linguini_cldr::compiled_calendar_formatting(locale, "gregory") else {
            continue;
        };
        let Some(rules) = linguini_cldr::built_in_plural_rules(locale) else {
            continue;
        };
        let plan = linguini_codegen::WeekDatePlan::for_skeleton(locale, &calendar, &rules, "MMMMW")
            .unwrap();
        if !plan.selection.branches.is_empty() {
            locales.insert(locale);
        }
    }
    let mut script = String::from(
        r#"
import assert from 'node:assert/strict';
function reference(input, firstDay, minDays, patterns, categories, names, digits) {
  const date=new Date(input.getTime());
  date.setUTCFullYear(2000+(input.getUTCFullYear()%400+400)%400);
  const first=new Date(date.getTime());first.setUTCDate(1);
  const preceding=(first.getUTCDay()-firstDay+7)%7;
  const start=1-preceding+(7-preceding<minDays?7:0);
  const week=Math.floor((date.getUTCDate()-start)/7)+1;
  const pattern=patterns['MMMMW-count-'+categories[week]]??patterns['MMMMW-count-other'];
  let result='',quoted=false;
  for(let index=0;index<pattern.length;) {
    const character=pattern[index++];
    if(character==="'") {
      if(pattern[index]==="'") {result+="'";index++;} else quoted=!quoted;
    } else if(!quoted&&['W','M','L'].includes(character)) {
      let width=1;while(pattern[index]===character){width++;index++;}
      if(character!=='W'&&width>=3) result+=names[character.repeat(width)][date.getUTCMonth()];
      else result+=String(character==='W'?week:date.getUTCMonth()+1).padStart(width,'0').replace(/\d/g,digit=>digits[Number(digit)]);
    } else result+=character;
  }
  return result;
}
globalThis.Intl.DateTimeFormat=class {constructor(){throw Error('host date formatting must not run');}};
"#,
    );
    for locale in locales {
        let emitted = formatter_data_declaration(
            locale,
            &[request(IrFormatterKind::Date, &[("skeleton", "MMMMW")])],
            true,
        );
        for forbidden in [
            "getUTCFullYear",
            "leapYear",
            "weekYear",
            "weekAnchor",
            "getWeekYear(",
            "pluralOperands",
            "pluralOperandMatches",
            "selectBranch",
            "MMMMW-count-",
            "firstDay",
            "minDays",
            "weekData",
        ] {
            assert!(!emitted.contains(forbidden), "{locale}: {forbidden}");
        }
        assert_eq!(emitted.matches("date.getUTCMonth()").count(), 1, "{locale}");
        let snapshot_name = if locale == "sa" {
            Some("constant-weeks".to_owned())
        } else if ["en-GB", "mt", "fa", "zh"].contains(&locale) {
            Some(format!("month-week-{locale}"))
        } else {
            None
        };
        if let Some(snapshot_name) = snapshot_name {
            let source = if locale == "sa" {
                formatter_data_declaration(
                    locale,
                    &[
                        request(IrFormatterKind::Date, &[("skeleton", "yw")]),
                        request(IrFormatterKind::Date, &[("skeleton", "MMMMW")]),
                    ],
                    true,
                )
            } else {
                emitted.clone()
            };
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../tests/fixtures/golden/snapshots/js-date/{snapshot_name}.js"
            ));
            if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
                fs::write(&path, &source).unwrap();
            }
            assert_eq!(source, fs::read_to_string(path).unwrap());
        }
        let calendar = linguini_cldr::compiled_calendar_formatting(locale, "gregory").unwrap();
        let week = linguini_cldr::compiled_week_data(locale).unwrap();
        let rules = linguini_cldr::compiled_plural_rules(locale);
        let categories = (0..=6)
            .map(|value| {
                rules.as_ref().map_or("other", |rules| {
                    rules.category_for(&value.to_string()).unwrap()
                })
            })
            .collect::<Vec<_>>();
        let patterns = calendar
            .skeletons
            .iter()
            .filter(|(key, _)| key.starts_with("MMMMW-count-"))
            .copied()
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut names = std::collections::BTreeMap::new();
        for (key, context, width) in [
            ("MMMM", "format", "wide"),
            ("MMM", "format", "abbreviated"),
            ("LLLL", "stand-alone", "wide"),
        ] {
            names.insert(
                key,
                (1..=12)
                    .map(|month| {
                        calendar
                            .symbol(&format!("months.{context}.{width}.{month}"))
                            .unwrap()
                    })
                    .collect::<Vec<_>>(),
            );
        }
        let digits = compiled_number_formatting(locale).unwrap().digits;
        script.push_str(&format!(r#"
{{
const {{formatDate_skeleton_MMMMW}}=await import('data:text/javascript,'+encodeURIComponent({emitted}));
const patterns={patterns},categories={categories},names={names},digits=Array.from({digits});
const years=Array.from({{length:400}},(_,index)=>1600+index).concat([-271820,-10000,-400,-100,-4,-1,0,1,4,100,400,10000,275000]);
for(const year of years) for(let month=0;month<12;month++) for(let day=1;day<=31;day++) {{
 const date=new Date(0);date.setUTCFullYear(year,month,day);
 if(date.getUTCMonth()!==month) continue;
 assert.equal(formatDate_skeleton_MMMMW(date),reference(date,{first_day},{min_days},patterns,categories,names,digits),{locale}+'/'+year+'/'+month+'/'+day);
}}
for(const timestamp of [-8640000000000000,8640000000000000,0,-1]) {{
 const date=new Date(timestamp);
 assert.equal(formatDate_skeleton_MMMMW(timestamp),reference(date,{first_day},{min_days},patterns,categories,names,digits));
}}
const value=new Date('2021-01-01T00:00:00Z');
const reads={{}};
for(const name of ['getTime','getUTCMonth','getUTCDate','getUTCDay']) {{
 const method=value[name];value[name]=function(){{reads[name]=(reads[name]??0)+1;return method.call(this)}};
}}
value.getUTCFullYear=()=>{{throw Error('unused year read')}};
formatDate_skeleton_MMMMW(value);
assert.deepEqual(Object.values(reads),[1,1,1,1]);
const failure=new Error('caller date getter');value.getUTCDay=()=>{{throw failure}};
assert.throws(()=>formatDate_skeleton_MMMMW(value),error=>error===failure);
assert.equal(formatDate_skeleton_MMMMW(false),formatDate_skeleton_MMMMW(0));
assert.equal(formatDate_skeleton_MMMMW(null),formatDate_skeleton_MMMMW(0));
for(const value of ['2024-02-30',Infinity,new Date(NaN)]) assert.throws(()=>formatDate_skeleton_MMMMW(value),RangeError);
}}
"#, emitted=crate::ecmascript::string_literal(&emitted), patterns=serde_json::to_string(&patterns).unwrap(), categories=serde_json::to_string(&categories).unwrap(), names=serde_json::to_string(&names).unwrap(), digits=crate::ecmascript::string_literal(digits), first_day=week.first_day, min_days=week.min_days, locale=crate::ecmascript::string_literal(locale)));
    }
    script.push_str("console.log('ok');\n");
    assert_eq!(execute(&script), "ok");
}
