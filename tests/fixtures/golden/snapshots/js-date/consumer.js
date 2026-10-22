import { formatFlexible, formatFixed, formatNarrow } from "./date.js";

/** @type {string[]} */
const results = [
  formatFlexible(new Date()),
  formatFixed(0),
  formatNarrow("2024-01-02T13:30:00Z"),
];
// @ts-expect-error formatter inputs retain their public date contract
formatFlexible(false);
// @ts-expect-error an input date remains required
formatNarrow();
void results;
