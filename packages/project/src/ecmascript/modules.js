import { defineModule } from "../modules.js";

export function web(options = {}) { return defineModule("ecmascript.web", options); }
export function svelte() { if (arguments.length) throw new TypeError("svelte accepts no options"); return defineModule("svelte"); }
export function svelteKit() { if (arguments.length) throw new TypeError("svelteKit accepts no options"); return defineModule("sveltekit"); }
