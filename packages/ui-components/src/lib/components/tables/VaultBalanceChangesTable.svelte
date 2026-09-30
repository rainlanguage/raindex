<script lang="ts">
	import { Heading, TableHeadCell, TableBodyCell } from 'flowbite-svelte';
	import type { RaindexVaultBalanceChange } from '@rainlanguage/orderbook';
	import { formatTimestampSecondsAsLocal } from '../../services/time';
	import Hash, { HashType } from '../Hash.svelte';
	import { Table, TableBody, TableBodyRow, TableHead } from 'flowbite-svelte';

	export let data: RaindexVaultBalanceChange[] | undefined = undefined;
</script>

<Heading tag="h5" class="mb-4 mt-6 font-normal">Vault balance changes</Heading>

{#if data?.length === 0}
	<div data-testid="emptyMessage" class="text-center text-gray-900 dark:text-white">
		No deposits or withdrawals found
	</div>
{:else if data}
	<Table
		divClass="cursor-pointer rounded-lg overflow-auto dark:border-none border"
		hoverable={false}
	>
		<TableHead data-testid="head">
			<TableHeadCell padding="p-4">Date</TableHeadCell>
			<TableHeadCell padding="p-0">Sender</TableHeadCell>
			<TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
			<TableHeadCell padding="p-0">Balance Change</TableHeadCell>
			<TableHeadCell padding="p-0">Balance</TableHeadCell>
			<TableHeadCell padding="p--">Type</TableHeadCell>
		</TableHead>
		<TableBody>
			{#each data as item}
				<TableBodyRow class="whitespace-nowrap" data-testid="bodyRow">
					<TableBodyCell tdClass="px-4 py-2" data-testid="vaultBalanceChangesTableDate">
						{formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
					</TableBodyCell>
					<TableBodyCell
						tdClass="break-all py-2 min-w-48"
						data-testid="vaultBalanceChangesTableFrom"
					>
						<Hash type={HashType.Wallet} value={item.transaction.from} />
					</TableBodyCell>
					<TableBodyCell tdClass="break-all py-2 min-w-48" data-testid="vaultBalanceChangesTableTx">
						<Hash type={HashType.Transaction} value={item.transaction.id} />
					</TableBodyCell>
					<TableBodyCell
						tdClass="break-word p-0 text-left"
						data-testid="vaultBalanceChangesTableBalanceChange"
					>
						{`${item.formattedAmount} ${item.token.symbol}`}
					</TableBodyCell>
					<TableBodyCell
						tdClass="break-word p-0 text-left"
						data-testid="vaultBalanceChangesTableBalance"
					>
						{`${item.formattedNewBalance} ${item.token.symbol}`}
					</TableBodyCell>
					<TableBodyCell
						tdClass="break-word p-0 text-left"
						data-testid="vaultBalanceChangesTableType"
					>
						{item.type}
					</TableBodyCell>
				</TableBodyRow>
			{/each}
		</TableBody>
	</Table>
{/if}
