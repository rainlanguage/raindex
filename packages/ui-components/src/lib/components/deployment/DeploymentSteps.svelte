<script lang="ts">
	import { Alert, Button, Spinner, Toggle } from 'flowbite-svelte';
	import TokenIOInput from './TokenIOInput.svelte';
	import ComposedRainlangModal from './ComposedRainlangModal.svelte';
	import {
		type OrderBuilderSelectTokensCfg,
		type ExtendedTokenInfo,
		type OrderBuilderDepositCfg,
		type OrderBuilderFieldDefinitionCfg,
		type NameAndDescriptionCfg,
		type OrderIOCfg,
		RaindexOrderBuilder,
		RaindexClient,
		AccountBalance,
		Float
	} from '@rainlanguage/orderbook';
	import WalletConnect from '../wallet/WalletConnect.svelte';
	import { type Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import { handleShareChoices } from '../../services/handleShareChoices';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';
	import { onDestroy, onMount } from 'svelte';
	import FieldDefinitionInput from './FieldDefinitionInput.svelte';
	import DepositInput from './DepositInput.svelte';
	import SelectToken from './SelectToken.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';
	import { fade } from 'svelte/transition';
	import ShareChoicesButton from './ShareChoicesButton.svelte';
	import { useRegistry } from '$lib/providers/registry/useRegistry';
	import type { Account } from '$lib/types/account';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	import type { TokenBalance } from '$lib/types/tokenBalance';

	interface Deployment {
		key: string;
		name: string;
		description: string;
	}

	/** The deployment configuration containing key, name and description */
	export let deployment: Deployment;
	/** Strategy details containing name and description configuration */
	export let orderDetail: NameAndDescriptionCfg;
	/** Handlers for deployment modals */
	export let onDeploy: (raindexClient: RaindexClient, builder: RaindexOrderBuilder) => void;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
	export let account: Account;

	let allDepositFields: OrderBuilderDepositCfg[] = [];
	let allTokenOutputs: OrderIOCfg[] = [];
	let allTokenInputs: OrderIOCfg[] = [];
	let allFieldDefinitionsWithoutDefaults: OrderBuilderFieldDefinitionCfg[] = [];
	let allFieldDefinitionsWithDefaults: OrderBuilderFieldDefinitionCfg[] = [];
	let allTokensSelected: boolean = false;
	let showAdvancedOptions: boolean = false;
	let allTokenInfos: ExtendedTokenInfo[] = [];
	let selectTokens: OrderBuilderSelectTokensCfg[] | undefined = undefined;
	let checkingDeployment: boolean = false;
	let tokenBalances: Map<string, TokenBalance> = new Map();

	const builder = useRaindexOrderBuilder();
	const registry = useRegistry();
	const raindexClient = useRaindexClient();

	let deploymentStepsError = DeploymentStepsError.error;

	onMount(async () => {
		const selectTokensResult = builder.getSelectTokens();
		if (selectTokensResult.error) {
			throw new Error(selectTokensResult.error.msg);
		}
		selectTokens = selectTokensResult.value;
		await areAllTokensSelected();
	});

	$: if (selectTokens?.length === 0 || allTokensSelected) {
		updateFields();
	}

	let unsubscribeAccount = account.subscribe((account) => {
		if (!account) {
			const balances = tokenBalances;
			balances.clear();
			tokenBalances = balances;
			return;
		}
		if (selectTokens) {
			selectTokens.forEach(async (selectToken) => {
				await getTokenInfoAndFetchBalance(selectToken.key);
			});
		}
	});
	onDestroy(() => {
		unsubscribeAccount();
	});

	function getAllBuilderConfig() {
		try {
			let result = builder.getAllBuilderConfig();
			if (result.error) {
				throw new Error(result.error.msg);
			}
			allFieldDefinitionsWithoutDefaults = result.value.fieldDefinitionsWithoutDefaults;
			allFieldDefinitionsWithDefaults = result.value.fieldDefinitionsWithDefaults;
			allDepositFields = result.value.deposits;
			allTokenOutputs = result.value.orderOutputs;
			allTokenInputs = result.value.orderInputs;
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_BUILDER_CONFIG);
		}
	}

	function updateFields() {
		try {
			DeploymentStepsError.clear();
			getAllBuilderConfig();
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_BUILDER);
		}
	}

	async function _handleShareChoices() {
		await handleShareChoices(builder, registry.getCurrentRegistry());
	}

	async function fetchTokenBalance(tokenInfo: ExtendedTokenInfo) {
		if (!$account) return;

		const balances = tokenBalances;
		balances.set(tokenInfo.key, {
			value: { balance: Float.parse('0').value, formattedBalance: '0' } as AccountBalance,
			loading: true,
			error: ''
		});

		const { value: accountBalance, error } = await builder.getAccountBalance(
			tokenInfo.address,
			$account
		);
		if (error) {
			balances.set(tokenInfo.key, {
				value: { balance: Float.parse('0').value, formattedBalance: '0' } as AccountBalance,
				loading: false,
				error: error.readableMsg
			});
			tokenBalances = balances;
			return;
		}
		balances.set(tokenInfo.key, {
			value: accountBalance,
			loading: false,
			error: ''
		});
		tokenBalances = balances;
	}

	async function getTokenInfoAndFetchBalance(key: string) {
		const tokenInfoResult = await builder.getTokenInfo(key);
		if (tokenInfoResult.error) {
			throw new Error(tokenInfoResult.error.msg);
		}
		const tokenInfo = tokenInfoResult.value;
		if (!tokenInfo || !tokenInfo.address) {
			return;
		}
		await fetchTokenBalance(tokenInfo);
	}

	async function onSelectTokenSelect(key: string) {
		await areAllTokensSelected();

		await getTokenInfoAndFetchBalance(key);

		if (allTokensSelected) {
			let result = await builder.getAllTokenInfos();
			if (result.error) {
				throw new Error(result.error.msg);
			}
			let newAllTokenInfos = result.value;
			if (allTokenInfos !== newAllTokenInfos) {
				allTokenInfos = newAllTokenInfos;
				getAllBuilderConfig();
			}
		}
	}

	const areAllTokensSelected = async () => {
		try {
			const areAllTokensSelectedResult = builder.areAllTokensSelected();
			if (areAllTokensSelectedResult.error) {
				throw new Error(areAllTokensSelectedResult.error.msg);
			}
			allTokensSelected = areAllTokensSelectedResult.value;
			if (!allTokensSelected) return;

			const getAllTokenInfosResult = await builder.getAllTokenInfos();
			if (getAllTokenInfosResult.error) {
				throw new Error(getAllTokenInfosResult.error.msg);
			}
			allTokenInfos = getAllTokenInfosResult.value;

			// if we have deposits or vault ids set, show advanced options
			const hasDepositsResult = builder.hasAnyDeposit();
			if (hasDepositsResult.error) {
				throw new Error(hasDepositsResult.error.msg);
			}
			const hasVaultIdsResult = builder.hasAnyVaultId();
			if (hasVaultIdsResult.error) {
				throw new Error(hasVaultIdsResult.error.msg);
			}
			if (hasDepositsResult.value || hasVaultIdsResult.value) {
				showAdvancedOptions = true;
			}
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_SELECT_TOKENS);
		}
	};

	async function handleDeployButtonClick() {
		if (checkingDeployment) {
			return;
		}
		checkingDeployment = true;
		try {
			if (!$account) {
				DeploymentStepsError.catch(
					'No wallet connected',
					DeploymentStepsErrorCode.ADD_ORDER_FAILED
				);
				return;
			}
			DeploymentStepsError.clear();

			return onDeploy(raindexClient, builder);
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
		} finally {
			checkingDeployment = false;
		}
	}
