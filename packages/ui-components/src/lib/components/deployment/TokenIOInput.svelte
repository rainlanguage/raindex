<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import { type OrderIOCfg, type TokenInfo, type VaultType } from '@rainlanguage/orderbook';
	import { onMount } from 'svelte';
	import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';
	import type { TokenBalance } from '$lib/types/tokenBalance';
	import VaultIdInformation from './VaultIdInformation.svelte';

	const builder = useRaindexOrderBuilder();

	export let label: 'Input' | 'Output';
	export let vault: OrderIOCfg;
	export let tokenBalances: Map<string, TokenBalance> = new Map();

	let tokenInfo: TokenInfo | null = null;
	let inputValue: string = '';
	let error: string = '';

	onMount(() => {
		if (!vault.token?.key) return;

		const result = builder.getVaultIds();
		if (result.error) {
			error = result.error.msg;
			return;
		}
		const vaultIds = result.value;
		const vaultMap = vaultIds.get(label.toLowerCase());
		if (vaultMap) {
			const vaultId = vaultMap.get(vault.token.key);
			inputValue = vaultId || '';
		}
	});

	const handleGetTokenInfo = async () => {
		if (!vault.token?.key) return;
		try {
			let result = await builder.getTokenInfo(vault.token?.key);
			if (result.error) {
				error = result.error.msg;
				return;
			}
			tokenInfo = result.value;
		} catch (e) {
			const errorMessage = (e as Error).message
				? (e as Error).message
				: 'Error getting token info.';
			error = errorMessage;
		}
	};

	const handleInput = async () => {
		if (!vault.token) {
			error = 'Vault token is not set.';
			return;
		}
		error = '';
		try {
			builder.setVaultId(label.toLowerCase() as VaultType, vault.token.key, inputValue);
		} catch (e) {
			const errorMessage = (e as Error).message ? (e as Error).message : 'Error setting vault ID.';
			error = errorMessage;
		}
	};

	$: if (vault.token?.key) {
		handleGetTokenInfo();
	}
</script>

<div class="flex w-full flex-col gap-6">
	<div class="flex w-full flex-col gap-2">
		<div class="flex items-center gap-2">
			<VaultIdInformation
				title={`${label} ${tokenInfo?.symbol ? `(${tokenInfo.symbol})` : ''}`}
				description={`${tokenInfo?.symbol || 'Token'} vault ID`}
				tokenBalance={tokenBalances.get(vault.token?.key || '')}
			/>
		</div>
	</div>
	<div class="flex flex-col gap-2">
		<Input
			data-testid="vault-id-input"
			size="lg"
			type="text"
			placeholder="Enter vault ID"
			bind:value={inputValue}
			on:input={handleInput}
		/>
		{#if error}
			<p class="text-red-500">{error}</p>
		{/if}
	</div>
</div>
