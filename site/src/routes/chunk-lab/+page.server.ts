import type {PageServerLoad} from './$types';
import {building} from '$app/environment';

export const load: PageServerLoad = async ({url}) => {
  if (building || !url.searchParams.has('deferred-ssr')) return {};
  const {additionalCopy} = await import('$lib/deferred-lab-copy.js');
  return {deferredSsr:additionalCopy()};
};
