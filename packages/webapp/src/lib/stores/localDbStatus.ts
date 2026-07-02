import type {
	LocalDbSyncSnapshot,
	NetworkSyncStatus,
	RaindexSyncStatus
} from '@rainlanguage/raindex';
import { aggregateLocalDbStatus } from '@rainlanguage/ui-components';
import { writable, derived } from 'svelte/store';

export const networkStatuses = writable<Map<number, NetworkSyncStatus>>(new Map());

export const raindexStatuses = writable<Map<string, RaindexSyncStatus>>(new Map());

export type LocalDbSyncGateState =
	| { status: 'idle' }
	| { status: 'ready' }
	| { status: 'syncing'; phaseMessage?: string }
	| { status: 'failure'; error?: string };

let initialSyncComplete = false;
let blockingInitialSyncStarted = false;
let lastBlockingPhaseMessage: string | undefined;

const BLOCKING_SYNC_PHASE_MESSAGES = new Set(['Downloading initial dump', 'Running bootstrap']);

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

export function seedLocalDbSyncSnapshot(snapshot: LocalDbSyncSnapshot) {
	if (!snapshot.configured) return;

	networkStatuses.set(
		new Map(snapshot.networks.map((status) => [status.chainId, status as NetworkSyncStatus]))
	);
	raindexStatuses.set(
		new Map(
			snapshot.raindexes.map((status) => [raindexStatusKey(status), status as RaindexSyncStatus])
		)
	);
}

export function resetLocalDbStatus() {
	initialSyncComplete = false;
	blockingInitialSyncStarted = false;
	lastBlockingPhaseMessage = undefined;
	networkStatuses.set(new Map());
	raindexStatuses.set(new Map());
}

export const aggregateStatus = derived(
	[networkStatuses, raindexStatuses],
	([$networkMap, $raindexMap]) =>
		aggregateLocalDbStatus([
			...Array.from($networkMap.values()),
			...Array.from($raindexMap.values())
		])
);

export const localDbStatus = aggregateStatus;

export const localDbSyncGate = derived(
	[networkStatuses, raindexStatuses],
	([$networkMap, $raindexMap]): LocalDbSyncGateState => {
		const statuses = [...Array.from($networkMap.values()), ...Array.from($raindexMap.values())];

		if (statuses.length === 0) {
			return { status: 'idle' };
		}

		const aggregate = aggregateLocalDbStatus(statuses);
		if (aggregate.status === 'active') {
			initialSyncComplete = true;
			blockingInitialSyncStarted = false;
			lastBlockingPhaseMessage = undefined;
			return { status: 'ready' };
		}

		if (initialSyncComplete) {
			return { status: 'ready' };
		}

		if (aggregate.status === 'failure') {
			blockingInitialSyncStarted = false;
			lastBlockingPhaseMessage = undefined;
			return { status: 'failure', error: aggregate.error };
		}

		const phaseMessage = Array.from($raindexMap.values()).find(
			(status) =>
				status.status === 'syncing' &&
				status.schedulerState !== 'notLeader' &&
				Boolean(status.phaseMessage) &&
				BLOCKING_SYNC_PHASE_MESSAGES.has(status.phaseMessage as string)
		)?.phaseMessage;

		if (phaseMessage) {
			blockingInitialSyncStarted = true;
			lastBlockingPhaseMessage = phaseMessage;
			return { status: 'syncing', phaseMessage };
		}

		if (blockingInitialSyncStarted) {
			return { status: 'syncing', phaseMessage: lastBlockingPhaseMessage };
		}

		return { status: 'ready' };
	}
);
