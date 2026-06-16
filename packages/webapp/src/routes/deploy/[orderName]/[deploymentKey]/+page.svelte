<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import {
		DeploymentSteps,
		GuiProvider,
		useAccount,
		useToasts,
		useTransactions
	} from '@rainlanguage/ui-components';
	import { connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDisclaimerModal } from '$lib/services/modal';
	import { RaindexClient, DotrainOrderGui } from '@rainlanguage/orderbook';
	import { onMount } from 'svelte';
	import { handleAddOrder } from '$lib/services/handleAddOrder';
	import { handleTransactionConfirmationModal } from '$lib/services/modal';
	import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

	const { orderName, deployment, orderDetail, rainlang } = $page.data;
	const stateFromUrl = $page.url.searchParams?.get('state') || '';

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	let gui: DotrainOrderGui | null = null;
	let getGuiError: string | null = null;

	onMount(async () => {
		if (!deployment || !rainlang || !orderName) {
			setTimeout(() => {
				goto('/deploy');
			}, 5000);
			return;
		}

		const serializedState = stateFromUrl || undefined;
		const guiResult = await rainlang.getGui(
			orderName,
			deployment.key,
			serializedState,
			pushGuiStateToUrlHistory
		);
		if (guiResult.error) {
			getGuiError = guiResult.error.readableMsg ?? guiResult.error.msg;
			return;
		}

		gui = guiResult.value;
	});

	const onDeploy = (raindexClient: RaindexClient, gui: DotrainOrderGui) => {
		handleDisclaimerModal({
			open: true,
			onAccept: () => {
				handleAddOrder({
					raindexClient,
					handleTransactionConfirmationModal,
					errToast,
					manager,
					gui,
					account: $account
				});
			}
		});
	};
</script>

{#if !deployment || !rainlang}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if gui}
	<div data-testid="gui-provider">
		<GuiProvider {gui}>
			<DeploymentSteps
				{orderDetail}
				{deployment}
				wagmiConnected={connected}
				{appKitModal}
				{onDeploy}
				{account}
			/>
		</GuiProvider>
	</div>
{:else if getGuiError}
	<div>
		{getGuiError}
	</div>
{/if}
