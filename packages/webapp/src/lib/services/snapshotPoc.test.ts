import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { networkStatuses, raindexStatuses, resetLocalDbStatus } from '$lib/stores/localDbStatus';

const sqliteMocks = vi.hoisted(() => ({
	init: vi.fn(),
	newDatabase: vi.fn()
}));
const registryMocks = { newRegistry: vi.fn() };

vi.mock('@rainlanguage/sqlite-web', () => ({
	default: sqliteMocks.init,
	SQLiteWasmDatabase: { new: sqliteMocks.newDatabase }
}));
import { initializeSnapshotPoc } from './snapshotPoc';

const successful = <T>(value: T) => ({ value, error: undefined });

describe('snapshot POC bootstrap', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetLocalDbStatus();
		sqliteMocks.init.mockResolvedValue(undefined);
	});

	afterEach(() => vi.unstubAllGlobals());

	it('opens an existing snapshot immediately and starts incremental sync', async () => {
		const query = vi
			.fn()
			.mockResolvedValueOnce(successful('[{"ready":1}]'))
			.mockResolvedValueOnce(successful('[{"ready":1}]'));
		const installSnapshot = vi.fn();
		const localDb = { query, installSnapshot };
		sqliteMocks.newDatabase.mockResolvedValue(successful(localDb));
		const client = { id: 'client' };
		const getRaindexClient = vi.fn().mockResolvedValue(successful(client));
		const registry = { getRaindexClient, free: vi.fn() };
		registryMocks.newRegistry.mockResolvedValue(successful(registry));
		const phases: string[] = [];

		const result = await initializeSnapshotPoc(
			'https://registry.example',
			(phase) => phases.push(phase),
			{ loadRegistry: registryMocks.newRegistry }
		);

		expect(result.localDb).toBe(localDb);
		expect(result.raindexClient).toBe(client);
		expect(result.registry).toBe(registry);
		expect(result.snapshotBootstrap).toMatchObject({
			status: 'reused',
			bytesWritten: 1_051_512_832
		});
		expect(installSnapshot).not.toHaveBeenCalled();
		expect(getRaindexClient).toHaveBeenCalledWith(
			expect.objectContaining({
				localDb,
				localDbProvisioning: 'preinstalled-snapshot'
			})
		);
		expect(phases).toEqual([
			'Loading registry',
			'Opening local database',
			'Starting live database sync'
		]);
	});

	it('installs a fresh snapshot before exposing the client', async () => {
		const query = vi.fn().mockResolvedValue(successful('[]'));
		const installSnapshot = vi.fn().mockResolvedValue(
			successful(
				JSON.stringify({
					bytesWritten: 1_051_512_832,
					compression: 'gzip',
					elapsedMs: 100,
					sha256: 'a1119bdc84400745b79be8b8906fb4403b645ea168122a95ce5e85c874f71139'
				})
			)
		);
		const localDb = { query, installSnapshot };
		sqliteMocks.newDatabase.mockResolvedValue(successful(localDb));
		const client = { id: 'client' };
		const getRaindexClient = vi.fn().mockResolvedValue(successful(client));
		registryMocks.newRegistry.mockResolvedValue(successful({ getRaindexClient, free: vi.fn() }));

		const result = await initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});

		expect(result.snapshotBootstrap).toMatchObject({
			status: 'installed',
			bytesWritten: 1_051_512_832
		});
		expect(sqliteMocks.newDatabase).toHaveBeenCalledWith(
			'worker-snapshot-poc-a1119bdc84400745.db'
		);
		expect(installSnapshot).toHaveBeenCalledTimes(1);
		expect(get(networkStatuses).size).toBe(0);
		expect(get(raindexStatuses).size).toBe(0);
	});

	it('rechecks snapshot state inside the cross-tab installation lock', async () => {
		const query = vi
			.fn()
			.mockResolvedValueOnce(successful('[]'))
			.mockResolvedValueOnce(successful('[{"ready":1}]'))
			.mockResolvedValueOnce(successful('[{"ready":1}]'));
		const installSnapshot = vi.fn();
		const localDb = { query, installSnapshot };
		sqliteMocks.newDatabase.mockResolvedValue(successful(localDb));
		const getRaindexClient = vi.fn().mockResolvedValue(successful({ id: 'client' }));
		registryMocks.newRegistry.mockResolvedValue(successful({ getRaindexClient, free: vi.fn() }));
		const request = vi.fn(async (_name: string, callback: () => Promise<unknown>) => callback());
		vi.stubGlobal('navigator', { locks: { request } });

		const result = await initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});

		expect(request).toHaveBeenCalledTimes(1);
		expect(installSnapshot).not.toHaveBeenCalled();
		expect(result.snapshotBootstrap.status).toBe('reused');
	});

	it('does not start Raindex when snapshot authentication fails', async () => {
		const query = vi.fn().mockResolvedValue(successful('[]'));
		const installSnapshot = vi.fn().mockResolvedValue({
			value: undefined,
			error: { readableMsg: 'Snapshot SHA-256 mismatch' }
		});
		const free = vi.fn();
		sqliteMocks.newDatabase.mockResolvedValue(successful({ query, installSnapshot, free }));
		const getRaindexClient = vi.fn();
		const freeRegistry = vi.fn();
		registryMocks.newRegistry.mockResolvedValue(
			successful({ getRaindexClient, free: freeRegistry })
		);

		await expect(
			initializeSnapshotPoc('https://registry.example', () => {}, {
				loadRegistry: registryMocks.newRegistry
			})
		).rejects.toThrow('Snapshot SHA-256 mismatch');
		expect(getRaindexClient).not.toHaveBeenCalled();
		expect(free).toHaveBeenCalledTimes(1);
		expect(freeRegistry).toHaveBeenCalledTimes(1);
	});
});
