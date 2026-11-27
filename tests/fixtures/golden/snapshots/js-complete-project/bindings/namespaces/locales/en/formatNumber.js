import { normalizeMessageArgs } from "../../shared.js";
import { formatNumber } from "./_runtime.js";

/** @typedef {import("../../shared.js").__lgl_name_String$Choice} __lgl_name_String$Choice */

const __lgl_keys_formatNumber$number = ["value"];
const __lgl_name_$formatNumber = {
  number: /** @type {((value: number | bigint | string) => string) & ((args: { value: number | bigint | string }) => string)} */ ((...__lgl_args) => { const [value] = /** @type {[value: number | bigint | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_formatNumber$number)); return formatNumber(value); }),
};
export { __lgl_name_$formatNumber as formatNumber };

const lgl = {
  formatNumber: __lgl_name_$formatNumber,
};

export default lgl;
//# sourceMappingURL=formatNumber.js.map
