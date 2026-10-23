import { page } from '$app/state';
import { goto } from '$app/navigation';

export function getRequestLocale(): unknown {
  return page.data.linguini?.locale;
}

export type NavigationState = App.PageState;
export function navigate(href: string, options: Parameters<typeof goto>[1]): Promise<void> {
  return goto(href, options);
}