</script>

<div>
	{#if builder}
		<div class="flex max-w-3xl flex-col gap-12" in:fade>
			{#if deployment}
				<div class="flex max-w-2xl flex-col gap-4 text-start">
					<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
						{orderDetail.name}
					</h1>
					<p class="text-xl text-gray-600 lg:text-2xl dark:text-gray-400">
						{deployment.description}
					</p>
				</div>
			{/if}

			{#if selectTokens && selectTokens.length > 0}
				<div class="flex w-full flex-col gap-4">
					<DeploymentSectionHeader
						title="Select Tokens"
						description="Select the tokens that you want to use in your order."
					/>
					{#each selectTokens as token}
						<SelectToken {token} {onSelectTokenSelect} {tokenBalances} />
					{/each}
				</div>
			{/if}

			{#if allTokensSelected || selectTokens?.length === 0}
				{#if allFieldDefinitionsWithoutDefaults.length > 0}
					{#each allFieldDefinitionsWithoutDefaults as fieldDefinition}
						<FieldDefinitionInput {fieldDefinition} />
					{/each}
				{/if}

				<Toggle bind:checked={showAdvancedOptions}>Show advanced options</Toggle>

				{#if showAdvancedOptions}
					{#each allFieldDefinitionsWithDefaults as fieldDefinition}
						<FieldDefinitionInput {fieldDefinition} />
					{/each}

					{#each allDepositFields as deposit}
						<DepositInput {deposit} />
					{/each}

					{#each allTokenOutputs as output}
						<TokenIOInput label="Output" vault={output} {tokenBalances} />
					{/each}

					{#each allTokenInputs as input}
						<TokenIOInput label="Input" vault={input} {tokenBalances} />
					{/each}
				{/if}

				{#if $deploymentStepsError}
					<Alert color="red">
						<p class="text-red-500">{$deploymentStepsError.code}</p>
						{#if $deploymentStepsError.details}
							<p class="text-red-500">{$deploymentStepsError.details}</p>
						{/if}
					</Alert>
				{/if}

				<div class="flex flex-wrap items-start justify-start gap-2">
					{#if $account}
						<Button
							data-testid="deploy-button"
							size="lg"
							disabled={checkingDeployment}
							on:click={handleDeployButtonClick}
							class="bg-gradient-to-br from-blue-600 to-violet-600"
						>
							{#if checkingDeployment}
								<Spinner size="4" color="white" />
								<span class="ml-2">Checking deployment...</span>
							{:else}
								Deploy Order
							{/if}
						</Button>
					{:else}
						<WalletConnect {appKitModal} connected={wagmiConnected} />
					{/if}
					<ComposedRainlangModal />
					<ShareChoicesButton handleShareChoices={_handleShareChoices} />
				</div>
			{/if}
		</div>
	{/if}
</div>
