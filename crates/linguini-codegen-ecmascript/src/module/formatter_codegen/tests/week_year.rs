use super::*;

fn runtime(locale: &str, pattern: &str, exported: bool) -> String {
    let dates = linguini_cldr::compiled_date_formatting(locale).unwrap();
    let compiled = date_parts(pattern, &dates, "formatWeekYear", locale);
    assert!(compiled.helpers.week_year);
    let formatter = crate::templates::render(
        "formatters/date",
        minijinja::context! { name => "formatWeekYear", exported, parts => compiled.parts, tables => compiled.tables, periods => compiled.periods, clock => compiled.clock, digits => dates.digits, localize_result => compiled.localize_result },
    );
    assert_eq!(formatter.matches("date.getUTCFullYear()").count(), 1);
    assert_eq!(formatter.matches("date.getUTCMonth()").count(), 1);
    let mut output = formatter;
    output.push_str(&date_formatter_helpers(compiled.helpers.padding));
    output.push_str(&week_year_formatter_helper(locale));
    if compiled.helpers.digits {
        output.push_str(&digit_formatter_helper(dates.digits));
    }
    output
}

#[test]
fn compiled_week_years_match_reference_boundaries_and_signed_widths() {
    let mut script = String::from("import assert from 'node:assert/strict';\n");
    script.push_str(
        r#"
function january(year) {
  const date = new Date(0);
  date.setUTCFullYear(year, 0, 1);
  return date;
}
function start(year, firstDay, minDays) {
  const date = january(year);
  const preceding = (date.getUTCDay() - firstDay + 7) % 7;
  return date.getTime() + (7 - preceding < minDays ? 7 - preceding : -preceding) * 86400000;
}
function reference(date, firstDay, minDays) {
  const year = date.getUTCFullYear();
  if(date.getTime() < start(year, firstDay, minDays)) return year - 1;
  if(date.getTime() >= start(year + 1, firstDay, minDays)) return year + 1;
  return year;
}
"#,
    );
    for locale in ["en-US", "en-GB", "en-AU", "pt-PT", "dv", "ar", "fa", "gd"] {
        let emitted = runtime(
            locale,
            "Y'|'YY'|'YYY'|'YYYY'|'YYYYY'|'YYYYYY'|'YYYYYYY'|'YYYYYYYY",
            false,
        );
        let week = linguini_cldr::compiled_week_data(locale).unwrap();
        let numbers = compiled_number_formatting(locale).unwrap();
        let digits = crate::ecmascript::string_literal(numbers.digits);
        let minus = crate::ecmascript::string_literal(numbers.minus_sign);
        for forbidden in [
            "Intl.",
            "weekData",
            "firstDay",
            "minDays",
            "Date.UTC",
            "new Date(",
        ] {
            // coerceDate constructs the selected input Date; week arithmetic must not allocate one.
            let helper = week_year_formatter_helper(locale);
            assert!(!helper.contains(forbidden), "{locale}: {forbidden}");
        }
        script.push_str(&format!(r#"
{{
{emitted}
const digits=Array.from({digits});
const localize=value=>value.replace(/\d/g,digit=>digits[Number(digit)]);
const years=Array.from({{length:400}},(_,index)=>1600+index).concat([-271820,-10000,-400,-100,-4,-1,0,1,4,100,400,10000,275000]);
for(const year of years) {{
  for(const [month,days] of [[0,[1,2,3,4,5,6,7]],[11,[25,26,27,28,29,30,31]],[5,[15]]]) {{
    for(const day of days) {{
      const date=new Date(0);date.setUTCFullYear(year,month,day);
      const expectedYear=reference(date,{first_day},{min_days});
      const magnitude=Math.abs(expectedYear);
      const expected=Array.from({{length:8}},(_,index)=>{{
        const width=index+1;
        return (expectedYear<0?{minus}:"")+String(width===2?magnitude%100:magnitude).padStart(width,"0");
      }}).join('|');
      assert.equal(formatWeekYear(date),localize(expected),`${{year}}/${{month}}/${{day}}`);
    }}
  }}
}}
}}
"#,
            first_day = week.first_day, min_days = week.min_days));
    }
    script.push_str("console.log('ok');\n");
    assert_eq!(execute(&script), "ok");
}

#[test]
fn week_year_helpers_are_demand_selected_and_preserve_calendar_years() {
    let only_calendar = formatter_data_declaration(
        "gd",
        &[request(IrFormatterKind::Date, &[("skeleton", "yMMMd")])],
        true,
    );
    assert!(!only_calendar.contains("getWeekYear"));
    let selected = formatter_data_declaration(
        "gd",
        &[
            request(IrFormatterKind::Date, &[("skeleton", "yMMM")]),
            request(IrFormatterKind::Date, &[("skeleton", "yMMMd")]),
        ],
        true,
    );
    assert_eq!(selected.matches("function getWeekYear(").count(), 1);
    assert!(!selected.contains("weekYearSign"));
    assert!(!selected.contains("weekYearMagnitude"));
    assert!(!selected.contains("padNumber"));
    assert!(!week_year_formatter_helper("en-US").contains("month === 0"));
    assert!(week_year_formatter_helper("en-GB").contains("month === 0"));
    let emitted = runtime("en-GB", "Y'|'y'|'G'|'MMM'|'QQ", false);
    let script = format!(
        r#"{emitted}
const date=new Date('2021-01-01T00:00:00Z');
assertResult('2020|2021|AD|Jan|01');
function assertResult(expected){{if(formatWeekYear(date)!==expected)throw Error(formatWeekYear(date));}}
date.setUTCFullYear(0,0,1);
assertResult('-1|1|BC|Jan|01');
let yearReads=0,monthReads=0,dayReads=0,weekdayReads=0;
date.getUTCFullYear=()=>{{yearReads++;return 2021;}};
date.getUTCMonth=()=>{{monthReads++;return 5;}};
date.getUTCDate=()=>{{dayReads++;return 15;}};
date.getUTCDay=()=>{{weekdayReads++;return 2;}};
assertResult('2021|2021|AD|Jun|02');
if(yearReads!==1||monthReads!==1||dayReads!==0||weekdayReads!==0)throw Error('unneeded getter');
console.log('ok');"#
    );
    assert_eq!(execute(&script), "ok");
}

#[test]
fn compiled_week_year_fixture_preserves_readable_checked_javascript() {
    let output = runtime("en-GB", "Y'|'YY'|'YYYY'|'y'|'G'|'MMM'|'QQ", true);
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-date/week-year.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &output).unwrap();
    }
    assert_eq!(output, std::fs::read_to_string(snapshot).unwrap());
}
