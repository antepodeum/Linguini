import type {MessageRuntime,LocaleState,LinguiniRune,LocaleStateOptions} from './types.js';
export type * from './types.js';
export {createLocaleState} from './state.svelte.js';

export function createLinguiniRune<Locale extends string, Messages>(runtime: MessageRuntime<Locale, Messages>, state: LocaleState<Locale>, options: Pick<LocaleStateOptions,'getRequestIdentity'> = {}): LinguiniRune<Locale, Messages> {
  const messages = runtime.createLinguiniProvider({getLocale:()=>state.getCurrentLocale()});
  let latestChange = 0;
  return {
    messages,
    l: messages,
    get locale() {return state.getCurrentLocale();},
    get lang() {return state.getCurrentLocale();},
    get direction() {return runtime.getTextDirection(state.getCurrentLocale());},
    get textDirection() {return runtime.getTextDirection(state.getCurrentLocale());},
    get htmlAttrs() {
      const locale=state.getCurrentLocale();
      return {lang:locale,dir:runtime.getTextDirection(locale)};
    },
    async setLocale(value:unknown) {
      const browser = typeof window !== 'undefined';
      const change = browser ? ++latestChange : 0;
      const request = browser ? options.getRequestIdentity?.() : undefined;
      const locale = await state.prepareLocale(value);
      if (browser && (change !== latestChange
        || (options.getRequestIdentity && options.getRequestIdentity() !== request))) {
        return state.getCurrentLocale();
      }
      return state.setCurrentLocale(locale);
    },
  };
}
