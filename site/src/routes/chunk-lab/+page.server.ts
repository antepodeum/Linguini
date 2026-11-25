import type {PageServerLoad} from './$types';
import {building} from '$app/environment';

export const load: PageServerLoad = async ({url}) => {
  if (building || !url.searchParams.has('deferred-ssr')) return {};
  const {additionalCopy,loadDetails} = await import('$lib/deferred-lab-copy.js');
  const details = url.searchParams.has('nested-ssr') ? await loadDetails() : undefined;
  return {deferredSsr:additionalCopy(),nestedSsr:details?.()};
};
