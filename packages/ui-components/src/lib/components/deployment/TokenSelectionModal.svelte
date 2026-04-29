<script lang="ts">
	import { Input, Button, Modal } from 'flowbite-svelte';
	import { SearchOutline, CheckCircleSolid, ChevronDownSolid } from 'flowbite-svelte-icons';
	import type { ExtendedTokenInfo } from '@rainlanguage/orderbook';
	import { useRaindexOrderBuilder } from '$lib/hooks/useRaindexOrderBuilder';
	import { onMount, tick } from 'svelte';

	export let selectedToken: ExtendedTokenInfo | null = null;
	export let onSelect: (token: ExtendedTokenInfo) => void;

	let modalOpen = false;
	let searchQuery = '';
	let tokens: ExtendedTokenInfo[] = [];
	let isSearching = false;
	let failedImages: Set<string> = new Set();

	const builder = useRaindexOrderBuilder();

	async function loadTokens(search?: string) {
		isSearching = true;

		const result = await builder.getAllTokens(search);
		if (result.error) {
			tokens = [];
		} else {
			tokens = result.value;
		}

		isSearching = false;
	}

	function handleSearch(event: Event) {
		const target = event.target as HTMLInputElement;
		searchQuery = target.value;
		loadTokens(searchQuery || undefined);
	}

	onMount(() => loadTokens());

	function formatAddress(address: string): string {
		return `${address.slice(0, 6)}...${address.slice(-4)}`;
	}

	function handleTokenSelect(token: ExtendedTokenInfo) {
		onSelect(token);
		modalOpen = false;
	}

	function handleImageError(address: string) {
		failedImages = new Set([...failedImages, address]);
	}

	$: displayText = selectedToken
		? `${selectedToken.name} (${selectedToken.symbol})`
		: 'Select a token...';

	$: if (modalOpen) {
		tick().then(() => {
			const input = document.querySelector('.token-search-input') as HTMLInputElement;
			if (input) {
				input.focus();
			}
		});
	}
</script>

<div class="token-dropdown">
	<div class="relative w-full">
		<Button
			color="alternative"
			class="flex w-full justify-between overflow-hidden overflow-ellipsis pl-4 pr-2 text-left"
			size="lg"
			on:click={() => (modalOpen = true)}
		>
			<div class="flex-grow overflow-hidden">
				<span class="text-gray-900 dark:text-white">{displayText}</span>
			</div>
			<ChevronDownSolid class="ml-2 h-4 w-4 text-black dark:text-white" />
		</Button>

		<Modal bind:open={modalOpen} size="md" class="w-full max-w-lg">
			<div slot="header" class="flex w-full items-center justify-between">
				<h3 class="text-xl font-medium text-gray-900 dark:text-white">Select a token</h3>
			</div>
			<div class="relative w-full border-b border-gray-200 p-2 dark:border-gray-600">
				<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-5">
					<SearchOutline class="h-4 w-4 text-gray-500 dark:text-gray-400" />
				</div>
				<Input
					type="text"
					placeholder="Search tokens..."
					bind:value={searchQuery}
					on:input={handleSearch}
					class="token-search-input pl-10"
				/>
			</div>

			<div class="token-list max-h-80 overflow-y-auto">
				{#if isSearching}
					<div class="p-4 text-center text-gray-500 dark:text-gray-400">
						<p>Searching tokens...</p>
					</div>
				{:else}
					{#each tokens as token (token.address)}
						<div
							class="token-item flex cursor-pointer items-center border-b border-gray-100 p-3 last:border-b-0 hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-700"
							class:bg-blue-50={selectedToken?.address === token.address}
							class:dark:bg-blue-900={selectedToken?.address === token.address}
							class:border-l-4={selectedToken?.address === token.address}
							class:border-l-blue-500={selectedToken?.address === token.address}
							on:click={() => handleTokenSelect(token)}
							on:keydown={(e) => e.key === 'Enter' && handleTokenSelect(token)}
							role="button"
							tabindex="0"
						>
							{#if token.logoUri && !failedImages.has(token.address)}
								<img
									src={token.logoUri}
									alt="{token.symbol} logo"
									class="mr-3 h-8 w-8 rounded-full object-cover"
									on:error={() => handleImageError(token.address)}
								/>
							{:else}
								<div
									class="mr-3 flex h-8 w-8 items-center justify-center rounded-full bg-gray-200 dark:bg-gray-600"
								>
									<span class="text-xs font-medium text-gray-500 dark:text-gray-400">
										{token.symbol?.slice(0, 2) || '??'}
									</span>
								</div>
							{/if}
							<div class="token-info flex-grow">
								<div class="token-name font-medium text-gray-900 dark:text-white">
									{token.name}
								</div>
								<div class="token-details flex gap-2 text-sm text-gray-500 dark:text-gray-400">
									<span class="symbol font-medium">{token.symbol}</span>
									<span class="address">{formatAddress(token.address)}</span>
								</div>
							</div>
							{#if selectedToken?.address === token.address}
								<CheckCircleSolid class="selected-icon h-5 w-5 text-green-500" />
							{/if}
						</div>
					{/each}

					{#if tokens.length === 0}
						<div class="no-results p-4 text-center text-gray-500 dark:text-gray-400">
							<p>No tokens found matching your search.</p>
							<button
								class="mt-2 text-blue-600 underline hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
								on:click={() => {
									searchQuery = '';
									loadTokens();
								}}
							>
								Clear search
							</button>
						</div>
					{/if}
				{/if}
			</div>
		</Modal>
	</div>
</div>
