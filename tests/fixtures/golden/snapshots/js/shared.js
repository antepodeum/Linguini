export function formatCurrency(value, locale, options = {}) {
  const currency = options.code ?? "USD";
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency,
  }).format(Number(value));
}

export function formatDate(value, locale, options = {}) {
  if (typeof value === "string") return value;
  const intlOptions = {};
  if (options.style) intlOptions.dateStyle = options.style;
  return new Intl.DateTimeFormat(locale, intlOptions).format(value);
}

export function selectBranch(key, branches) {
  return branches[key] ?? branches._ ?? branches.other ?? "";
}
