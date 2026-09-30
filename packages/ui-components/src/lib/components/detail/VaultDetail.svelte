<script lang="ts">
	import Hash, { HashType } from '../Hash.svelte';
	import VaultBalanceChangesTable from '../tables/VaultBalanceChangesTable.svelte';
	import VaultBalanceChart from '../charts/VaultBalanceChart.svelte';
	import TanstackPageContentDetail from './TanstackPageContentDetail.svelte';
	import CardProperty from '../CardProperty.svelte';
	import { QKEY_VAULT, QKEY_VAULT_CHANGES } from '../../queries/keys';
	import {
		RaindexClient,
		type Address,
		type Hex,
		type RaindexVault,
		type RaindexVaultBalanceChange
	} from '@rainlanguage/orderbook';
	import type { ChartTheme } from '../../utils/lightweightChartsThemes';
	import { toHex } from 'viem';
	import { createQuery } from '@tanstack/svelte-query';
	import { onDestroy } from 'svelte';
	import type { Readable } from 'svelte/store';
	import { useQueryClient } from '@tanstack/svelte-query';
	import OrderOrVaultHash from '../OrderOrVaultHash.svelte';
	import Refresh from '../icon/Refresh.svelte';
	import { invalidateTanstackQueries } from '$lib/queries/queryClient';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { Button } from 'flowbite-svelte';
	import { ArrowDownToBracketOutline, ArrowUpFromBracketOutline } from 'flowbite-svelte-icons';
	import { useToasts } from '$lib/providers/toasts/useToasts';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	export let id: Hex;
	export let orderbookAddress: Address;
	export let chainId: number;
	export let lightweightChartsTheme: Readable<ChartTheme> | undefined = undefined;

	/**
	 * Required callback function when deposit action is triggered for a vault
	 * @param vault The vault to deposit into
	 */
	export let onDeposit: (raindexClient: RaindexClient, vault: RaindexVault) => void;

	/**
	 * Required callback function when withdraw action is triggered for a vault
	 * @param vault The vault to withdraw from
	 */
	export let onWithdraw: (raindexClient: RaindexClient, vault: RaindexVault) => void;

	const queryClient = useQueryClient();
	const { matchesAccount } = useAccount();
	const { errToast } = useToasts();
	const raindexClient = useRaindexClient();

	$: vaultDetailQuery = createQuery<RaindexVault>({
		queryKey: [id, QKEY_VAULT + id],
		queryFn: async () => {
			const result = await raindexClient.getVault(chainId, orderbookAddress, id);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		}
	});

	$: balanceChangesQuery = createQuery<RaindexVaultBalanceChange[]>({
		queryKey: [
			$vaultDetailQuery.data?.id ?? '',
			QKEY_VAULT_CHANGES + ($vaultDetailQuery.data?.id ?? ''),
			QKEY_VAULT_CHANGES
		],
		enabled: !!$vaultDetailQuery.data,
		queryFn: async () => {
			// eslint-disable-next-line no-console
			console.log('[VaultDetail] fetching balance changes, vault:', $vaultDetailQuery.data?.id);
			const result = await $vaultDetailQuery.data!.getBalanceChanges(1);
			// eslint-disable-next-line no-console
			console.log(
				'[VaultDetail] result:',
				result,
				'value length:',
				result.value?.length,
				'error:',
				result.error
			);
			if (result.error) throw new Error(result.error.msg);
			return result.value;
		}
	});

	// eslint-disable-next-line no-console
	$: console.log(
		'[VaultDetail] vaultData:',
		!!$vaultDetailQuery.data,
		'bcData:',
		$balanceChangesQuery?.data?.length,
		'bcLoading:',
		$balanceChangesQuery?.isLoading,
		'bcError:',
		$balanceChangesQuery?.error
	);

	const interval = setInterval(async () => {
		invalidateTanstackQueries(queryClient, [id, QKEY_VAULT + id]);
	}, 5000);

	onDestroy(() => {
		clearInterval(interval);
	});

	const handleRefresh = async () => {
		try {
			await invalidateTanstackQueries(queryClient, [id, QKEY_VAULT + id]);
		} catch {
			errToast('Failed to refresh');
		}
	};
