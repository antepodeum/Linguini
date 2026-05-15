import { type Locale } from "./index";

export type LocaleLink = {
  locale: Locale;
  href: string;
  current: boolean;
  lang: string;
  dir: "ltr" | "rtl";
};

export type LocaleLinkOptions = {
  currentLocale?: Locale;
  stripBaseLocale?: boolean;
};

export declare function localeLinks(pathname: string, options?: LocaleLinkOptions): LocaleLink[];
export declare function staticLocaleEntries(paths: readonly string[], options?: LocaleLinkOptions): string[];
export declare function htmlLocaleAttributes(locale?: Locale): { readonly lang: Locale; readonly dir: "ltr" | "rtl" };
