import { normalizeMessageArgs } from "../../shared.js";

/** @typedef {import("../../shared.js").__lgl_name_String$Choice} __lgl_name_String$Choice */

const __lgl_keys_normalizeMessageArgs$hello = ["name"];
const __lgl_name_$normalizeMessageArgs = {
  hello: /** @type {((name: string) => string) & ((args: { name: string }) => string)} */ ((...__lgl_args) => { const [name] = /** @type {[name: string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_normalizeMessageArgs$hello)); return "Hello " + String(name); }),
};
export { __lgl_name_$normalizeMessageArgs as normalizeMessageArgs };

const lgl = {
  normalizeMessageArgs: __lgl_name_$normalizeMessageArgs,
};

export default lgl;
//# sourceMappingURL=normalizeMessageArgs.js.map
