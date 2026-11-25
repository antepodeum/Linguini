import * as catalog from '$lib/generated/linguini/svelte';

export function additionalDetails() {
  return {
    detail:catalog.l.deferred_lab.detail,
    parent:catalog.l.deferred_lab.title,
    summary:catalog.l.deferred_lab.summary({value:1299.5}),
  };
}
