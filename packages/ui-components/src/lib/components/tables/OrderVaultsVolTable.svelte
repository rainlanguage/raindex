<script lang="ts">
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { QKEY_VAULTS_VOL_LIST } from '../../queries/keys';
	import { type RaindexVaultVolume, type RaindexOrder } from '@rainlanguage/raindex';
	import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import TableTimeFilters from '../charts/TableTimeFilters.svelte';

	export let order: RaindexOrder;

	let startTimestamp: number | undefined;
	let endTimestamp: number | undefined;

	$: queryStartTime = startTimestamp ? BigInt(startTimestamp) : undefined;
	$: queryEndTime = endTimestamp ? BigInt(endTimestamp) : undefined;

	$: vaultsVol = createInfiniteQuery<RaindexVaultVolume[]>({
		queryKey: [order.id, QKEY_VAULTS_VOL_LIST + order.id],
		queryFn: async () => {
			const result = await order.getVaultsVolume(queryStartTime, queryEndTime);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		},
		initialPageParam: 0,
		getNextPageParam: () => undefined
	});

	const AppTable = TanstackAppTable<RaindexVaultVolume>;
</script>

<AppTable query={vaultsVol} emptyMessage="No trades found" rowHoverable={false} queryKey={order.id}>
	<svelte:fragment slot="timeFilter">
		<TableTimeFilters bind:startTimestamp bind:endTimestamp />
	</svelte:fragment>
	<svelte:fragment slot="head">
		<TableHeadCell padding="p-4">Vault</TableHeadCell>
		<TableHeadCell padding="p-0">Token</TableHeadCell>
		<TableHeadCell padding="p-0">In Volume</TableHeadCell>
		<TableHeadCell padding="p-0">Out Volume</TableHeadCell>
		<TableHeadCell padding="p-0">Net Volume</TableHeadCell>
		<TableHeadCell padding="p-0">Total Volume</TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell tdClass="px-4 py-2">
			<Hash type={HashType.Identifier} shorten value={item.id.toString()} />
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-32">
			<div class="flex gap-x-3">
				<Hash type={HashType.Address} shorten value={item.token.address} />
				{item.token.symbol}
			</div>
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-32" data-testid="total-in">
			{item.details.formattedTotalIn}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2" data-testid="total-out">
			{item.details.formattedTotalOut}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2" data-testid="net-vol">
			{item.details.formattedNetVol}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2" data-testid="total-vol">
			{item.details.formattedTotalVol}
		</TableBodyCell>
	</svelte:fragment>
</AppTable>
