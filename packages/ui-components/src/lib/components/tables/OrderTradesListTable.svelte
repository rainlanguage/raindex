<script lang="ts">
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { QKEY_ORDER_TRADES_LIST } from '../../queries/keys';
	import { DEFAULT_PAGE_SIZE } from '../../queries/constants';
	import { Badge, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import { formatTimestampSecondsAsLocal } from '../../services/time';
	import Hash, { HashType } from '../Hash.svelte';
	import { BugOutline } from 'flowbite-svelte-icons';
	import type { RaindexOrder, RaindexTrade } from '@rainlanguage/orderbook';
	import TableTimeFilters from '../charts/TableTimeFilters.svelte';
	import Tooltip from '../Tooltip.svelte';

	type BadgeColor = 'blue' | 'pink' | 'dark';

	function getTypeBadgeColor(type: string): BadgeColor {
		const lowerType = type.toLowerCase();
		if (lowerType.includes('clear')) return 'pink';
		if (lowerType.includes('trade') || lowerType.includes('takeorder')) return 'blue';
		return 'dark';
	}

	function getTradeTypeDisplayName(type: string): string {
		if (type === 'Clear') return 'Clear';
		return 'Take order';
	}

	export let order: RaindexOrder;
	export let rpcs: string[] | undefined = undefined;
	export let handleDebugTradeModal: ((hash: string, rpcs: string[]) => void) | undefined =
		undefined;

	let startTimestamp: number | undefined;
	let endTimestamp: number | undefined;
	let tradesCount: number | undefined;

	$: orderTradesQuery = createInfiniteQuery({
		queryKey: [order.id, QKEY_ORDER_TRADES_LIST + order.id],
		queryFn: async ({ pageParam }: { pageParam: number }) => {
			tradesCount = undefined;

			const [countResult, tradesResult] = await Promise.all([
				order.getTradeCount(
					startTimestamp ? BigInt(startTimestamp) : undefined,
					endTimestamp ? BigInt(endTimestamp) : undefined
				),
				order.getTradesList(
					startTimestamp ? BigInt(startTimestamp) : undefined,
					endTimestamp ? BigInt(endTimestamp) : undefined,
					pageParam + 1
				)
			]);
			if (countResult.error) throw new Error(countResult.error.readableMsg);
			if (tradesResult.error) throw new Error(tradesResult.error.readableMsg);

			const count = countResult.value;
			const trades = tradesResult.value;

			if (typeof count === 'number') {
				tradesCount = count;
			}

			return trades;
		},
		initialPageParam: 0,
		getNextPageParam: (
			lastPage: RaindexTrade[],
			_allPages: RaindexTrade[][],
			lastPageParam: number
		) => {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		}
	});

	const AppTable = TanstackAppTable<RaindexTrade>;
</script>

<AppTable
	query={orderTradesQuery}
	emptyMessage="No trades found"
	rowHoverable={false}
	queryKey={order.id}
>
	<svelte:fragment slot="info">
		{#if tradesCount !== undefined}
			<div class="px-2">{tradesCount} Trades</div>
		{/if}
	</svelte:fragment>
	<svelte:fragment slot="timeFilter">
		<TableTimeFilters bind:startTimestamp bind:endTimestamp />
	</svelte:fragment>
	<svelte:fragment slot="head">
		<TableHeadCell padding="p-4" class="w-[14%]">Info</TableHeadCell>
		<TableHeadCell padding="p-4" class="w-[16%]">Transaction</TableHeadCell>
		<TableHeadCell padding="p-4" class="w-[16%]">Counterparty</TableHeadCell>
		<TableHeadCell padding="p-2" class="w-[15%]">Input</TableHeadCell>
		<TableHeadCell padding="p-2" class="w-[15%]">Output</TableHeadCell>
		<TableHeadCell padding="p-2" class="w-[20%]">IO Ratio</TableHeadCell>
		<TableHeadCell padding="p-0" class="w-[4%]"><span class="sr-only">Actions</span></TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		{@const inputAmt = Number(item.inputVaultBalanceChange.formattedAmount)}
		{@const outputAmt = Number(item.outputVaultBalanceChange.formattedAmount)}
		{@const ioRatio = Math.abs(inputAmt / outputAmt)}
		{@const oiRatio = Math.abs(outputAmt / inputAmt)}
		{@const validRatio = Number.isFinite(ioRatio) && Number.isFinite(oiRatio)}
		<TableBodyCell tdClass="px-4 py-2">
			<div class="flex flex-col gap-1 overflow-hidden">
				<div class="flex">
					<Badge color={getTypeBadgeColor(item.tradeEventType)}>
						{getTradeTypeDisplayName(item.tradeEventType)}
					</Badge>
				</div>
				<span class="text-xs text-gray-500 dark:text-gray-400">
					{formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
				</span>
			</div>
		</TableBodyCell>
		<TableBodyCell tdClass="px-4 py-2">
			<div class="flex flex-col gap-1 text-sm">
				<div class="flex items-center gap-1">
					<span class="text-gray-500 dark:text-gray-400">Sender:</span>
					<Hash type={HashType.Wallet} value={item.transaction.from} />
				</div>
				<div class="flex items-center gap-1">
					<span class="text-gray-500 dark:text-gray-400">Tx:</span>
					<Hash type={HashType.Transaction} value={item.transaction.id} />
				</div>
			</div>
		</TableBodyCell>
		<TableBodyCell tdClass="px-4 py-2">
			{#if item.counterparty}
				<div class="flex flex-col gap-1 text-sm">
					<div class="flex items-center gap-1">
						<span class="text-gray-500 dark:text-gray-400">
							{item.tradeEventType === 'Clear' ? 'Counterparty:' : 'Taker:'}
						</span>
						<Hash type={HashType.Wallet} value={item.counterparty.owner} />
					</div>
					{#if item.counterparty.orderHash}
						<div class="flex items-center gap-1">
							<span class="text-gray-500 dark:text-gray-400">Order:</span>
							<a
								href={`/orders/${order.chainId}-${order.orderbook}-${item.counterparty.orderHash}`}
								class="text-blue-600 hover:underline dark:text-blue-400"
							>
								<Hash
									type={HashType.Identifier}
									value={item.counterparty.orderHash}
									copyOnClick={false}
								/>
							</a>
						</div>
					{/if}
				</div>
			{:else}
				<span class="text-gray-400">-</span>
			{/if}
		</TableBodyCell>
		<TableBodyCell tdClass="p-2" data-testid="input">
			<div class="flex flex-col overflow-hidden">
				<span class="truncate font-medium">{item.inputVaultBalanceChange.token.symbol}</span>
				<span id={`input-${item.id}`} class="truncate text-sm text-gray-500 dark:text-gray-400"
					>{item.inputVaultBalanceChange.formattedAmount}</span
				>
				<Tooltip triggeredBy={`#input-${item.id}`}>
					{item.inputVaultBalanceChange.formattedAmount}
					{item.inputVaultBalanceChange.token.symbol}
				</Tooltip>
			</div>
		</TableBodyCell>
		<TableBodyCell tdClass="p-2" data-testid="output">
			<div class="flex flex-col overflow-hidden">
				<span class="truncate font-medium">{item.outputVaultBalanceChange.token.symbol}</span>
				<span id={`output-${item.id}`} class="truncate text-sm text-gray-500 dark:text-gray-400"
					>{item.outputVaultBalanceChange.formattedAmount}</span
				>
				<Tooltip triggeredBy={`#output-${item.id}`}>
					{item.outputVaultBalanceChange.formattedAmount}
					{item.outputVaultBalanceChange.token.symbol}
				</Tooltip>
			</div>
		</TableBodyCell>
		<TableBodyCell tdClass="p-2" data-testid="io-ratio">
			<div id={`io-ratio-${item.id}`} class="truncate">
				{#if validRatio}
					{ioRatio}
					<span class="text-gray-400">({oiRatio})</span>
				{:else}
					-
				{/if}
			</div>
			<Tooltip triggeredBy={`#io-ratio-${item.id}`}>
				{#if validRatio}
					{ioRatio} ({oiRatio})
				{:else}
					-
				{/if}
			</Tooltip>
		</TableBodyCell>
		<TableBodyCell tdClass="py-2">
			{#if rpcs && handleDebugTradeModal}
				<button
					data-testid="debug-trade-button"
					class="text-gray-500 hover:text-gray-700"
					on:click={() => {
						if (rpcs) handleDebugTradeModal(item.transaction.id, rpcs);
					}}
				>
					<BugOutline size="xs" />
				</button>
			{/if}
		</TableBodyCell>
	</svelte:fragment>
</AppTable>
