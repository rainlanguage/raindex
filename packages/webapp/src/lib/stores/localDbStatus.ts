import type { NetworkSyncStatus, LocalDbStatus, RaindexSyncStatus } from '@rainlanguage/raindex';
import { writable, derived } from 'svelte/store';

export const networkStatuses = writable<Map<number, NetworkSyncStatus>>(new Map());

export const raindexStatuses = writable<Map<string, RaindexSyncStatus>>(new Map());

function raindexStatusKey(status: RaindexSyncStatus): string {
	return `${status.raindexId.chainId}:${status.raindexId.raindexAddress}`;
}

function isRaindexSyncStatus(
	status: NetworkSyncStatus | RaindexSyncStatus
): status is RaindexSyncStatus {
	return 'raindexId' in status;
}

export function updateNetworkStatus(status: NetworkSyncStatus) {
	networkStatuses.update((map) => {
		map.set(status.chainId, status);
		return new Map(map);
	});
}

export function updateRaindexStatus(status: RaindexSyncStatus) {
	raindexStatuses.update((map) => {
		const key = raindexStatusKey(status);
		map.set(key, status);
		return new Map(map);
	});
}

export function updateStatus(status: NetworkSyncStatus | RaindexSyncStatus) {
	if (isRaindexSyncStatus(status)) {
		updateRaindexStatus(status);
	} else {
		updateNetworkStatus(status);
	}
}

export const aggregateStatus = derived(
	[networkStatuses, raindexStatuses],
	([$networkMap, $raindexMap]) => {
		const allStatuses: { status: LocalDbStatus; error?: string }[] = [
			...Array.from($networkMap.values()),
			...Array.from($raindexMap.values())
		];

		if (allStatuses.length === 0) {
			return { status: 'active' as LocalDbStatus, error: undefined };
		}

		const failure = allStatuses.find((s) => s.status === 'failure');
		if (failure) {
			return { status: 'failure' as LocalDbStatus, error: failure.error };
		}

		const syncing = allStatuses.find((s) => s.status === 'syncing');
		if (syncing) {
			return { status: 'syncing' as LocalDbStatus, error: undefined };
		}

		return { status: 'active' as LocalDbStatus, error: undefined };
	}
);

export const localDbStatus = aggregateStatus;
