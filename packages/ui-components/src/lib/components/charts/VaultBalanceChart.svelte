<!-- TODO: Issue #1989 -->
<script lang="ts">
	// import { timestampSecondsToUTCTimestamp } from '../../services/time';
	// 	import type { RaindexVault, RaindexVaultBalanceChange } from '@rainlanguage/raindex';
	// 	import { createQuery } from '@tanstack/svelte-query';
	// 	import TanstackLightweightChartLine from '../charts/TanstackLightweightChartLine.svelte';
	// 	import { QKEY_VAULT_CHANGES } from '../../queries/keys';

	// export let vault: RaindexVault;
	// export let lightweightChartsTheme;

	// $: query = createQuery({
	// 	queryKey: [vault.id, QKEY_VAULT_CHANGES + vault.id, QKEY_VAULT_CHANGES],
	// 	queryFn: async () => {
	// 		const result = await vault.getBalanceChanges(1);
	// 		if (result.error) throw new Error(result.error.msg);
	// 		return result.value;
	// 	}
	// });

	// const Chart = TanstackLightweightChartLine<RaindexVaultBalanceChange>;
</script>

<!--
{#if vault && $query.data}
	<Chart
		title="Balance history"
		priceSymbol={vault.token.symbol}
		{query}
		timeTransform={(d) => timestampSecondsToUTCTimestamp(BigInt(d.timestamp))}
		valueTransform={(d) => parseFloat(d.formattedNewBalance)}
		emptyMessage="No deposits or withdrawals found"
		{lightweightChartsTheme}
	/>
{/if} -->

{#if false}
	<div>TODO: Issue #1989</div>
{/if}
