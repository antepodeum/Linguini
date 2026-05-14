export type FormatterOptions = Record<string, string>;

export function formatCurrency(
  value: number | string,
  locale: string,
  options: FormatterOptions = {},
): string {
  const currency = options.code ?? "USD";
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency,
  }).format(Number(value));
}

export function formatDate(
  value: Date | number | string,
  locale: string,
  options: FormatterOptions = {},
): string {
  if (typeof value === "string") return value;
  const intlOptions: Record<string, string> = {};
  if (options.style) intlOptions.dateStyle = options.style;
  return new Intl.DateTimeFormat(locale, intlOptions as Intl.DateTimeFormatOptions).format(value);
}

export function selectBranch(
  key: string,
  branches: Record<string, string>,
): string {
  return branches[key] ?? branches._ ?? branches.other ?? "";
}
