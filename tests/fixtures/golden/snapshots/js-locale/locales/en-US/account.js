import { normalizeMessageArgs } from "../../shared.js";

const __lgl_keys_account$personalized = ["name"];
export const account = {
  label: "Account",
  personalized: /** @type {((name: string) => string) & ((args: { name: string }) => string)} */ ((...__lgl_args) => { const [name] = /** @type {[name: string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_account$personalized)); return "Welcome " + String(name); }),
  nested: {
    status: "Ready",
  },
};

const lgl = {
  account,
};

export default lgl;
//# sourceMappingURL=account.js.map
