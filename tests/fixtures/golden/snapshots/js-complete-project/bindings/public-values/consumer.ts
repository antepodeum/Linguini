import messages, { formatNumber, normalizeMessageArgs, message, String as stringMessage, lgl } from "./locales/en.js";
import * as module from "./locales/en.js";

formatNumber({ value: 1234.5 });
normalizeMessageArgs(2n);
message(2);
stringMessage("first");
const text: string = lgl;
void [messages.lgl, text];
// @ts-expect-error Source named keys remain exact.
formatNumber({ wrong: 2 });
// @ts-expect-error Source enum values remain finite.
stringMessage("second");
// @ts-expect-error Internal bindings are not additional public exports.
module.__lgl_name_$formatNumber;
