import type { LinguiniRequestContext, LinguiniRuntime, LinguiniWeb, LinguiniWebEnvironment, LinguiniWebOptions } from "./types.js";
import { createWebLocaleI18n } from './locale.js';
import { inputFromRequest } from './request.js';
import { createLocaleRequestContext } from './context.js';

export function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions = {}, environment: LinguiniWebEnvironment = {}): LinguiniWeb<Locale, Linguini> {
    const web = createWebLocaleI18n(runtime, options, environment);

    function createRequestContext(locale: Locale, contextInput: Record<string, unknown> = {}): LinguiniRequestContext<Locale, Linguini> {
        const resolved = web.matchLocale(locale) ?? runtime.baseLocale;
        const messages = runtime.createLinguini(resolved);
        return {...createLocaleRequestContext(web, resolved, contextInput), messages, l: messages};
    }
    return {
        ...web,
        createLinguini: runtime.createLinguini,
        resolveRequest: async (request, input = {}) => {
            const requestInput = inputFromRequest(request, input);
            const locale = web.resolveLocaleSync(requestInput);
            await runtime.prepareLinguini?.(locale);
            return createRequestContext(locale, requestInput);
        },
        createRequestContext,
    };
}
