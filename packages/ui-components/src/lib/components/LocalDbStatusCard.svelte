<script lang="ts">
	import type { LocalDbStatus, NetworkSyncStatus, RaindexSyncStatus } from '@rainlanguage/raindex';
	import LocalDbStatusBadge from './LocalDbStatusBadge.svelte';
	import LocalDbStatusModal from './LocalDbStatusModal.svelte';
	import { ChevronRightOutline } from 'flowbite-svelte-icons';

	export let networkStatuses: Map<number, NetworkSyncStatus> = new Map();
	export let raindexStatuses: Map<string, RaindexSyncStatus> = new Map();

	let modalOpen = false;

	$: networkList = Array.from(networkStatuses.values());
	$: hasNetworks = networkList.length > 0;
	$: hasFailure = networkList.some((s) => s.status === 'failure');
	$: displayStatus = (hasFailure ? 'failure' : 'active') as LocalDbStatus;

	function openModal() {
		modalOpen = true;
	}
</script>

<div
	class="rounded-lg border border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-900"
	data-testid="local-db-status-card"
>
	<button
		type="button"
		class="flex w-full items-center justify-between px-3 py-3 text-left transition hover:bg-gray-50 dark:hover:bg-gray-800"
		on:click={openModal}
		data-testid="local-db-status-header"
	>
		<div class="flex items-center gap-2">
			<span class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
				>LocalDB</span
			>
		</div>
		<div class="flex items-center gap-2">
			<LocalDbStatusBadge status={displayStatus} />
			{#if hasNetworks}
				<ChevronRightOutline class="h-3 w-3 shrink-0" />
			{/if}
		</div>
	</button>
</div>

<LocalDbStatusModal bind:open={modalOpen} {networkStatuses} {raindexStatuses} />
