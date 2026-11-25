import type {MessageRuntime,LocaleState,LinguiniRune,LinguiniRuneOptions} from './types.js';
import {createLocaleSetter} from './locale-change.js';
export {createLocaleSetter} from './locale-change.js';
export type * from './types.js';
export {createLocaleState} from './state.svelte.js';

export function createLinguiniRune<Locale extends string, Messages>(runtime: MessageRuntime<Locale, Messages>, state: LocaleState<Locale>, options: LinguiniRuneOptions<Locale> = {}): LinguiniRune<Locale, Messages> {
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
    setLocale: options.setLocale ?? createLocaleSetter(state,options),
  };
}
