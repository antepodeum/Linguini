use super::*;

#[test]
fn native_integer_inputs_skip_decimal_regexes_and_keep_bounds_and_negative_zero() {
    let runtime = formatter_data_declaration(
        "en",
        &[
            request(IrFormatterKind::Number, &[]),
            request(IrFormatterKind::Currency, &[]),
        ],
        false,
    );
    let script = format!(
        r#"{runtime}
import assert from 'node:assert/strict';
const huge='9'.repeat(8192),tooLarge=BigInt('9'.repeat(8193));
const cases=[
  [0,false,'0'],[-0,true,'0'],[1,false,'1'],[-1,true,'1'],
  [Number.MAX_SAFE_INTEGER,false,'9007199254740991'],[-Number.MAX_SAFE_INTEGER,true,'9007199254740991'],
  [0n,false,'0'],[1n,false,'1'],[-1n,true,'1'],
  [12345678901234567890n,false,'12345678901234567890'],
  [BigInt(huge),false,huge],[-BigInt(huge),true,huge],
];
const original=RegExp.prototype.exec;
RegExp.prototype.exec=function(){{throw Error('native integer path executed a regex');}};
try {{
  for(const [value,negative,integer] of cases) {{
    assert.deepEqual(parseGeneratedDecimal(value),{{negative,integer,fraction:''}});
  }}
  assert.equal(formatCurrency(1234),'$1,234.00');
  assert.equal(formatCurrency(-0),'-$0.00');
  assert.equal(formatCurrency(1234n),'$1,234.00');
  assert.equal(formatNumber(1234),'1,234');
  assert.equal(formatNumber(-0),'-0');
  assert.equal(formatNumber(1234n),'1,234');
  assert.throws(()=>parseGeneratedDecimal(tooLarge),RangeError);
  assert.throws(()=>parseGeneratedDecimal(-tooLarge),RangeError);
}} finally {{RegExp.prototype.exec=original;}}
assert.equal(formatNumber(-0),'-0');
for(const [value,expected] of [
  [1.25,{{negative:false,integer:'1',fraction:'25'}}],
  ['001.2500',{{negative:false,integer:'1',fraction:'25'}}],
  [1e-7,{{negative:false,integer:'0',fraction:'0000001'}}],
  [9007199254740992,{{negative:false,integer:'9007199254740992',fraction:''}}],
  [1e21,{{negative:false,integer:'1'+'0'.repeat(21),fraction:''}}],
]) assert.deepEqual(parseGeneratedDecimal(value),expected);
for(const value of [NaN,Infinity,-Infinity]) {{
  assert.equal(parseGeneratedDecimal(value),undefined);
  assert.equal(formatNumber(value),String(value));
  assert.equal(formatCurrency(value),String(value));
}}
for(const value of [huge+'9','-'+huge+'9','1e8192','--1','++1','-+1','+-1']) assert.throws(()=>formatNumber(value),RangeError);
const hints=[];
const object={{[Symbol.toPrimitive](hint){{hints.push(hint);return '1234.5';}}}};
assert.equal(formatNumber(object),'1,234.5');
assert.deepEqual(hints,['string']);
const failure=Error('caller coercion failed');
assert.throws(()=>formatCurrency({{[Symbol.toPrimitive](){{throw failure;}}}}),error=>error===failure);
console.log('ok');
"#
    );
    assert_eq!(execute(&script), "ok");
}
