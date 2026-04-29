<script lang="ts">
	import type { InvalidOrderDetail } from '$lib/types/order';
	export let ordersWithErrors: InvalidOrderDetail[];
</script>

<div
	class="flex flex-col gap-4 rounded-xl bg-red-100 p-6 dark:bg-red-900"
	data-testid="invalid-orders"
>
	<h2 class="text-xl font-semibold text-gray-900 dark:text-white">Invalid orders in rainlang</h2>
	<div class="flex flex-col gap-2">
		{#each ordersWithErrors as order}
			<div class="flex flex-col gap-1">
				<span class="font-medium">{order.name}</span>
				<span class="text-red-600 dark:text-red-400">{order.error}</span>
			</div>
		{/each}
	</div>
</div>
