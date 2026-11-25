export interface LocaleStateOptions {
  getRequestLocale?: () => unknown;
  /** Identity of the authoritative request context; changing it invalidates a client override. */
  getRequestIdentity?: () => unknown;
}
export interface LocaleRuntime<Locale extends string> {
  readonly baseLocale: Locale;
  readonly locales: readonly Locale[];
  normalizeLocale(value: unknown): Locale | undefined;
  getTextDirection(locale: Locale): 'ltr'|'rtl';
}
export interface LocaleState<Locale extends string> {
  getCurrentLocale(): Locale;
  initializeCurrentLocale(value: unknown): Locale;
  setCurrentLocale(value: unknown): Locale;
  clearCurrentLocaleOverride(): void;
  prepareLocale(value: unknown): Promise<Locale>;
  registerLocaleLoader(loader: (locale: Locale) => void|Promise<void>): () => void;
}
export interface MessageRuntime<Locale extends string, Messages> extends LocaleRuntime<Locale> {
  createLinguiniProvider(options: {getLocale: () => Locale}): Messages;
}
export type LocaleSetter<Locale extends string> = (value:unknown) => Promise<Locale>;
export interface LinguiniRuneOptions<Locale extends string> extends Pick<LocaleStateOptions,'getRequestIdentity'> {
  setLocale?: LocaleSetter<Locale>;
}
export interface LinguiniRune<Locale extends string, Messages> {
  readonly messages: Messages;
  readonly l: Messages;
  readonly locale: Locale;
  readonly lang: Locale;
  readonly direction: 'ltr'|'rtl';
  readonly textDirection: 'ltr'|'rtl';
  readonly htmlAttrs: {lang:Locale;dir:'ltr'|'rtl'};
  setLocale: LocaleSetter<Locale>;
}
