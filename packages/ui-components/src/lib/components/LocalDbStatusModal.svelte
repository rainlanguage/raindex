<script lang="ts">
	import { Modal } from 'flowbite-svelte';
	import type { NetworkSyncStatus, RaindexSyncStatus } from '@rainlanguage/raindex';
	import LocalDbStatusBadge from './LocalDbStatusBadge.svelte';
	import { getNetworkName } from '$lib/utils/getNetworkName';

	export let open: boolean = false;
	export let networkStatuses: Map<number, NetworkSyncStatus> = new Map();
	export let raindexStatuses: Map<string, RaindexSyncStatus> = new Map();

	$: networkList = Array.from(networkStatuses.values());
	$: raindexList = Array.from(raindexStatuses.values());
	$: networkGroups = buildNetworkGroups(networkList, raindexList);

	interface NetworkGroup {
		chainId: number;
		networkName: string;
		status: NetworkSyncStatus;
		raindexes: RaindexSyncStatus[];
	}

	function buildNetworkGroups(
		networks: NetworkSyncStatus[],
		raindexes: RaindexSyncStatus[]
	): NetworkGroup[] {
		const groups: NetworkGroup[] = [];
		for (const network of networks) {
			const networkRaindexes = raindexes.filter(
				(raindex) => raindex.raindexId.chainId === network.chainId
			);
			groups.push({
				chainId: network.chainId,
				networkName: getNetworkName(network.chainId) ?? `Chain ${network.chainId}`,
				status: network,
				raindexes: networkRaindexes
			});
		}
		return groups;
	}
</script>

<Modal
	bind:open
	size="lg"
	class="dark:border dark:border-gray-700 dark:bg-gray-900"
	dialogClass="fixed top-0 start-0 end-1 h-modal md:inset-0 md:h-full z-50 w-full p-4 flex justify-center items-center h-full"
	data-testid="local-db-status-modal"
>
	<div class="flex flex-col gap-4" slot="header">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Database Sync Status</h3>
	</div>

	<div class="max-h-[60vh] overflow-y-auto">
		{#if networkList.length === 0}
			<p class="text-sm text-gray-500 dark:text-gray-400">No networks are being synced.</p>
		{:else}
			<div class="space-y-4">
				{#each networkGroups as group (group.chainId)}
					<div
						class="rounded-lg border border-gray-200 dark:border-gray-700"
						data-testid="network-group-{group.chainId}"
					>
						<div
							class="flex items-center justify-between border-b border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-800"
						>
							<div class="flex items-center gap-2">
								<span class="font-medium text-gray-900 dark:text-white">{group.networkName}</span>
								{#if group.status.schedulerState === 'notLeader'}
									<span
										class="rounded bg-amber-100 px-1.5 py-0.5 text-[10px] font-medium uppercase text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
									>
										Observing
									</span>
								{/if}
							</div>
							<LocalDbStatusBadge status={group.status.status} />
						</div>

						{#if group.raindexes.length > 0}
							<ul class="divide-y divide-gray-100 dark:divide-gray-800">
								{#each group.raindexes as raindexStatus (raindexStatus.raindexId.raindexAddress)}
									<li class="px-4 py-3">
										<div class="flex items-start justify-between gap-4">
											<div class="min-w-0 flex-1">
												<span
													class="font-mono text-sm text-gray-700 dark:text-gray-300"
													title={raindexStatus.raindexId.raindexAddress}
												>
													{raindexStatus.raindexId.raindexAddress}
												</span>
												{#if raindexStatus.status === 'syncing' && raindexStatus.phaseMessage && raindexStatus.schedulerState !== 'notLeader'}
													<div class="mt-2 text-sm text-sky-600 dark:text-sky-400">
														{raindexStatus.phaseMessage}
													</div>
												{/if}
												{#if raindexStatus.status === 'failure' && raindexStatus.error}
													<div class="mt-2 text-sm text-red-600 dark:text-red-400">
														{raindexStatus.error}
													</div>
												{/if}
											</div>
										</div>
									</li>
								{/each}
							</ul>
						{/if}

						{#if group.status.status === 'failure' && group.status.error}
							<div class="border-t border-gray-200 px-4 py-3 dark:border-gray-700">
								<div class="text-sm text-red-600 dark:text-red-400">
									{group.status.error}
								</div>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</Modal>
