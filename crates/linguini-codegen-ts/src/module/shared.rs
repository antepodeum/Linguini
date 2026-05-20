pub fn emit_shared(output: &mut String) {
    output.push_str(
        r#"export type FormatterWidth = "full" | "long" | "medium" | "short";

export type NumberFormatterOptions = Record<string, never>;

export type CurrencyFormatterOptions = {
  code?: string;
  accounting?: "true" | "false";
};

export type DateFormatterOptions = {
  style?: FormatterWidth;
};

export type FormatterOptions = NumberFormatterOptions | CurrencyFormatterOptions | DateFormatterOptions;

export type CldrFormatWidths = Record<FormatterWidth, string>;

export type CldrFormatterData = {
  locale: string;
  numbers?: {
    decimalSymbol: string;
    groupSymbol: string;
    decimalPattern: string;
    percentPattern: string;
  };
  currency?: {
    standardPattern: string;
    accountingPattern?: string;
  };
  date?: {
    dateFormats: CldrFormatWidths;
    timeFormats: CldrFormatWidths;
    dateTimeFormats: CldrFormatWidths;
  };
};

export function formatNumber(
  value: number | string,
  data: CldrFormatterData,
): string {
  return formatDecimal(Number(value), data);
}

export function formatCurrency(
  value: number | string,
  data: CldrFormatterData,
  options: CurrencyFormatterOptions = {},
): string {
  const currency = options.code ?? "USD";
  const pattern = options.accounting === "true"
    ? data.currency?.accountingPattern ?? data.currency?.standardPattern
    : data.currency?.standardPattern;
  return applyNumberPattern(formatDecimal(Number(value), data, 2), pattern, currencySymbol(currency, data.locale));
}

export function formatDate(
  value: Date | number | string,
  data: CldrFormatterData,
  options: DateFormatterOptions = {},
): string {
  if (typeof value === "string") return value;
  const style = options.style ?? "medium";
  const pattern = data.date?.dateFormats[style];
  if (!pattern) return new Intl.DateTimeFormat(data.locale).format(value);
  return new Intl.DateTimeFormat(data.locale, { dateStyle: style }).format(value);
}

function formatDecimal(value: number, data: CldrFormatterData, fractionDigits?: number): string {
  const fixed = typeof fractionDigits === "number" ? value.toFixed(fractionDigits) : String(value);
  const [integer, fraction] = fixed.split(".");
  const grouped = integer.replace(/\B(?=(\d{3})+(?!\d))/g, data.numbers?.groupSymbol ?? ",");
  return fraction ? `${grouped}${data.numbers?.decimalSymbol ?? "."}${fraction}` : grouped;
}

function applyNumberPattern(value: string, pattern: string | undefined, currency: string): string {
  if (!pattern) return `${currency} ${value}`;
  return pattern.replace(/[#0.,]+/, value).replace("¤", currency);
}

function currencySymbol(currency: string, locale: string): string {
  return new Intl.NumberFormat(locale, { style: "currency", currency }).formatToParts(0).find((part) => part.type === "currency")?.value ?? currency;
}

export function selectBranch(
  key: string,
  branches: Record<string, string>,
): string {
  return branches[key] ?? branches._ ?? branches.other ?? "";
}
"#,
    );
}
