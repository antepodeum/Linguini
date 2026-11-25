import {l} from '$lib/generated/linguini/svelte';

export function additionalCopy() {
  return {
    title:l.deferred_lab.title,
    summary:l.deferred_lab.summary({value:1299.5}),
  };
}

export function unusedDeferredCopy() {
  return l.deferred_lab.unused;
}
