import {l} from '$lib/generated/linguini/svelte';

export function additionalDetails() {
  return {
    detail:l.deferred_lab.detail,
    parent:l.deferred_lab.title,
    summary:l.deferred_lab.summary({value:1299.5}),
  };
}
