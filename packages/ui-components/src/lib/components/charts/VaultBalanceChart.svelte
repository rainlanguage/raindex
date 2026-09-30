<script lang="ts">
	import { timestampSecondsToUTCTimestamp } from '../../services/time';
	import type { RaindexVault, RaindexVaultBalanceChange } from '@rainlanguage/orderbook';
	import type { CreateQueryResult } from '@tanstack/svelte-query';
	import TanstackLightweightChartLine from '../charts/TanstackLightweightChartLine.svelte';

	export let vault: RaindexVault;
	export let query: CreateQueryResult<RaindexVaultBalanceChange[]>;
	export let lightweightChartsTheme;

	const Chart = TanstackLightweightChartLine<RaindexVaultBalanceChange>;
</script>

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
{/if}
