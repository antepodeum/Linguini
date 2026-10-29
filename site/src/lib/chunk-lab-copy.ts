import { l } from '$lib/generated/linguini/svelte';

export function labTagline() { return l.main.hero.tagline; }

// Deliberately shared with consumers that need codegen copy; the lab does not.
export function codegenIntroduction() { return l.main.codegen.intro; }
