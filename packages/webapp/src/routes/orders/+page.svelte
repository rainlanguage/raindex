<script lang="ts">
	import { page } from '$app/stores';
	import {
		OrdersListTable,
		PageHeader,
		useTransactions,
		useAccount,
		useToasts,
		useRaindexClient
	} from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import {
		orderHash,
		showInactiveOrders,
		activeTokens,
		selectedChainIds,
		activeOrderbookAddresses,
		ownerFilter
	} from '$lib/stores/settings';
	import { handleTransactionConfirmationModal, handleTakeOrderModal } from '$lib/services/modal';
	import { handleTakeOrder } from '$lib/services/handleTakeOrder';
	import type { RaindexOrder } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';

	const { hideZeroBalanceVaults, hideInactiveOrdersVaults }: AppStoresInterface = $page.data.stores;

	const { manager } = useTransactions();
	const { account } = useAccount();
	const { errToast, addToast } = useToasts();
	const raindexClient = useRaindexClient();

	const onTakeOrderCallback = (item: RaindexOrder) => {
		handleTakeOrder({
			raindexClient,
			order: item,
			handleTakeOrderModal,
			handleTransactionConfirmationModal,
			errToast,
			addToast,
			manager,
			account: $account as Hex
		});
	};
</script>

<PageHeader title={'Orders'} pathname={$page.url.pathname} />

<OrdersListTable
	{selectedChainIds}
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
	{hideInactiveOrdersVaults}
	{activeTokens}
	{activeOrderbookAddresses}
	{ownerFilter}
	handleTakeOrderModal={onTakeOrderCallback}
/>
