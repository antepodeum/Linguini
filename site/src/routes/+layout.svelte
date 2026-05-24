<script lang="ts">
	import '../styles/app.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" type="image/png" href="/icons/favicon-96x96.png?v=1" sizes="96x96" />
	<link rel="icon" type="image/svg+xml" href="/icons/favicon.svg?v=1" />
	<link rel="shortcut icon" href="/icons/favicon.ico?v=1" />
</svelte:head>

{@render children()}
