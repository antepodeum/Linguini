<script lang="ts">
  let additionalCopy = $state<typeof import('$lib/deferred-lab-copy.js').additionalCopy>();
  const additionalText = $derived(additionalCopy?.());

  async function loadAdditionalCopy() {
    try {
      const {additionalCopy:loadedCopy} = await import('$lib/deferred-lab-copy.js');
      additionalCopy = loadedCopy;
    } catch (error) {
      if (!(error instanceof Error && error.name === 'AbortError')) throw error;
    }
  }
</script>

<button
  type="button"
  class="mt-4 rounded-md border border-border/70 px-3 py-1.5 text-sm text-foreground hover:border-primary"
  onclick={loadAdditionalCopy}
>Load additional messages</button>
{#if additionalText}
  <output aria-label="Additional copy" class="mt-3 block text-sm text-muted-foreground">
    <span data-additional-title>{additionalText.title}</span>
    <span data-additional-summary class="block">{additionalText.summary}</span>
  </output>
{/if}
