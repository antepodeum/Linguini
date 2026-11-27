import { String as classify } from "./plural-name.js";
import * as classifier from "./plural-name.js";

/** @type {string[]} */
const results = [classify(1), classify(2n), classify("1.00"), classify("few")];
// @ts-expect-error Numeric and category inputs retain their public contract.
classify({ value: 1 });
// @ts-expect-error A plural input remains required.
classify();
// @ts-expect-error A compiler binding is not a public export.
classifier.__lgl_name_$String;
void results;
