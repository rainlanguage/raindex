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

const deferred = <T>() => {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((resolvePromise) => {
		resolve = resolvePromise;
	});
	return { promise, resolve };
};

const serializedLocks = () => {
	let previous = Promise.resolve<unknown>(undefined);
	return vi.fn((_name: string, callback: () => Promise<unknown>) => {
		const current = previous.then(callback);
		previous = current.catch(() => undefined);
		return current;
	});
};

const reusableDatabase = () => ({
	query: vi
		.fn()
		.mockResolvedValueOnce(successful('[{"ready":1}]'))
		.mockResolvedValueOnce(successful('[{"ready":1}]')),
	installSnapshot: vi.fn(),
	close: vi.fn().mockResolvedValue(successful(undefined)),
	free: vi.fn()
});

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
		const localDb = {
			query,
			installSnapshot,
			close: vi.fn().mockResolvedValue(successful(undefined)),
			free: vi.fn()
		};
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
		const localDb = {
			query,
			installSnapshot,
			close: vi.fn().mockResolvedValue(successful(undefined)),
			free: vi.fn()
		};
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

	it('holds the cross-tab lock until the database runtime is freed', async () => {
		const localDbs = [reusableDatabase(), reusableDatabase()];
		const closeResult = deferred<{ value: undefined; error: undefined }>();
		localDbs[0].close.mockImplementation(() => closeResult.promise);
		sqliteMocks.newDatabase
			.mockResolvedValueOnce(successful(localDbs[0]))
			.mockResolvedValueOnce(successful(localDbs[1]));
		const clients = [0, 1].map(() => ({ free: vi.fn() }));
		const registries = clients.map((client) => ({
			getRaindexClient: vi.fn().mockResolvedValue(successful(client)),
			free: vi.fn()
		}));
		registryMocks.newRegistry
			.mockResolvedValueOnce(successful(registries[0]))
			.mockResolvedValueOnce(successful(registries[1]));
		const request = serializedLocks();
		vi.stubGlobal('navigator', { locks: { request } });

		const first = await initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		const secondPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await Promise.resolve();
		await Promise.resolve();

		expect(request).toHaveBeenCalledTimes(2);
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1);

		const firstFree = first.free();
		await Promise.resolve();
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1);
		closeResult.resolve(successful(undefined));
		await firstFree;
		const second = await secondPending;

		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(2);
		expect(second.snapshotBootstrap.status).toBe('reused');
		expect(clients[0].free).toHaveBeenCalledTimes(1);
		expect(localDbs[0].close).toHaveBeenCalledTimes(1);
		expect(localDbs[0].free).toHaveBeenCalledTimes(1);
		expect(registries[0].free).toHaveBeenCalledTimes(1);
		expect(localDbs[0].close.mock.invocationCallOrder[0]).toBeLessThan(
			localDbs[0].free.mock.invocationCallOrder[0]
		);
		expect(localDbs[0].close.mock.invocationCallOrder[0]).toBeLessThan(
			sqliteMocks.newDatabase.mock.invocationCallOrder[1]
		);

		await second.free();
		await second.free();
		expect(clients[1].free).toHaveBeenCalledTimes(1);
		expect(localDbs[1].close).toHaveBeenCalledTimes(1);
		expect(localDbs[1].free).toHaveBeenCalledTimes(1);
		expect(registries[1].free).toHaveBeenCalledTimes(1);
	});

	it('does not start Raindex when snapshot authentication fails', async () => {
		const query = vi.fn().mockResolvedValue(successful('[]'));
		const installSnapshot = vi.fn().mockResolvedValue({
			value: undefined,
			error: { readableMsg: 'Snapshot SHA-256 mismatch' }
		});
		const close = vi.fn().mockResolvedValue(successful(undefined));
		const free = vi.fn();
		sqliteMocks.newDatabase.mockResolvedValue(successful({ query, installSnapshot, close, free }));
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
		expect(close).toHaveBeenCalledTimes(1);
		expect(free).toHaveBeenCalledTimes(1);
		expect(freeRegistry).toHaveBeenCalledTimes(1);
	});

	it('does not install over the database when the schema-presence query fails', async () => {
		const request = serializedLocks();
		vi.stubGlobal('navigator', { locks: { request } });
		const schemaQuery = deferred<{
			value: undefined;
			error: { readableMsg: string };
		}>();
		const installSnapshot = vi.fn();
		const database = {
			query: vi.fn(() => schemaQuery.promise),
			installSnapshot,
			close: vi.fn().mockResolvedValue(successful(undefined)),
			free: vi.fn()
		};
		const secondDatabase = reusableDatabase();
		sqliteMocks.newDatabase
			.mockResolvedValueOnce(successful(database))
			.mockResolvedValueOnce(successful(secondDatabase));
		const getRaindexClient = vi.fn();
		const firstRegistry = { getRaindexClient, free: vi.fn() };
		const secondClient = { free: vi.fn() };
		const secondRegistry = {
			getRaindexClient: vi.fn().mockResolvedValue(successful(secondClient)),
			free: vi.fn()
		};
		registryMocks.newRegistry
			.mockResolvedValueOnce(successful(firstRegistry))
			.mockResolvedValueOnce(successful(secondRegistry));

		const firstPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(database.query).toHaveBeenCalledTimes(1));
		const secondPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(2));
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1);

		schemaQuery.resolve({
			value: undefined,
			error: { readableMsg: 'sqlite_master read failed' }
		});
		await expect(firstPending).rejects.toThrow('sqlite_master read failed');
		const second = await secondPending;

		expect(installSnapshot).not.toHaveBeenCalled();
		expect(getRaindexClient).not.toHaveBeenCalled();
		expect(database.close).toHaveBeenCalledTimes(1);
		expect(database.free).toHaveBeenCalledTimes(1);
		expect(firstRegistry.free).toHaveBeenCalledTimes(1);
		await second.free();
	});

	it('does not install over the database when the watermark-presence query fails', async () => {
		const request = serializedLocks();
		vi.stubGlobal('navigator', { locks: { request } });
		const watermarkQuery = deferred<{
			value: undefined;
			error: { readableMsg: string };
		}>();
		const installSnapshot = vi.fn();
		const database = {
			query: vi
				.fn()
				.mockResolvedValueOnce(successful('[{"ready":1}]'))
				.mockImplementationOnce(() => watermarkQuery.promise),
			installSnapshot,
			close: vi.fn().mockResolvedValue(successful(undefined)),
			free: vi.fn()
		};
		const secondDatabase = reusableDatabase();
		sqliteMocks.newDatabase
			.mockResolvedValueOnce(successful(database))
			.mockResolvedValueOnce(successful(secondDatabase));
		const getRaindexClient = vi.fn();
		const firstRegistry = { getRaindexClient, free: vi.fn() };
		const secondClient = { free: vi.fn() };
		const secondRegistry = {
			getRaindexClient: vi.fn().mockResolvedValue(successful(secondClient)),
			free: vi.fn()
		};
		registryMocks.newRegistry
			.mockResolvedValueOnce(successful(firstRegistry))
			.mockResolvedValueOnce(successful(secondRegistry));

		const firstPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(database.query).toHaveBeenCalledTimes(2));
		const secondPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(2));
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1);

		watermarkQuery.resolve({
			value: undefined,
			error: { readableMsg: 'watermark read failed' }
		});
		await expect(firstPending).rejects.toThrow('watermark read failed');
		const second = await secondPending;

		expect(installSnapshot).not.toHaveBeenCalled();
		expect(getRaindexClient).not.toHaveBeenCalled();
		expect(database.close).toHaveBeenCalledTimes(1);
		expect(database.free).toHaveBeenCalledTimes(1);
		expect(firstRegistry.free).toHaveBeenCalledTimes(1);
		await second.free();
	});

	it('releases registry ownership when lock acquisition rejects before its callback', async () => {
		const lockError = new Error('Web Locks unavailable');
		const request = vi.fn().mockRejectedValue(lockError);
		vi.stubGlobal('navigator', { locks: { request } });
		const freeRegistry = vi.fn();
		registryMocks.newRegistry.mockResolvedValue(
			successful({ getRaindexClient: vi.fn(), free: freeRegistry })
		);

		await expect(
			initializeSnapshotPoc('https://registry.example', () => {}, {
				loadRegistry: registryMocks.newRegistry
			})
		).rejects.toBe(lockError);

		expect(freeRegistry).toHaveBeenCalledTimes(1);
		expect(sqliteMocks.newDatabase).not.toHaveBeenCalled();
	});

	it('does not initialize SQLite when registry loading fails', async () => {
		registryMocks.newRegistry.mockResolvedValue({
			value: undefined,
			error: { readableMsg: 'registry failed' }
		});

		await expect(
			initializeSnapshotPoc('https://registry.example', () => {}, {
				loadRegistry: registryMocks.newRegistry
			})
		).rejects.toThrow('registry failed');

		expect(sqliteMocks.init).not.toHaveBeenCalled();
		expect(sqliteMocks.newDatabase).not.toHaveBeenCalled();
	});

	it('frees the registry when SQLite WASM initialization fails', async () => {
		const initError = new Error('WASM initialization failed');
		sqliteMocks.init.mockRejectedValue(initError);
		const freeRegistry = vi.fn();
		registryMocks.newRegistry.mockResolvedValue(
			successful({ getRaindexClient: vi.fn(), free: freeRegistry })
		);

		await expect(
			initializeSnapshotPoc('https://registry.example', () => {}, {
				loadRegistry: registryMocks.newRegistry
			})
		).rejects.toBe(initError);

		expect(freeRegistry).toHaveBeenCalledTimes(1);
		expect(sqliteMocks.newDatabase).not.toHaveBeenCalled();
	});

	it('releases the lock and registry when opening the database fails', async () => {
		const request = serializedLocks();
		vi.stubGlobal('navigator', { locks: { request } });
		const firstOpen = deferred<{
			value: undefined;
			error: { readableMsg: string };
		}>();
		const firstRegistry = { getRaindexClient: vi.fn(), free: vi.fn() };
		const secondClient = { free: vi.fn() };
		const secondRegistry = {
			getRaindexClient: vi.fn().mockResolvedValue(successful(secondClient)),
			free: vi.fn()
		};
		registryMocks.newRegistry
			.mockResolvedValueOnce(successful(firstRegistry))
			.mockResolvedValueOnce(successful(secondRegistry));
		const openError = {
			value: undefined,
			error: { readableMsg: 'open failed' }
		};
		const secondDatabase = reusableDatabase();
		sqliteMocks.newDatabase
			.mockImplementationOnce(() => firstOpen.promise)
			.mockResolvedValueOnce(successful(secondDatabase));

		const firstPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1));
		const secondPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(2));
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1);

		firstOpen.resolve(openError);
		await expect(firstPending).rejects.toThrow('open failed');
		const second = await secondPending;

		expect(firstRegistry.free).toHaveBeenCalledTimes(1);
		expect(firstRegistry.getRaindexClient).not.toHaveBeenCalled();
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(2);
		await second.free();
	});

	it('closes a partially initialized database and lets the next lock waiter proceed', async () => {
		const request = serializedLocks();
		vi.stubGlobal('navigator', { locks: { request } });
		const install = deferred<{
			value: undefined;
			error: { readableMsg: string };
		}>();
		const failedDatabase = {
			query: vi.fn().mockResolvedValue(successful('[]')),
			installSnapshot: vi.fn(() => install.promise),
			close: vi.fn().mockResolvedValue(successful(undefined)),
			free: vi.fn()
		};
		const secondDatabase = reusableDatabase();
		sqliteMocks.newDatabase
			.mockResolvedValueOnce(successful(failedDatabase))
			.mockResolvedValueOnce(successful(secondDatabase));
		const firstRegistry = { getRaindexClient: vi.fn(), free: vi.fn() };
		const secondClient = { free: vi.fn() };
		const secondRegistry = {
			getRaindexClient: vi.fn().mockResolvedValue(successful(secondClient)),
			free: vi.fn()
		};
		registryMocks.newRegistry
			.mockResolvedValueOnce(successful(firstRegistry))
			.mockResolvedValueOnce(successful(secondRegistry));

		const firstPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(failedDatabase.installSnapshot).toHaveBeenCalledTimes(1));
		const secondPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(2));
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1);

		install.resolve({
			value: undefined,
			error: { readableMsg: 'snapshot install failed' }
		});
		await expect(firstPending).rejects.toThrow('snapshot install failed');
		const second = await secondPending;

		expect(failedDatabase.close).toHaveBeenCalledTimes(1);
		expect(failedDatabase.free).toHaveBeenCalledTimes(1);
		expect(failedDatabase.close.mock.invocationCallOrder[0]).toBeLessThan(
			failedDatabase.free.mock.invocationCallOrder[0]
		);
		expect(firstRegistry.free).toHaveBeenCalledTimes(1);
		await second.free();
	});

	it('closes the database when client creation fails', async () => {
		const database = reusableDatabase();
		sqliteMocks.newDatabase.mockResolvedValue(successful(database));
		const registry = {
			getRaindexClient: vi.fn().mockResolvedValue({
				value: undefined,
				error: { readableMsg: 'client failed' }
			}),
			free: vi.fn()
		};
		registryMocks.newRegistry.mockResolvedValue(successful(registry));

		await expect(
			initializeSnapshotPoc('https://registry.example', () => {}, {
				loadRegistry: registryMocks.newRegistry
			})
		).rejects.toThrow('client failed');

		expect(database.close).toHaveBeenCalledTimes(1);
		expect(database.free).toHaveBeenCalledTimes(1);
		expect(registry.free).toHaveBeenCalledTimes(1);
	});

	it('does not leak the lock or wrappers when close returns an error', async () => {
		const request = serializedLocks();
		vi.stubGlobal('navigator', { locks: { request } });
		const firstDatabase = reusableDatabase();
		firstDatabase.close.mockResolvedValue({
			value: undefined,
			error: { readableMsg: 'worker close failed' }
		});
		const secondDatabase = reusableDatabase();
		sqliteMocks.newDatabase
			.mockResolvedValueOnce(successful(firstDatabase))
			.mockResolvedValueOnce(successful(secondDatabase));
		const clients = [{ free: vi.fn() }, { free: vi.fn() }];
		const registries = clients.map((client) => ({
			getRaindexClient: vi.fn().mockResolvedValue(successful(client)),
			free: vi.fn()
		}));
		registryMocks.newRegistry
			.mockResolvedValueOnce(successful(registries[0]))
			.mockResolvedValueOnce(successful(registries[1]));

		const first = await initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		const secondPending = initializeSnapshotPoc('https://registry.example', () => {}, {
			loadRegistry: registryMocks.newRegistry
		});
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(2));
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(1);

		await expect(first.free()).rejects.toThrow('worker close failed');
		const second = await secondPending;

		expect(firstDatabase.free).toHaveBeenCalledTimes(1);
		expect(registries[0].free).toHaveBeenCalledTimes(1);
		expect(sqliteMocks.newDatabase).toHaveBeenCalledTimes(2);
		await second.free();
	});
});
