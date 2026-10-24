import { normalizeMessageArgs } from "../../shared.js";

const __lgl___lgl_name_account$personalized_keys = ["name"];
export const account = {
  label: "Account",
  personalized: /** @type {((name: string) => string) & ((args: { name: string }) => string)} */ ((...__lgl_args) => { const [name] = /** @type {[name: string]} */ (normalizeMessageArgs(__lgl_args, __lgl___lgl_name_account$personalized_keys)); return "Welcome " + String(name); }),
  nested: {
    status: "Ready",
  },
};

const lgl = {
  account,
};

export default lgl;
//# sourceMappingURL=account.js.map
