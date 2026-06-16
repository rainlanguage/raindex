<script lang="ts">
	import { useTransactions } from '$lib/providers/transactions/useTransactions';
	import { CloseOutline } from 'flowbite-svelte-icons';
	import { writable } from 'svelte/store';
	import { getStatusEmoji } from './getStatusEmoji';

	const { transactions } = useTransactions();
	const isDismissed = writable(false);

	// Get the latest transaction (most recent)
	$: latestTransaction =
		$transactions.length > 0 ? $transactions[$transactions.length - 1].state : null;

	// Reset dismiss state when new transaction appears
	$: if (latestTransaction) {
		isDismissed.set(false);
	}

	function dismissTransaction() {
		isDismissed.set(true);
	}
</script>

{#if $latestTransaction && !$isDismissed}
	<div
		class="fixed bottom-0 left-0 right-0 z-40 border-t border-gray-200 bg-white lg:hidden dark:border-gray-700 dark:bg-gray-900"
	>
		<div class="flex items-start gap-3 p-3">
			<!-- Transaction details -->
			<div class="min-w-0 flex-1">
				<div class="mb-1 flex items-center gap-2">
					<span class="text-lg">{getStatusEmoji($latestTransaction.status)}</span>
					<p class="truncate text-sm font-semibold text-gray-600 dark:text-gray-400">
						{$latestTransaction.name}
					</p>
				</div>
				<p class="truncate text-xs text-gray-600 dark:text-gray-400">
					{$latestTransaction.status}
				</p>
				{#if $latestTransaction.errorDetails}
					<p class="truncate text-xs text-red-600 dark:text-red-400">
						{$latestTransaction.errorDetails}
					</p>
				{/if}
				{#if $latestTransaction.links.length > 0}
					<div class="mt-1 flex gap-2">
						{#each $latestTransaction.links.slice(0, 2) as link}
							<a
								href={link.link}
								target="_blank"
								rel="noopener noreferrer"
								class="truncate text-xs text-blue-500 hover:underline"
							>
								{link.label}
							</a>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Close button -->
			<button
				on:click={dismissTransaction}
				class="flex-shrink-0 p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
				title="Dismiss transaction"
				aria-label="Dismiss transaction"
			>
				<CloseOutline class="h-4 w-4" />
			</button>
		</div>
	</div>
{/if}
