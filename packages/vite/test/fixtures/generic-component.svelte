<!-- 😀 <script>not a script scope</script> -->
<script module>
  export const marker = 'module';
</script>

<style>
  /* <script>not a script scope</script> */
  a { color: var(--fg); }
</style>

<div data-example="<script>not a script scope</script>"></div>

<script lang="ts" generics="T extends { text: string; nested: Array<' module '> }">
  import { l } from './catalog.js';
  let { value }: { value: T } = $props();
  const title = l.main.title;
</script>

<a href="/account" title="2 > 1">{title} {value.text}</a>
