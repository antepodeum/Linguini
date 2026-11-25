import {page} from '$app/state';
import {browser} from '$app/environment';

export function getRequestLocale(): unknown {
  // Route modules can load before Kit publishes page.data during hydration.
  return page.data.linguini?.locale ?? (browser ? document.documentElement.lang : undefined);
}
