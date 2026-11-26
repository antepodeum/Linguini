import assert from "node:assert/strict";
import test from "node:test";
import { formatFlexible, formatFixed, formatNarrow, formatYear, formatLocalized } from "./date.js";
import { formatDate_skeleton_yQQQ } from "./skeletons.js";

test("compiled periods follow displayed precision and localized rules", () => {
  assert.equal(formatFlexible("2024-01-02T09:30:00Z"), "9:30:00.000 in the morning");
  assert.equal(formatFlexible("2024-01-02T12:00:00Z"), "12:00:00.000 noon");
  assert.equal(formatFlexible("2024-01-02T12:00:00.001Z"), "12:00:00.001 in the afternoon");
  assert.equal(formatFlexible("2024-01-02T00:00:00Z"), "12:00:00.000 midnight");
  assert.equal(formatFlexible("2024-01-02T21:00:00Z"), "9:00:00.000 at night");
  assert.equal(formatFixed("2024-01-02T12:00:59Z"), "12:00 полдень");
  assert.equal(formatFixed("2024-01-02T12:01:00Z"), "12:01 PM");
  assert.equal(formatFixed("2024-01-02T00:00:00Z"), "00:00 полночь");
  assert.equal(formatNarrow("2024-01-02T13:30:00Z"), "بعد الظهر");
});

test("Gregorian years use the era and requested minimum widths", () => {
  assert.equal(formatYear("0001-01-02"), "1 01 0001 AD");
  assert.equal(formatYear("0000-01-02"), "1 01 0001 BC");
  const before = new Date(0);
  before.setUTCFullYear(-1, 0, 2);
  assert.equal(formatYear(before), "2 02 0002 BC");
});

test("localized date fields reuse one compiled digit callback", () => {
  const original = String.prototype.replace;
  const callbacks = new Set();
  let fields = 0;
  String.prototype.replace = function(pattern, replacement) {
    if (pattern instanceof RegExp && pattern.source === '\\d' && pattern.flags === 'g') {
      fields++;
      callbacks.add(replacement);
    }
    return original.call(this, pattern, replacement);
  };
  try {
    const text = '2024-01-02T13:04:05.006Z';
    for (const input of [text, new Date(text), Date.parse(text)]) {
      assert.equal(formatLocalized(input), '٢٠٢٤ ٠١ ٠٢ ١٣:٠٤:٠٥.٠٠٦');
    }
  } finally {
    String.prototype.replace = original;
  }
  assert.equal(fields, 3);
  assert.equal(callbacks.size, 1);
});

test("quarter skeletons follow UTC boundaries and the Date input contract", () => {
  for (const [value, expected] of [
    ["2024-03-31T23:59:59.999Z", "Q1 2024"],
    ["2024-04-01T00:00:00Z", "Q2 2024"],
    ["2024-07-01", "Q3 2024"],
    ["2024-10-01", "Q4 2024"],
    ["2024-04-01T00:30:00+01:00", "Q1 2024"],
    ["0000-04-01", "Q2 1"],
  ]) {
    assert.equal(formatDate_skeleton_yQQQ(value), expected);
  }
  const value = new Date(0);
  let reads = 0;
  value.getUTCMonth = () => { reads++; return 6; };
  assert.equal(formatDate_skeleton_yQQQ(value), "Q3 1970");
  assert.equal(reads, 1);
  assert.equal(formatDate_skeleton_yQQQ(0), "Q1 1970");
  assert.throws(() => formatDate_skeleton_yQQQ("2024-02-30"), RangeError);
});
