<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import type { OrderBuilderSelectTokensCfg, ExtendedTokenInfo } from '@rainlanguage/orderbook';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import { Spinner } from 'flowbite-svelte';
	import { onMount } from 'svelte';
	import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import TokenSelectionModal from './TokenSelectionModal.svelte';
	import TokenBalanceComponent from './TokenBalance.svelte';
	import type { TokenBalance } from '$lib/types/tokenBalance';

	export let token: OrderBuilderSelectTokensCfg;
	export let onSelectTokenSelect: (key: string) => void;
	export let tokenBalances: Map<string, TokenBalance> = new Map();

	let inputValue: string | null = null;
	let tokenInfo: ExtendedTokenInfo | null = null;
	let error = '';
	let checking = false;
	let selectionMode: 'dropdown' | 'custom' = 'dropdown';
	let selectedToken: ExtendedTokenInfo | null = null;

	const builder = useRaindexOrderBuilder();

	onMount(async () => {
		try {
			let result = await builder.getTokenInfo(token.key);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			tokenInfo = result.value;
			if (result.value.address) {
				inputValue = result.value.address;
				onSelectTokenSelect(token.key);
			}
		} catch {
			// do nothing
		}
	});

	$: if (tokenInfo?.address && inputValue === null) {
		inputValue = tokenInfo.address;
	}

	function setMode(mode: 'dropdown' | 'custom') {
		selectionMode = mode;
		error = '';

		if (mode === 'custom') {
			selectedToken = null;
			tokenInfo = null;
			inputValue = '';
			error = '';
			clearTokenSelection();
		}
	}

	function handleTokenSelect(token: ExtendedTokenInfo) {
		selectedToken = token;
		inputValue = token.address;
		saveTokenSelection(token.address);
	}

	async function saveTokenSelection(address: string) {
		checking = true;
		error = '';
		try {
			await builder.setSelectToken(token.key, address);
			await getInfoForSelectedToken();
		} catch (e) {
			const errorMessage = (e as Error).message || 'Invalid token address.';
			error = errorMessage;
		} finally {
			checking = false;
			onSelectTokenSelect(token.key);
		}
	}

	function clearTokenSelection() {
		builder.unsetSelectToken(token.key);
		onSelectTokenSelect(token.key);
	}

	async function getInfoForSelectedToken() {
		error = '';
		try {
			let result = await builder.getTokenInfo(token.key);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			tokenInfo = result.value;
			error = '';
		} catch {
			return (error = 'No token exists at this address.');
		}
	}

	async function handleInput(event: Event) {
		const currentTarget = event.currentTarget;
		if (currentTarget instanceof HTMLInputElement) {
			inputValue = currentTarget.value;

			if (tokenInfo && tokenInfo.address.toLowerCase() !== inputValue.toLowerCase()) {
				tokenInfo = null;
				selectedToken = null;
			}

			if (!inputValue) {
				error = '';
				tokenInfo = null;
				selectedToken = null;
				return;
			}

			saveTokenSelection(inputValue);
		}
	}
</script>

<div class="token-selection-container flex w-full flex-col gap-4">
	<div class="token-header">
		{#if token.name || token.description}
			<div class="flex flex-col">
				{#if token.name}
					<h1 class="break-words text-xl font-semibold text-gray-900 lg:text-xl dark:text-white">
						{token.name}
					</h1>
				{/if}
				{#if token.description}
					<p class="text-sm font-light text-gray-600 lg:text-base dark:text-gray-400">
						{token.description}
					</p>
				{/if}
			</div>
		{/if}
	</div>

	<div class="selection-mode flex gap-2">
		<ButtonSelectOption
			active={selectionMode === 'dropdown'}
			buttonText="Select from list"
			clickHandler={() => setMode('dropdown')}
			dataTestId="dropdown-mode-button"
		/>
		<ButtonSelectOption
			active={selectionMode === 'custom'}
			buttonText="Custom address"
			clickHandler={() => setMode('custom')}
			dataTestId="custom-mode-button"
		/>
	</div>

	{#if selectionMode === 'dropdown'}
		<TokenSelectionModal {selectedToken} onSelect={handleTokenSelect} />
	{/if}

	{#if selectionMode === 'custom'}
		<div class="custom-input">
			<Input
				type="text"
				size="lg"
				placeholder="Enter token address (0x...)"
				bind:value={inputValue}
				on:input={handleInput}
			/>
		</div>
	{/if}

	<div class="token-status">
		{#if checking}
			<div class="flex h-5 flex-row items-center gap-2">
				<Spinner class="h-5 w-5" />
				<span>Checking...</span>
			</div>
		{:else if tokenInfo}
			<div
				class="flex h-5 flex-row items-center gap-2"
				data-testid={`select-token-success-${token.key}`}
			>
				<CheckCircleSolid class="h-5 w-5" color="green" />
				<span>{tokenInfo.name}</span>
				<TokenBalanceComponent tokenBalance={tokenBalances.get(token.key)} />
			</div>
		{:else if error}
			<div class="flex h-5 flex-row items-center gap-2" data-testid="error">
				<CloseCircleSolid class="h-5 w-5" color="red" />
				<span>{error}</span>
			</div>
		{/if}
	</div>
</div>
