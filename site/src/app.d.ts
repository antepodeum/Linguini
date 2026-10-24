import type { LinguiniLocaleRequestContext } from '@antepodeum/linguini-web';
import type { SerializedLinguiniContext } from '@antepodeum/linguini-sveltekit';
import type { Locale } from './lib/generated/linguini/locale.js';

// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals { linguini?: LinguiniLocaleRequestContext<Locale> }
		interface PageData { linguini?: SerializedLinguiniContext<Locale> }
		// interface PageState {}
		// interface Platform {}
	}
}

declare module '*.png' {
	const src: string;
	export default src;
}

export {};
