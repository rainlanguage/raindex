<script lang="ts">
	import type { Address, RaindexVault } from '@rainlanguage/raindex';
	import { toHex } from 'viem';
	import Tooltip from './Tooltip.svelte';

	export let tokenVault: RaindexVault;
	export let chainId: number;
	export let raindexAddress: Address;
</script>

<div
	class="flex cursor-pointer items-center justify-between space-y-2 rounded-lg border border-gray-100 p-2"
	data-testid="vault-link"
>
	<div class="flex flex-col items-start gap-y-2">
		<Tooltip triggeredBy={`#token-info-${tokenVault.vaultId}`}>
			ID: <span class="font-mono">{toHex(tokenVault.vaultId)}</span>
		</Tooltip>
		<a
			href={`/vaults/${chainId}-${raindexAddress}-${tokenVault.id}`}
			id={`token-info-${tokenVault.vaultId}`}
		>
			{tokenVault.token.name} ({tokenVault.token.symbol})
		</a>
		<span class="text-sm text-gray-500 dark:text-gray-400">
			Balance: {tokenVault.formattedBalance}
		</span>
	</div>
	<div>
		<slot name="buttons" />
	</div>
</div>
