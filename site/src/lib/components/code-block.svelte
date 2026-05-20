<script lang="ts">
  let {
    html,
    class: className = ''
  }: {
    html: string;
    class?: string;
  } = $props();
</script>

<div class={['shiki-wrap', className]}>{@html html}</div>
