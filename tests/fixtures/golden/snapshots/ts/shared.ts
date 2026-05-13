export function selectBranch(
  key: string,
  branches: Record<string, string>,
): string {
  return branches[key] ?? branches.other ?? "";
}
