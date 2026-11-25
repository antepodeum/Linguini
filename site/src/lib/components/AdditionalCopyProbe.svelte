<script lang="ts">
  let additionalCopy = $state<typeof import('$lib/deferred-lab-copy.js').additionalCopy>();
  let loadDetails = $state<typeof import('$lib/deferred-lab-copy.js').loadDetails>();
  let additionalDetails = $state<typeof import('$lib/deferred-lab-detail.js').additionalDetails>();
  const additionalText = $derived(additionalCopy?.());
  const nestedText = $derived(additionalDetails?.());

  async function loadAdditionalCopy() {
    try {
      const {additionalCopy:loadedCopy,loadDetails:loadedDetails} = await import('$lib/deferred-lab-copy.js');
      additionalCopy = loadedCopy;
      loadDetails = loadedDetails;
    } catch (error) {
      if (!(error instanceof Error && error.name === 'AbortError')) throw error;
    }
  }

  async function loadNestedCopy() {
    if (!loadDetails) return;
    try {
      additionalDetails = await loadDetails();
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
{#if loadDetails}
  <button
    type="button"
    class="mt-4 rounded-md border border-border/70 px-3 py-1.5 text-sm text-foreground hover:border-primary"
    onclick={loadNestedCopy}
  >Load nested messages</button>
{/if}
{#if nestedText}
  <output aria-label="Nested copy" class="mt-3 block text-sm text-muted-foreground">
    <span data-nested-detail>{nestedText.detail}</span>
    <span data-nested-parent class="block">{nestedText.parent}</span>
    <span data-nested-summary class="block">{nestedText.summary}</span>
  </output>
{/if}
