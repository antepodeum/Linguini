import type { Locale } from "./locale";

export declare function getCurrentLocale(): Locale;
export declare function initializeCurrentLocale(locale: unknown): Locale;
export declare function setCurrentLocale(locale: unknown): Locale;
export declare function clearCurrentLocaleOverride(): void;
