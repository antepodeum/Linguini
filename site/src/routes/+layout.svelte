<script lang="ts">
	import { base } from '$app/paths';
	import '../styles/app.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={`${base}/favicon.svg`} />
</svelte:head>

{@render children()}
