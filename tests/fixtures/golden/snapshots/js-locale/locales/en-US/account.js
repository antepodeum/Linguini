import { normalizeMessageArgs } from "../../shared.js";

export const account = {
  label: "Account",
  personalized: /** @type {((name: string) => string) & ((args: { name: string }) => string)} */ ((...__lgl_args) => { const [name] = normalizeMessageArgs(__lgl_args, ["name"]); return "Welcome " + String(name); }),
  nested: {
    status: "Ready",
  },
};

const lgl = {
  account,
};

export default lgl;
//# sourceMappingURL=account.js.map
