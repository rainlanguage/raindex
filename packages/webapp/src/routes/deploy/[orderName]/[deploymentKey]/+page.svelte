<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import {
		DeploymentSteps,
		RaindexOrderBuilderProvider,
		useAccount,
		useToasts,
		useTransactions
	} from '@rainlanguage/ui-components';
	import { connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDisclaimerModal } from '$lib/services/modal';
	import { RaindexClient, RaindexOrderBuilder } from '@rainlanguage/orderbook';
	import { onMount } from 'svelte';
	import { handleAddOrder } from '$lib/services/handleAddOrder';
	import { handleTransactionConfirmationModal } from '$lib/services/modal';
	import { pushBuilderStateToUrlHistory } from '$lib/services/handleUpdateBuilderState';

	const { orderName, deployment, orderDetail, registry } = $page.data;
	const stateFromUrl = $page.url.searchParams?.get('state') || '';

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	let builder: RaindexOrderBuilder | null = null;
	let getBuilderError: string | null = null;

	onMount(async () => {
		if (!deployment || !registry || !orderName) {
			setTimeout(() => {
				goto('/deploy');
			}, 5000);
			return;
		}

		const serializedState = stateFromUrl || undefined;
		const builderResult = await registry.getOrderBuilder(
			orderName,
			deployment.key,
			serializedState,
			pushBuilderStateToUrlHistory
		);
		if (builderResult.error) {
			getBuilderError = builderResult.error.readableMsg ?? builderResult.error.msg;
			return;
		}

		builder = builderResult.value;
	});

	const onDeploy = (raindexClient: RaindexClient, builder: RaindexOrderBuilder) => {
		handleDisclaimerModal({
			open: true,
			onAccept: () => {
				handleAddOrder({
					raindexClient,
					handleTransactionConfirmationModal,
					errToast,
					manager,
					builder,
					account: $account
				});
			}
		});
	};
</script>

{#if !deployment || !registry}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if builder}
	<div data-testid="builder-provider">
		<RaindexOrderBuilderProvider {builder}>
			<DeploymentSteps
				{orderDetail}
				{deployment}
				wagmiConnected={connected}
				{appKitModal}
				{onDeploy}
				{account}
			/>
		</RaindexOrderBuilderProvider>
	</div>
{:else if getBuilderError}
	<div>
		{getBuilderError}
	</div>
{/if}
