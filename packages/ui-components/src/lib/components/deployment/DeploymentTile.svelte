<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';

	export let orderName: string;
	export let key: string;
	export let name: string;
	export let description: string;

	let customRegistryParam = '';

	onMount(async () => {
		// Get the custom registry from URL if it exists
		customRegistryParam = $page.url.searchParams.get('registry')
			? `?registry=${$page.url.searchParams.get('registry')}`
			: '';
	});
</script>

<a
	href={`/deploy/${orderName}/${key}${customRegistryParam}`}
	class="flex h-full w-full max-w-sm cursor-pointer flex-col gap-y-4 rounded-3xl border border-gray-200 bg-white p-4 text-left hover:bg-gray-100 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
>
	<h1 class="text-2xl font-semibold text-gray-900 dark:text-white">{name}</h1>
	<p class="text-gray-600 dark:text-gray-400">{description}</p>
</a>
