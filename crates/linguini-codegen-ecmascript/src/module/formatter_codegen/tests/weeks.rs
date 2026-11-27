use super::*;

#[test]
fn compiled_week_formats_match_raw_cldr_wording_and_calendar_boundaries() {
    let mut locales = std::collections::BTreeSet::from([
        "en", "en-GB", "ru", "ar-EG", "fa", "ff-Adlm", "gd", "ja", "sa",
    ]);
    locales.extend([
        "en-GB-u-fw-sun",
        "en-GB-u-fw-mon",
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
        let plan = linguini_codegen::WeekDatePlan::new(locale, &calendar, &rules).unwrap();
        if !plan.selection.branches.is_empty() {
            locales.insert(locale);
        }
    }
    let mut script = String::from(
        r#"
import assert from 'node:assert/strict';
function start(year, firstDay, minDays) {
  const date = new Date(0); date.setUTCFullYear(year,0,1);
  const preceding = (date.getUTCDay()-firstDay+7)%7;
  return date.getTime()+(7-preceding<minDays?7-preceding:-preceding)*86400000;
}
function reference(date, firstDay, minDays, patterns, categories, digits, minus) {
  // Gregorian cycles preserve weekdays. Keep the independent Date-based oracle's
  // January anchors inside TimeClip even when the selected week-year lies outside it.
  const cycle=Math.floor((date.getUTCFullYear()-2000)/400)*400;
  date=new Date(date.getTime());date.setUTCFullYear(date.getUTCFullYear()-cycle);
  let year=date.getUTCFullYear();
  if(date.getTime()<start(year,firstDay,minDays)) year--;
  else if(date.getTime()>=start(year+1,firstDay,minDays)) year++;
  const week=Math.floor((date.getTime()-start(year,firstDay,minDays))/604800000)+1;
  year+=cycle;
  const pattern=patterns['yw-count-'+categories[week-1]]??patterns['yw-count-other']??patterns.yw;
  let result='',quoted=false;
  for(let index=0;index<pattern.length;) {
    const character=pattern[index++];
    if(character==="'") {
      if(pattern[index]==="'") {result+="'";index++;} else quoted=!quoted;
    } else if(!quoted&&(character==='Y'||character==='w')) {
      let width=1;while(pattern[index]===character){width++;index++;}
      const magnitude=character==='w'?week:Math.abs(year);
      const field=(character==='Y'&&year<0?minus:'')+String(character==='Y'&&width===2?magnitude%100:magnitude).padStart(width,'0');
      result+=field.replace(/\d/g,digit=>digits[Number(digit)]);
    } else result+=character;
  }
  return result;
}
"#,
    );
    for locale in locales {
        let emitted = formatter_data_declaration(
            locale,
            &[request(IrFormatterKind::Date, &[("skeleton", "yw")])],
            false,
        );
        assert_eq!(
            emitted.matches("Math.floor((275 * month + 5) / 9)").count(),
            1
        );
        for forbidden in [
            "getWeekYear(",
            "pluralOperands",
            "pluralOperandMatches",
            "selectBranch",
            "yw-count-",
            "firstDay",
            "minDays",
            "weekData",
        ] {
            assert!(!emitted.contains(forbidden), "{locale}: {forbidden}");
        }
        if locale == "mt" {
            assert_eq!(emitted.matches("String(weekNumber)").count(), 1);
            assert_eq!(emitted.matches("String(weekYear)").count(), 1);
        }
        let calendar = linguini_cldr::compiled_calendar_formatting(locale, "gregory").unwrap();
        let week = linguini_cldr::compiled_week_data(locale).unwrap();
        let rules = linguini_cldr::compiled_plural_rules(locale);
        let categories = (1..=53)
            .map(|value| {
                rules.as_ref().map_or("other", |rules| {
                    rules.category_for(&value.to_string()).unwrap()
                })
            })
            .collect::<Vec<_>>();
        let patterns = calendar
            .skeletons
            .iter()
            .filter(|(key, _)| key.starts_with("yw-count-") || *key == "yw")
            .copied()
            .collect::<std::collections::BTreeMap<_, _>>();
        let numbers = compiled_number_formatting(locale).unwrap();
        script.push_str(&format!(r#"
{{
{emitted}
const patterns={patterns},categories={categories},digits=Array.from({digits});
const years=Array.from({{length:400}},(_,index)=>1600+index).concat([-271820,-10000,-400,-100,-4,-1,0,1,4,100,400,10000,275000]);
for(const year of years) for(let month=0;month<12;month++) {{
  for(const day of [1,2,3,4,5,6,7,15,25,26,27,28,29,30,31]) {{
    const date=new Date(0);date.setUTCFullYear(year,month,day);
    if(date.getUTCMonth()!==month) continue;
    const expected=reference(date,{first_day},{min_days},patterns,categories,digits,{minus});
    assert.equal(formatDate_skeleton_yw(date),expected,{locale}+'/'+year+'/'+month+'/'+day);
  }}
}}
const value=new Date('2021-01-01T00:00:00Z');
const reads={{}};
for(const name of ['getTime','getUTCFullYear','getUTCMonth','getUTCDate','getUTCDay']) {{
 const method=value[name];value[name]=function(){{reads[name]=(reads[name]??0)+1;return method.call(this)}};
}}
formatDate_skeleton_yw(value);
assert.deepEqual(Object.values(reads),[1,1,1,1,1]);
const failure=new Error('caller date getter');
value.getUTCDate=()=>{{throw failure}};
assert.throws(()=>formatDate_skeleton_yw(value),error=>error===failure);
}}
"#, patterns=serde_json::to_string(&patterns).unwrap(), categories=serde_json::to_string(&categories).unwrap(),
            digits=crate::ecmascript::string_literal(numbers.digits), minus=crate::ecmascript::string_literal(numbers.minus_sign),
            first_day=week.first_day,min_days=week.min_days,locale=crate::ecmascript::string_literal(locale)));
    }
    script.push_str("console.log('ok');\n");
    assert_eq!(execute(&script), "ok");
}

#[test]
fn week_arithmetic_and_plural_work_follow_selected_requests() {
    let requests = [request(IrFormatterKind::Date, &[("skeleton", "yw")])];
    for (base, equivalent) in [("en", "en-u-fw-sun"), ("en-GB", "en-GB-u-fw-mon")] {
        assert_eq!(
            formatter_data_declaration(base, &requests, false),
            formatter_data_declaration(equivalent, &requests, false),
            "an equivalent weekday override must not add runtime overhead"
        );
    }
    let ordinary = formatter_data_declaration("en", &[request(IrFormatterKind::Date, &[])], false);
    assert!(!ordinary.contains("weekAnchor"));
    let english = formatter_data_declaration(
        "en",
        &[request(IrFormatterKind::Date, &[("skeleton", "yw")])],
        false,
    );
    assert!(!english.contains("weekNumberModulo"));
    assert!(!english.contains("if (weekNumber"));
    let selected = formatter_data_declaration(
        "gd",
        &[
            request(IrFormatterKind::Date, &[("skeleton", "yw")]),
            request(IrFormatterKind::Date, &[("skeleton", "yMMM")]),
        ],
        false,
    );
    assert_eq!(
        selected
            .matches("Math.floor((275 * month + 5) / 9)")
            .count(),
        1
    );
    assert_eq!(selected.matches("function getWeekYear(").count(), 1);
}
