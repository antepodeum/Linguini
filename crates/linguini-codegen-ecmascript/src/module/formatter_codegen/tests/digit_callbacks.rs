use super::*;

#[test]
fn numeric_and_date_formatters_share_one_digit_callback_per_alphabet() {
    for locale in ["en", "ar-EG", "fa", "ff-Adlm", "ccp"] {
        for order in [
            [
                IrFormatterKind::Number,
                IrFormatterKind::Currency,
                IrFormatterKind::Date,
            ],
            [
                IrFormatterKind::Date,
                IrFormatterKind::Currency,
                IrFormatterKind::Number,
            ],
        ] {
            let requests = order.map(|kind| request(kind, &[]));
            let source = formatter_data_declaration(locale, &requests, false);
            let expected_callbacks = usize::from(locale != "en");
            assert_eq!(
                source.matches("function localizeDigit(").count(),
                expected_callbacks
            );
            assert!(!source.contains("(digit) =>"));
            let script = format!(
                r#"
import assert from 'node:assert/strict';
{source}
const expected = [formatNumber('1234.5'), formatCurrency('-0'), formatDate('2024-01-02')];
const original = String.prototype.replace;
const callbacks = new Set();
String.prototype.replace = function(pattern, replacement) {{
  if (pattern instanceof RegExp && pattern.source === '\\d') callbacks.add(replacement);
  return original.call(this, pattern, replacement);
}};
try {{
  for (let index = 0; index < 5; index++) {{
    assert.deepEqual([formatNumber('1234.5'), formatCurrency('-0'), formatDate('2024-01-02')], expected);
  }}
  assert.equal(callbacks.size, {expected_callbacks}, 'digit callbacks must be shared across requests and calls');
}} finally {{String.prototype.replace = original;}}
console.log('ok');
"#
            );
            assert_eq!(execute(&script), "ok", "{locale}");
        }
    }
}

#[test]
fn unused_date_digits_do_not_emit_a_callback() {
    for locale in ["en", "ar-EG", "ff-Adlm"] {
        let source = formatter_data_declaration(
            locale,
            &[request(IrFormatterKind::Date, &[("skeleton", "MMM")])],
            false,
        );
        assert!(!source.contains("localizeDigit"), "{locale}");
        assert!(!source.contains("\\d/g"), "{locale}");
    }
}
