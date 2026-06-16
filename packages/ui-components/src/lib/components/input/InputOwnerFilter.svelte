<script lang="ts">
	import { Button, Label } from 'flowbite-svelte';
	import type { Address } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '$lib/types/appStores';
	import { useAccount } from '$lib/providers/wallet/useAccount';

	export let ownerFilter: AppStoresInterface['ownerFilter'];

	const { account } = useAccount();

	let value: string = '';
	$: value = $ownerFilter;

	function handleChange(event: Event) {
		const input = event.target as HTMLInputElement;
		value = input.value;
		ownerFilter.set(value as unknown as Address);
	}

	function handleMyItems() {
		if (!$account) return;
		if ($ownerFilter.toLowerCase() === $account.toLowerCase()) {
			ownerFilter.set('' as unknown as Address);
			value = '';
		} else {
			ownerFilter.set($account);
			value = $account;
		}
	}
</script>

<div data-testid="owner-filter-input" class="flex flex-col gap-x-2">
	<Label>Owner</Label>
	<div class="flex items-center gap-2">
		<div class="w-full lg:w-32">
			<input
				type="text"
				bind:value
				on:input={handleChange}
				placeholder="0x..."
				class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400"
			/>
		</div>
		<Button
			size="xs"
			color={$account && $ownerFilter.toLowerCase() === $account.toLowerCase()
				? 'primary'
				: 'alternative'}
			disabled={!$account}
			on:click={handleMyItems}
			data-testid="my-items-button"
		>
			My Items
		</Button>
	</div>
</div>