</script>

<TanstackPageContentDetail query={vaultDetailQuery} emptyMessage="Vault not found">
	<svelte:fragment slot="top" let:data>
		<div
			data-testid="vaultDetailTokenName"
			class="flex gap-x-4 text-3xl font-medium dark:text-white"
		>
			{data.token.name}
		</div>
		<div class="flex items-center gap-2">
			{#if matchesAccount(data.owner)}
				<Button
					color="light"
					size="xs"
					data-testid="deposit-button"
					aria-label="Deposit to vault"
					on:click={() => onDeposit(raindexClient, data)}
				>
					<ArrowDownToBracketOutline size="xs" />
				</Button>
				<Button
					color="light"
					size="xs"
					data-testid="withdraw-button"
					aria-label="Withdraw from vault"
					on:click={() => onWithdraw(raindexClient, data)}
				>
					<ArrowUpFromBracketOutline size="xs" />
				</Button>
			{/if}

			<Refresh
				testId="top-refresh"
				on:click={handleRefresh}
				spin={$vaultDetailQuery.isLoading || $vaultDetailQuery.isFetching}
			/>
		</div>
	</svelte:fragment>
	<svelte:fragment slot="card" let:data>
		<CardProperty data-testid="vaultDetailVaultId">
			<svelte:fragment slot="key">Vault ID</svelte:fragment>
			<svelte:fragment slot="value">{toHex(data.vaultId)}</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailOrderbookAddress">
			<svelte:fragment slot="key">Orderbook</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash type={HashType.Identifier} value={data.orderbook} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailOwnerAddress">
			<svelte:fragment slot="key">Owner address</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash type={HashType.Wallet} value={data.owner} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailTokenAddress">
			<svelte:fragment slot="key">Token address</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash value={data.token.id} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailBalance">
			<svelte:fragment slot="key">Balance</svelte:fragment>
			<svelte:fragment slot="value"
				>{`${data.formattedBalance} ${data.token.symbol}`}</svelte:fragment
			>
		</CardProperty>

		<CardProperty>
			<svelte:fragment slot="key">Orders as input</svelte:fragment>
			<svelte:fragment slot="value">
				<p data-testid="vaultDetailOrdersAsInput" class="flex flex-wrap justify-start">
					{#if data.ordersAsInput && data.ordersAsInput.length > 0}
						{#each data.ordersAsInput as order}
							<OrderOrVaultHash type="orders" orderOrVault={order} {chainId} {orderbookAddress} />
						{/each}
					{:else}
						None
					{/if}
				</p>
			</svelte:fragment>
		</CardProperty>

		<CardProperty>
			<svelte:fragment slot="key">Orders as output</svelte:fragment>
			<svelte:fragment slot="value">
				<p data-testid="vaultDetailOrdersAsOutput" class="flex flex-wrap justify-start">
					{#if data.ordersAsOutput && data.ordersAsOutput.length > 0}
						{#each data.ordersAsOutput as order}
							<OrderOrVaultHash type="orders" orderOrVault={order} {chainId} {orderbookAddress} />
						{/each}
					{:else}
						None
					{/if}
				</p>
			</svelte:fragment>
		</CardProperty>
	</svelte:fragment>

	<svelte:fragment slot="chart" let:data>
		<VaultBalanceChart vault={data} query={balanceChangesQuery} {lightweightChartsTheme} />
	</svelte:fragment>

	<svelte:fragment slot="below">
		<VaultBalanceChangesTable data={$balanceChangesQuery.data} />
	</svelte:fragment>
</TanstackPageContentDetail>
