export type __lgl_name_6D61696E2E4672756974 = "apple" | "pear" | "orange";

export type __lgl_name_6D61696E2E53697A65 = "small" | "big";

export type __lgl_name_6D61696E2E4D6F6E6579 = number;

export type __lgl_name_6D61696E2E53686F727444617465 = Date | number | string;

export type __lgl_name_6D61696E2E4D6561737572656D656E74 = number;

export function selectBranch(
  key: string,
  branches: Record<string, string>,
): string {
  if (Object.prototype.hasOwnProperty.call(branches, key)) {
    return branches[key];
  }
  if (Object.prototype.hasOwnProperty.call(branches, "_")) {
    return branches._;
  }
  if (Object.prototype.hasOwnProperty.call(branches, "other")) {
    return branches.other;
  }
  throw new Error(`Linguini dispatch has no branch for key ${JSON.stringify(key)}`);
}
