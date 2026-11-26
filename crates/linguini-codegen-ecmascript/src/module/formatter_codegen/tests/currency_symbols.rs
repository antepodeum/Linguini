use super::*;

#[test]
fn currency_separators_are_selected_without_changing_decimal_output() {
    for (locale, number, standard, accounting) in [
        (
            "de-AT",
            "1\u{a0}234,5",
            "$\u{a0}1.234,50",
            "-$\u{a0}1.234,50",
        ),
        (
            "fr-CH",
            "1'234,5",
            "1'234.50\u{a0}$US",
            "(1'234.50\u{a0}$US)",
        ),
        ("en", "1,234.5", "$1,234.50", "($1,234.50)"),
    ] {
        let runtime = formatter_data_declaration(
            locale,
            &[
                request(IrFormatterKind::Number, &[]),
                request(IrFormatterKind::Currency, &[]),
                request(IrFormatterKind::Currency, &[("accounting", "true")]),
            ],
            false,
        );
        let script = format!(
            r#"{runtime}
import assert from 'node:assert/strict';
Intl.NumberFormat=class {{constructor(){{throw Error('no host formatter');}}}};
for(const value of [1234.5,'1234.5']) {{
  assert.equal(formatNumber(value),{number:?});
  assert.equal(formatCurrency(value),{standard:?});
  assert.equal(formatCurrency_accounting_true('-1234.5'),{accounting:?});
}}
console.log('ok');
"#
        );
        assert_eq!(execute(&script), "ok", "{locale}");
        assert!(!runtime.contains("currencyDecimal"), "{locale}");
        assert!(!runtime.contains("currencyGroup"), "{locale}");
    }
}
