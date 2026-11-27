import { normalizeMessageArgs } from "../../shared.js";
import { __lgl_name_pluralEn$Count } from "./_globals.js";

/** @typedef {import("../../shared.js").__lgl_name_String$Choice} __lgl_name_String$Choice */

const __lgl_keys_pluralEn$count = ["value"];
const __lgl_name_$pluralEn = {
  count: /** @type {((value: number | bigint | string) => string) & ((args: { value: number | bigint | string }) => string)} */ ((...__lgl_args) => { const [value] = /** @type {[value: number | bigint | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_pluralEn$count)); return __lgl_name_pluralEn$Count(value); }),
};
export { __lgl_name_$pluralEn as pluralEn };

const lgl = {
  pluralEn: __lgl_name_$pluralEn,
};

export default lgl;
//# sourceMappingURL=pluralEn.js.map
