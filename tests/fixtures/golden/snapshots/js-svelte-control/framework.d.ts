/// <reference path="../js-svelte-effects/framework.d.ts" />

declare namespace App { interface PageState { tab?: string; } }
declare module "$app/navigation" {
  export function goto(href: string, options?: {
    replaceState?: boolean; invalidateAll?: boolean; keepFocus?: boolean;
    noScroll?: boolean; state?: App.PageState;
  }): Promise<void>;
}
