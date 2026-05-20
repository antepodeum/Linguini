<script lang="ts">
  import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';

  type ButtonProps = HTMLButtonAttributes & HTMLAnchorAttributes & {
    href?: string;
    variant?: 'primary' | 'secondary' | 'ghost';
    size?: 'sm' | 'md';
  };

  let {
    href,
    variant = 'primary',
    size = 'md',
    class: className,
    children,
    ...rest
  }: ButtonProps = $props();

  const classes = $derived(
    cn(
      'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg border text-sm font-medium transition duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50',
      size === 'sm' ? 'h-9 px-4' : 'h-11 px-5',
      variant === 'primary' &&
        'border-primary bg-primary text-primary-foreground hover:bg-primary/90',
      variant === 'secondary' &&
        'border-border bg-muted text-foreground hover:border-primary/40 hover:bg-muted/80',
      variant === 'ghost' &&
        'border-transparent text-muted-foreground hover:bg-muted hover:text-foreground',
      className
    )
  );
</script>

{#if href}
  <a {href} class={classes} {...rest}>
    {@render children?.()}
  </a>
{:else}
  <button class={classes} {...rest}>
    {@render children?.()}
  </button>
{/if}
