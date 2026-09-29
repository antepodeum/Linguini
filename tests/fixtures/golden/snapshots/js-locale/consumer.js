import locale, { account, root } from "./locales/en-US.js";

/**
 * @param {unknown} actual
 * @param {unknown} expected
 */
function assertEqual(actual, expected) {
  if (actual !== expected) {
    throw new Error(`Expected ${String(expected)}, received ${String(actual)}`);
  }
}

assertEqual(root("Ada"), "Howdy Ada");
assertEqual(root({ name: "Ada" }), "Howdy Ada");
assertEqual(account.label, "Account");
assertEqual(account.personalized("Ada"), "Welcome Ada");
assertEqual(account.personalized({ name: "Ada" }), "Welcome Ada");
assertEqual(account.nested.status, "Ready");
assertEqual(locale.root("Ada"), root("Ada"));
assertEqual(locale.account.personalized("Ada"), account.personalized("Ada"));
assertEqual(locale.account.nested.status, account.nested.status);

if (false) {
  // @ts-expect-error positional parameters retain their schema type
  root(42);
  // @ts-expect-error named arguments reject unknown properties
  root({ name: "Ada", extra: true });
  // @ts-expect-error nested callable leaves retain their schema type
  account.personalized({ name: 42 });
  // @ts-expect-error parameterless leaves remain values
  account.label();
}
