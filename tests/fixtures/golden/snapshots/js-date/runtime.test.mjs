import assert from "node:assert/strict";
import test from "node:test";
import { formatFlexible, formatFixed, formatNarrow } from "./date.js";

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
