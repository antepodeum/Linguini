export type FormatterOptions = Record<string, string>;

export declare function formatCurrency(
  value: number | string,
  locale: string,
  options?: FormatterOptions,
): string;

export declare function formatDate(
  value: Date | number | string,
  locale: string,
  options?: FormatterOptions,
): string;

export declare function selectBranch(
  key: string,
  branches: Record<string, string>,
): string;
