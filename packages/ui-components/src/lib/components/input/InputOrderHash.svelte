<script lang="ts">
	import { Label } from 'flowbite-svelte';
	import type { AppStoresInterface } from '$lib/types/appStores';
	import type { Hex } from '@rainlanguage/raindex';

	export let orderHash: AppStoresInterface['orderHash'];
	export let value = $orderHash;

	function handleChange(event: Event) {
		const input = event.target as HTMLInputElement;
		value = input.value as Hex;
		$orderHash = value;
	}
</script>

<div data-testid="order-hash-input" class="flex flex-col gap-x-2">
	<Label>Order hash</Label>
	<div class="w-full lg:w-32">
		<input
			type="text"
			bind:value
			on:input={handleChange}
			placeholder="0x..."
			class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400"
		/>
	</div>
</div>
