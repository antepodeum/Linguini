import messages, { formatNumber, normalizeMessageArgs, lgl, String as strings, main, pluralEn } from "./locales/en.js";
import * as module from "./locales/en.js";

formatNumber.number({ value: 1234.5 });
normalizeMessageArgs.hello("Name");
const text: string = lgl.hello;
strings.number("first");
pluralEn.count(2);
void [messages.main.main, main.main, text];
// @ts-expect-error Source keys stay exact.
formatNumber.number({ wrong: 2 });
// @ts-expect-error Source enums keep finite values.
strings.number("second");
// @ts-expect-error Compiler bindings are not additional public exports.
module.__lgl_name_$formatNumber;
