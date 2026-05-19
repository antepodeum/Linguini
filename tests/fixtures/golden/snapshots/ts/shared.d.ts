export type FormatterWidth = "full" | "long" | "medium" | "short";

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

export declare function formatNumber(
  value: number | string,
  data: CldrFormatterData,
): string;

export declare function formatCurrency(
  value: number | string,
  data: CldrFormatterData,
  options?: CurrencyFormatterOptions,
): string;

export declare function formatDate(
  value: Date | number | string,
  data: CldrFormatterData,
  options?: DateFormatterOptions,
): string;

export declare function selectBranch(
  key: string,
  branches: Record<string, string>,
): string;
