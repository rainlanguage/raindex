<script lang="ts">
	// import { createInfiniteQuery } from '@tanstack/svelte-query';
	// import TanstackAppTable from '../TanstackAppTable.svelte';
	// import { QKEY_ORDER_APY } from '../../queries/keys';
	// import { type RaindexOrder } from '@rainlanguage/raindex';
	// import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	// import ApyTimeFilters from '../charts/APYTimeFilters.svelte';
	// import { bigintStringToPercentage } from '$lib/utils/number';

	// export let order: RaindexOrder;

	// let startTimestamp: number | undefined;
	// let endTimestamp: number | undefined;

	// $: queryStartTime = startTimestamp ? BigInt(startTimestamp) : undefined;
	// $: queryEndTime = endTimestamp ? BigInt(endTimestamp) : undefined;

	// $: orderPerformance = createInfiniteQuery({
	// 	queryKey: [order.id, QKEY_ORDER_APY + order.id],
	// 	queryFn: async () => {
	// 		const result = await order.getPerformance(queryStartTime, queryEndTime);
	// 		if (result.error) throw new Error(result.error.readableMsg);
	// 		return [result.value];
	// 	},
	// 	initialPageParam: 0,
	// 	getNextPageParam: () => undefined
	// });
</script>

<!-- <TanstackAppTable
	query={orderPerformance}
	emptyMessage="APY Unavailable"
	rowHoverable={false}
	queryKey={order.id}
>
	<svelte:fragment slot="timeFilter">
		<ApyTimeFilters bind:startTimestamp bind:endTimestamp />
	</svelte:fragment>
	<svelte:fragment slot="head">
		<TableHeadCell padding="p-4">APY</TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell tdClass="break-all px-4 py-2" data-testid="apy-field">
			{item.denominatedPerformance
				? (item.denominatedPerformance.apyIsNeg ? '-' : '') +
					bigintStringToPercentage(item.denominatedPerformance.apy, 18, 5) +
					'% in ' +
					(item.denominatedPerformance.token.symbol ??
						item.denominatedPerformance.token.name ??
						item.denominatedPerformance.token.address)
				: 'Unavailable APY'}
		</TableBodyCell>
	</svelte:fragment>
</TanstackAppTable> -->

<div>TODO: Issue #1989</div>
