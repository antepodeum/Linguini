import { normalizeMessageArgs } from "../../shared.js";

/** @typedef {import("../../shared.js").__lgl_name_String$Choice} __lgl_name_String$Choice */

const __lgl_keys_String$number = ["value"];
const __lgl_name_$String = {
  number: /** @type {((value: __lgl_name_String$Choice) => string) & ((args: { value: __lgl_name_String$Choice }) => string)} */ ((...__lgl_args) => { const [value] = /** @type {[value: __lgl_name_String$Choice]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_String$number)); return String(value); }),
};
export { __lgl_name_$String as String };

const lgl = {
  String: __lgl_name_$String,
};

export default lgl;
//# sourceMappingURL=String.js.map
