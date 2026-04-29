<script lang="ts">
	import {
		type OrderBuilderDepositCfg,
		type TokenDeposit,
		type TokenInfo
	} from '@rainlanguage/orderbook';
	import { Input } from 'flowbite-svelte';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { CloseCircleSolid } from 'flowbite-svelte-icons';
	import { onMount } from 'svelte';
	import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';

	export let deposit: OrderBuilderDepositCfg;

	const builder = useRaindexOrderBuilder();

	let error: string = '';
	let currentDeposit: TokenDeposit | undefined;
	let inputValue: string = '';
	let tokenInfo: TokenInfo | null = null;

	onMount(() => {
		setCurrentDeposit();
	});

	const getCurrentDeposit = () => {
		const deposits = builder.getDeposits();
		if (deposits.error) {
			throw new Error(deposits.error.msg);
		}
		return deposits.value.find((d: TokenDeposit) => d.token === deposit.token?.key);
	};

	const setCurrentDeposit = () => {
		try {
			currentDeposit = getCurrentDeposit();
			inputValue = currentDeposit?.amount || '';
		} catch (e) {
			currentDeposit = undefined;
			error = (e as Error).message ? (e as Error).message : 'Error setting current deposit.';
		}
	};

	const getTokenSymbol = async () => {
		if (!deposit.token?.key) return;
		try {
			let result = await builder.getTokenInfo(deposit.token?.key);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			tokenInfo = result.value;
		} catch (e) {
			const errorMessage = (e as Error).message
				? (e as Error).message
				: 'Error getting token info.';
			error = errorMessage;
		}
	};

	async function handlePresetClick(preset: string) {
		if (deposit.token?.key) {
			inputValue = preset;
			await builder.setDeposit(deposit.token?.key, preset);

			try {
				currentDeposit = getCurrentDeposit();
			} catch (e) {
				error = (e as Error).message ? (e as Error).message : 'Error handling preset click.';
			}
		}
	}

	async function handleInput(e: Event) {
		if (deposit.token?.key) {
			if (e.currentTarget instanceof HTMLInputElement) {
				inputValue = e.currentTarget.value;
				await builder.setDeposit(deposit.token.key, e.currentTarget.value);
				try {
					currentDeposit = getCurrentDeposit();
				} catch (e) {
					error = (e as Error).message ? (e as Error).message : 'Error handling input.';
				}
			}
		}
	}

	$: if (deposit.token?.key) {
		getTokenSymbol();
	}
</script>

<div class="flex w-full flex-col gap-6">
	<DeploymentSectionHeader
		title={tokenInfo?.symbol ? `Deposit amount (${tokenInfo?.symbol})` : 'Deposit amount'}
		description={tokenInfo?.symbol
			? `The amount of ${tokenInfo?.symbol} that you want to deposit.`
			: 'The amount that you want to deposit.'}
	/>

	<div class="flex w-full flex-col gap-6">
		{#if deposit.presets}
			<div class="flex w-full flex-wrap gap-4">
				{#each deposit.presets as preset}
					<ButtonSelectOption
						dataTestId={`deposit-preset-${preset}`}
						active={currentDeposit?.amount === preset}
						buttonText={preset}
						clickHandler={() => handlePresetClick(preset)}
					/>
				{/each}
			</div>
		{/if}

		<Input
			size="lg"
			placeholder="Enter deposit amount"
			bind:value={inputValue}
			on:input={(e) => handleInput(e)}
		/>

		{#if error}
			<div class="flex h-5 flex-row items-center gap-2">
				<CloseCircleSolid class="h-5 w-5" color="red" />
				<span>{error}</span>
			</div>
		{/if}
	</div>
</div>
