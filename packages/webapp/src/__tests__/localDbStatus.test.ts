import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { NetworkSyncStatus, RaindexSyncStatus } from '@rainlanguage/raindex';
import {
	networkStatuses,
	raindexStatuses,
	updateNetworkStatus,
	updateRaindexStatus,
	updateStatus,
	aggregateStatus,
	localDbSyncGate,
	resetLocalDbStatus,
	seedLocalDbSyncSnapshot
} from '../lib/stores/localDbStatus';

describe('localDbStatus store', () => {
	beforeEach(() => {
		resetLocalDbStatus();
	});

	describe('networkStatuses', () => {
		it('starts with empty map', () => {
			const map = get(networkStatuses);
			expect(map.size).toBe(0);
		});

		it('can set network statuses directly', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			networkStatuses.update((map) => {
				map.set(137, status);
				return new Map(map);
			});

			const map = get(networkStatuses);
			expect(map.size).toBe(1);
			expect(map.get(137)).toEqual(status);
		});
	});

	describe('raindexStatuses', () => {
		it('starts with empty map', () => {
			const map = get(raindexStatuses);
			expect(map.size).toBe(0);
		});
	});

	describe('updateNetworkStatus', () => {
		it('adds new network status to map', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateNetworkStatus(status);

			const map = get(networkStatuses);
			expect(map.size).toBe(1);
			expect(map.get(137)).toEqual(status);
		});

		it('updates existing network status', () => {
			const initial: NetworkSyncStatus = {
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			};
			const updated: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};

			updateNetworkStatus(initial);
			updateNetworkStatus(updated);

			const map = get(networkStatuses);
			expect(map.size).toBe(1);
			expect(map.get(137)?.status).toBe('active');
		});

		it('handles multiple networks independently', () => {
			const polygon: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			const arbitrum: NetworkSyncStatus = {
				chainId: 42161,
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateNetworkStatus(polygon);
			updateNetworkStatus(arbitrum);

			const map = get(networkStatuses);
			expect(map.size).toBe(2);
			expect(map.get(137)?.status).toBe('active');
			expect(map.get(42161)?.status).toBe('syncing');
		});

		it('stores error field when present', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'failure',
				schedulerState: 'leader',
				error: 'RPC connection failed'
			};

			updateNetworkStatus(status);

			const map = get(networkStatuses);
			expect(map.get(137)?.error).toBe('RPC connection failed');
		});
	});

	describe('updateRaindexStatus', () => {
		it('adds new raindex status to map', () => {
			const status: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			};

			updateRaindexStatus(status);

			const map = get(raindexStatuses);
			expect(map.size).toBe(1);
			const key = '137:0x1234567890123456789012345678901234567890';
			expect(map.get(key)).toEqual(status);
		});

		it('updates existing raindex status', () => {
			const initial: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			};
			const updated: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			};

			updateRaindexStatus(initial);
			updateRaindexStatus(updated);

			const map = get(raindexStatuses);
			expect(map.size).toBe(1);
			const key = '137:0x1234567890123456789012345678901234567890';
			expect(map.get(key)?.status).toBe('active');
		});

		it('handles multiple raindexes on same chain', () => {
			const raindex1: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1111111111111111111111111111111111111111'
				},
				status: 'active',
				schedulerState: 'leader'
			};
			const raindex2: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x2222222222222222222222222222222222222222'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateRaindexStatus(raindex1);
			updateRaindexStatus(raindex2);

			const map = get(raindexStatuses);
			expect(map.size).toBe(2);
		});

		it('handles raindexes on different chains', () => {
			const polygonRaindex: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			};
			const arbitrumRaindex: RaindexSyncStatus = {
				raindexId: {
					chainId: 42161,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateRaindexStatus(polygonRaindex);
			updateRaindexStatus(arbitrumRaindex);

			const map = get(raindexStatuses);
			expect(map.size).toBe(2);
		});

		it('stores error field when present', () => {
			const status: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'failure',
				schedulerState: 'leader',
				error: 'Decode error'
			};

			updateRaindexStatus(status);

			const map = get(raindexStatuses);
			const key = '137:0x1234567890123456789012345678901234567890';
			expect(map.get(key)?.error).toBe('Decode error');
		});
	});

	describe('updateStatus', () => {
		it('dispatches NetworkSyncStatus to updateNetworkStatus', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};

			updateStatus(status);

			const networkMap = get(networkStatuses);
			const raindexMap = get(raindexStatuses);
			expect(networkMap.size).toBe(1);
			expect(raindexMap.size).toBe(0);
			expect(networkMap.get(137)).toEqual(status);
		});

		it('dispatches RaindexSyncStatus to updateRaindexStatus', () => {
			const status: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateStatus(status);

			const networkMap = get(networkStatuses);
			const raindexMap = get(raindexStatuses);
			expect(networkMap.size).toBe(0);
			expect(raindexMap.size).toBe(1);
		});

		it('distinguishes types by presence of raindexId field', () => {
			const networkStatus: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			const raindexStatus: RaindexSyncStatus = {
				raindexId: {
					chainId: 42161,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateStatus(networkStatus);
			updateStatus(raindexStatus);

			const networkMap = get(networkStatuses);
			const raindexMap = get(raindexStatuses);
			expect(networkMap.size).toBe(1);
			expect(raindexMap.size).toBe(1);
		});
	});

	describe('callback integration', () => {
		it('routes mixed status updates to correct stores via single updateStatus function', () => {
			const networkStatus1: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			const raindexStatus1: RaindexSyncStatus = {
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			};
			const networkStatus2: NetworkSyncStatus = {
				chainId: 42161,
				status: 'syncing',
				schedulerState: 'leader'
			};
			const raindexStatus2: RaindexSyncStatus = {
				raindexId: {
					chainId: 42161,
					raindexAddress: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcd'
				},
				status: 'active',
				schedulerState: 'leader'
			};

			updateStatus(networkStatus1);
			updateStatus(raindexStatus1);
			updateStatus(networkStatus2);
			updateStatus(raindexStatus2);

			const networkMap = get(networkStatuses);
			const raindexMap = get(raindexStatuses);

			expect(networkMap.size).toBe(2);
			expect(raindexMap.size).toBe(2);

			expect(networkMap.get(137)?.status).toBe('active');
			expect(networkMap.get(42161)?.status).toBe('syncing');

			const raindexKey1 = '137:0x1234567890123456789012345678901234567890';
			const raindexKey2 = '42161:0xabcdefabcdefabcdefabcdefabcdefabcdefabcd';
			expect(raindexMap.get(raindexKey1)?.status).toBe('syncing');
			expect(raindexMap.get(raindexKey1)?.phaseMessage).toBe('Fetching latest block');
			expect(raindexMap.get(raindexKey2)?.status).toBe('active');
		});

		it('simulates scheduler callback receiving interleaved network and raindex updates', () => {
			const updates = [
				{
					chainId: 137,
					status: 'syncing' as const,
					schedulerState: 'leader' as const
				},
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'syncing' as const,
					schedulerState: 'leader' as const,
					phaseMessage: 'Running bootstrap'
				},
				{
					raindexId: {
						chainId: 137,
						raindexAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'active' as const,
					schedulerState: 'leader' as const
				},
				{
					chainId: 137,
					status: 'active' as const,
					schedulerState: 'leader' as const
				}
			];

			for (const update of updates) {
				updateStatus(update as NetworkSyncStatus | RaindexSyncStatus);
			}

			const networkMap = get(networkStatuses);
			const raindexMap = get(raindexStatuses);

			expect(networkMap.get(137)?.status).toBe('active');
			const raindexKey = '137:0x1111111111111111111111111111111111111111';
			expect(raindexMap.get(raindexKey)?.status).toBe('active');
		});
	});

	describe('aggregateStatus', () => {
		it('returns active when both maps are empty', () => {
			const result = get(aggregateStatus);
			expect(result.status).toBe('active');
			expect(result.error).toBeUndefined();
		});

		it('returns active when all statuses are active', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('active');
			expect(result.error).toBeUndefined();
		});

		it('returns syncing when any status is syncing and none are failure', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'syncing',
				schedulerState: 'leader'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('syncing');
			expect(result.error).toBeUndefined();
		});

		it('returns failure when any network has failure', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'failure',
				schedulerState: 'leader',
				error: 'RPC timeout'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBe('RPC timeout');
		});

		it('returns failure when any raindex has failure', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'failure',
				schedulerState: 'leader',
				error: 'Decode error'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBe('Decode error');
		});

		it('failure takes precedence over syncing', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'failure',
				schedulerState: 'leader',
				error: 'Connection refused'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBe('Connection refused');
		});

		it('syncing takes precedence over active', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Building SQL batch'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('syncing');
		});

		it('returns first failure error when multiple failures exist', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'failure',
				schedulerState: 'leader',
				error: 'First error'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'failure',
				schedulerState: 'leader',
				error: 'Second error'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBeDefined();
		});

		it('updates reactively when statuses change', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});

			let result = get(aggregateStatus);
			expect(result.status).toBe('active');

			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});

			result = get(aggregateStatus);
			expect(result.status).toBe('syncing');

			updateNetworkStatus({
				chainId: 137,
				status: 'failure',
				schedulerState: 'leader',
				error: 'Error occurred'
			});

			result = get(aggregateStatus);
			expect(result.status).toBe('failure');
		});
	});

	describe('localDbSyncGate', () => {
		it('is idle before any configured sync status has been observed', () => {
			expect(get(localDbSyncGate)).toEqual({ status: 'idle' });
		});

		it('does not block data views for non-destructive initial sync phases', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			});

			expect(get(localDbSyncGate)).toEqual({ status: 'ready' });
		});

		it('blocks data views while the initial dump is downloading', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Downloading initial dump'
			});

			expect(get(localDbSyncGate)).toEqual({
				status: 'syncing',
				phaseMessage: 'Downloading initial dump'
			});
		});

		it('blocks data views while bootstrap is running', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Running bootstrap'
			});

			expect(get(localDbSyncGate)).toEqual({
				status: 'syncing',
				phaseMessage: 'Running bootstrap'
			});
		});

		it('keeps blocking after a destructive phase until the database becomes active', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Downloading initial dump'
			});

			expect(get(localDbSyncGate)).toEqual({
				status: 'syncing',
				phaseMessage: 'Downloading initial dump'
			});

			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			});

			expect(get(localDbSyncGate)).toEqual({
				status: 'syncing',
				phaseMessage: 'Downloading initial dump'
			});

			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			});

			expect(get(localDbSyncGate)).toEqual({ status: 'ready' });
		});

		it('shows failure if the initial destructive sync fails after it started', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Running bootstrap'
			});

			expect(get(localDbSyncGate)).toEqual({
				status: 'syncing',
				phaseMessage: 'Running bootstrap'
			});

			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'failure',
				schedulerState: 'leader',
				error: 'bootstrap failed'
			});

			expect(get(localDbSyncGate)).toEqual({
				status: 'failure',
				error: 'bootstrap failed'
			});
		});

		it('unblocks data views once all observed statuses become active', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Running bootstrap'
			});

			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateRaindexStatus({
				raindexId: {
					chainId: 137,
					raindexAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			});

			expect(get(localDbSyncGate)).toEqual({ status: 'ready' });
		});

		it('does not block data views during later background syncs after initial readiness', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			expect(get(localDbSyncGate)).toEqual({ status: 'ready' });

			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});

			expect(get(localDbSyncGate)).toEqual({ status: 'ready' });
		});

		it('shows initial sync failure before data views are ready', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'failure',
				schedulerState: 'leader',
				error: 'dump failed'
			});

			expect(get(localDbSyncGate)).toEqual({
				status: 'failure',
				error: 'dump failed'
			});
		});

		it('seeds observed statuses from the client sync snapshot', () => {
			seedLocalDbSyncSnapshot({
				configured: true,
				healthy: true,
				status: 'active',
				schedulerState: 'leader',
				networks: [
					{
						chainId: 137,
						networkKey: 'polygon',
						status: 'active',
						schedulerState: 'leader',
						raindexCount: 1,
						ready: true
					}
				],
				raindexes: [
					{
						raindexId: {
							chainId: 137,
							raindexAddress: '0x1234567890123456789012345678901234567890'
						},
						raindexKey: 'polygon-raindex',
						networkKey: 'polygon',
						status: 'active',
						schedulerState: 'leader',
						ready: true
					}
				]
			});

			expect(get(networkStatuses).get(137)?.status).toBe('active');
			expect(get(localDbSyncGate)).toEqual({ status: 'ready' });
		});
	});
});
