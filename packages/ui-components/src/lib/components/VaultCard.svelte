<script lang="ts">
	import { goto } from '$app/navigation';
	import type { RaindexVault } from '@rainlanguage/raindex';

	export let vault: RaindexVault;

	const handleClick = (event: MouseEvent) => {
		// Stop event propagation and prevent default to fully prevent parent table row click
		event.stopPropagation();
		event.preventDefault();
		// Navigate to vault details page
		goto(`/vaults/${vault.chainId}-${vault.raindex}-${vault.id}`);
	};
</script>

<button
	type="button"
	class="flex flex-col rounded-xl border border-gray-200 bg-gray-50 px-4 py-3 text-left shadow-sm transition-colors hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600"
	on:click={handleClick}
	data-testid="vault-card"
>
	<span class="font-semibold text-gray-800 dark:text-gray-200">{vault.token.symbol}</span>
	<span class="truncate text-xs text-gray-500 dark:text-gray-400">{vault.formattedBalance}</span>
</button>
