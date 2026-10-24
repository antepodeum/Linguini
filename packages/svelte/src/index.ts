import type {MessageRuntime,LocaleState,LinguiniRune} from './types.js';
export type * from './types.js';
export {createLocaleState} from './state.svelte.js';

export function createLinguiniRune<Locale extends string, Messages>(runtime: MessageRuntime<Locale, Messages>, state: LocaleState<Locale>): LinguiniRune<Locale, Messages> {
  const messages = runtime.createLinguiniProvider({getLocale:()=>state.getCurrentLocale()});
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
    async setLocale(value:unknown) {return state.setCurrentLocale(await state.prepareLocale(value));},
  };
}
