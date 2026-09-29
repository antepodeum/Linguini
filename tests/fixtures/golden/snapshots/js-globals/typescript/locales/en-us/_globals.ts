import type { Fruit } from "../../shared";
import { selectBranch } from "../../shared";
import { pluralEnUs } from "./_runtime";

type Gender = "male" | "other";

const prefix = "Total";

const __lgl_form_4672756974 = {
  apple: { Gender: "male", label: (value: number | bigint | string) => selectBranch(pluralEnUs(value), { one: "One " + String(prefix), _: "Many " + String(prefix) }) },
} as const;

function Render(__lgl_p0: Gender, __lgl_p1: number | bigint | string): string {
  return selectBranch(String(__lgl_p0), { male: (): string => selectBranch(pluralEnUs(__lgl_p1), { one: (): string => "One", _: (): string => "Many" })(), other: (): string => selectBranch(pluralEnUs(__lgl_p1), { one: (): string => "Other one", _: (): string => "Other many" })() })();
}

export type { Gender };
export { prefix, __lgl_form_4672756974, Render };
