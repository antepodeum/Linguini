import { normalizeMessageArgs } from "../../shared";

export const account = {
  label: "Account",
  personalized: (...__lgl_args: [name: string] | [args: { name: string }]) => { const [name] = normalizeMessageArgs(__lgl_args, ["name"]) as [string]; return "Welcome " + String(name); },
  nested: {
    status: "Ready",
  },
} as const;

const lgl = {
  account,
} as const;

export default lgl;
