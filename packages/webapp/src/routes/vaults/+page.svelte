<script lang="ts">
	import {
		PageHeader,
		useAccount,
		useToasts,
		useTransactions,
		VaultsListTable
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import {
		hideZeroBalanceVaults,
		hideInactiveOrdersVaults,
		orderHash,
		activeTokens,
		activeOrderbookAddresses,
		selectedChainIds,
		ownerFilter
	} from '$lib/stores/settings';
	import { handleTransactionConfirmationModal, handleWithdrawAllModal } from '$lib/services/modal';
	import type { RaindexClient, RaindexVaultsList } from '@rainlanguage/orderbook';
	import { handleVaultsWithdrawAll } from '$lib/services/handleVaultsWithdrawAll';
	import type { Hex } from 'viem';

	const { showInactiveOrders } = $page.data.stores;

	const { account } = useAccount();
	const { errToast } = useToasts();
	const { manager } = useTransactions();

	async function onWithdrawAll(raindexClient: RaindexClient, vaultsList: RaindexVaultsList) {
		if (!$account) {
			errToast('Please connect your wallet to withdraw');
			return;
		}
		await handleVaultsWithdrawAll({
			raindexClient,
			vaultsList,
			handleWithdrawAllModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			account: $account as Hex
		});
	}
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
	{orderHash}
	{showInactiveOrders}
	{hideZeroBalanceVaults}
	{hideInactiveOrdersVaults}
	{activeTokens}
	{selectedChainIds}
	{activeOrderbookAddresses}
	{ownerFilter}
	{onWithdrawAll}
/>
