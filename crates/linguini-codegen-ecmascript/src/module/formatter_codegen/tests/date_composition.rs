use super::*;

#[test]
fn composed_dates_preserve_inserted_literals_quotes_and_numeric_field_observations() {
    for (locale, year, time) in [("en", "2027", "02:45"), ("ar-EG", "٢٠٢٧", "٠٢:٤٥")] {
        let mut dates = linguini_cldr::compiled_date_formatting(locale).unwrap();
        dates.date_formats.long = "'date {0}' y";
        dates.time_formats.short = "HH:mm 'time {1}'";
        for (composition, expected) in [
            (
                "{1} / {0} / {1}",
                format!("date {{0}} {year} / {time} time {{1}} / date {{0}} {year}"),
            ),
            (
                "'o''clock' — {0}; {1}",
                format!("o'clock — {time} time {{1}}; date {{0}} {year}"),
            ),
        ] {
            dates.date_time_formats.long = composition;
            let (source, helpers) = generated_date_request(
                locale,
                &dates,
                &request(
                    IrFormatterKind::Date,
                    &[("style", "long"), ("time_style", "short")],
                ),
                "formatComposed",
                false,
            );
            let mut script = format!("{source}{}", date_formatter_helpers(helpers.padding));
            if helpers.digits {
                script.push_str(&digit_formatter_helper(dates.digits));
            }
            script.push_str(&format!(
                r#"
import assert from 'node:assert/strict';
const text = '2026-12-31T23:45-03:00';
const date = new Date(text);
const timestamp = date.getTime();
const getters = ['getTime', 'getUTCFullYear', 'getUTCHours', 'getUTCMinutes'];
const reads = Object.fromEntries(getters.map(name => [name, 0]));
for (const name of getters) {{
  const original = date[name];
  date[name] = function() {{reads[name]++; return original.call(this);}};
}}
for (const input of [date, timestamp, text]) {{
  assert.equal(formatComposed(input), {expected:?});
}}
assert.deepEqual(Object.values(reads), [1, 1, 1, 1]);
const failure = new Error('supplied date getter');
date.getUTCFullYear = () => {{throw failure;}};
assert.throws(() => formatComposed(date), error => error === failure);
console.log('ok');
"#
            ));
            assert_eq!(execute(&script), "ok", "{locale}: {composition}");
        }
    }
}
