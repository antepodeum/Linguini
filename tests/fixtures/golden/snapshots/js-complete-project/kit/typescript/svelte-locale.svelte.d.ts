import type { Locale } from "./locale";

export type LinguiniLocaleLoader = (locale: Locale) => void | Promise<void>;

export declare function registerLocaleLoader(loader: LinguiniLocaleLoader): () => void;
export declare function prepareLocale(locale: unknown): Promise<Locale>;
export declare function getCurrentLocale(): Locale;
export declare function initializeCurrentLocale(locale: unknown): Locale;
export declare function setCurrentLocale(locale: unknown): Locale;
export declare function clearCurrentLocaleOverride(): void;
