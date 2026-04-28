<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import type { ValidOrderDetail } from '$lib/types/order';

	export let orders: ValidOrderDetail[];
	let customRegistryParam = '';

	onMount(async () => {
		// Get the custom registry from URL if it exists
		customRegistryParam = $page.url.searchParams.get('registry')
			? `?registry=${$page.url.searchParams.get('registry')}`
			: '';
	});
</script>

<div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3" data-testid="valid-orders">
	{#each orders as order}
		<a
			href={`/deploy/${order.name}${customRegistryParam}`}
			data-testid="order-short-tile"
			class="flex flex-col gap-y-2 rounded-xl border border-gray-200 p-4 hover:bg-gray-50 dark:border-gray-800 dark:hover:bg-gray-900"
		>
			<h3 class="text-2xl font-semibold text-gray-900 dark:text-white">
				{order.details.name}
			</h3>
			<p class="text-lg text-gray-600 dark:text-gray-400">
				{order.details.short_description}
			</p>
		</a>
	{/each}
</div>
