import type {LocaleSetter,LocaleState,LocaleStateOptions} from './types.js';

/** Construct locale preparation/publication without creating a message provider. */
export function createLocaleSetter<Locale extends string>(state: LocaleState<Locale>, options: Pick<LocaleStateOptions,'getRequestIdentity'> = {}): LocaleSetter<Locale> {
  let latestChange = 0;
  return async function setLocale(value:unknown) {
    const browser = typeof window !== 'undefined';
    const change = browser ? ++latestChange : 0;
    const request = browser ? options.getRequestIdentity?.() : undefined;
    const locale = await state.prepareLocale(value);
    if (browser && (change !== latestChange
      || (options.getRequestIdentity && options.getRequestIdentity() !== request))) {
      return state.getCurrentLocale();
    }
    return state.setCurrentLocale(locale);
  };
}
