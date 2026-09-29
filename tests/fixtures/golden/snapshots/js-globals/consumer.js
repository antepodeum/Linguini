import {
  Render,
  __lgl_form_4672756974 as Fruit,
  prefix,
} from "./locales/en-us/_globals.js";

/**
 * @param {unknown} actual
 * @param {unknown} expected
 */
function assertEqual(actual, expected) {
  if (actual !== expected) {
    throw new Error(`Expected ${String(expected)}, received ${String(actual)}`);
  }
}

assertEqual(prefix, "Total");
assertEqual(Render("male", 1), "One");
assertEqual(Render("male", 2), "Many");
assertEqual(Render("other", 1), "Other one");
assertEqual(Render("other", 2), "Other many");
assertEqual(Fruit.apple.label(1), "One Total");
assertEqual(Fruit.apple.label(2), "Many Total");

if (false) {
  // @ts-expect-error locale enum variants stay closed in checked JavaScript
  Render("unknown", 1);
  // @ts-expect-error plural inputs reject unrelated values
  Render("male", true);
}
