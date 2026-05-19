<script lang="ts">
  import { codeToHtml } from 'shiki';

  let {
    code,
    lang = 'typescript',
    class: className = ''
  }: {
    code: string;
    lang?: string;
    class?: string;
  } = $props();

  let html = $state('');

  $effect(() => {
    let cancelled = false;
    codeToHtml(code, {
      lang,
      theme: 'github-dark'
    }).then((highlighted) => {
      if (!cancelled) html = highlighted;
    });
    return () => {
      cancelled = true;
    };
  });
</script>

<div class={['shiki-wrap overflow-x-auto', className]}>{@html html}</div>
