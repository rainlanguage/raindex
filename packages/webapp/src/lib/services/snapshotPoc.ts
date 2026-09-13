import init, { SQLiteWasmDatabase, type WasmEncodedResult } from '@rainlanguage/sqlite-web';
import { DotrainRegistry, type RaindexClient } from '@rainlanguage/raindex';
import {
	markLocalDbInitialSyncComplete,
	resetLocalDbStatus,
	updateStatus
} from '$lib/stores/localDbStatus';

const SNAPSHOT_SHA256 = 'a1119bdc84400745b79be8b8906fb4403b645ea168122a95ce5e85c874f71139';
const SNAPSHOT_UNCOMPRESSED_SIZE = 1_051_512_832;
const SNAPSHOT_URL = 'https://raindex-snapshot-poc.tabellio.io/raindex-base.db.gz';
// A changed artifact gets a new OPFS database instead of reusing or deleting
// data installed for a different snapshot identity.
const SNAPSHOT_DATABASE_NAME = `worker-snapshot-poc-${SNAPSHOT_SHA256.slice(0, 16)}.db`;
const SNAPSHOT_DATABASE_LOCK = `raindex-snapshot-database:${SNAPSHOT_DATABASE_NAME}`;

interface SnapshotInstallMetadata {
	bytesWritten: number;
	compression: string;
	elapsedMs: number;
	sha256: string;
}

interface SnapshotCapableDatabase {
	installSnapshot(
		url: string,
		compression: 'gzip' | 'none',
		sha256: string,
		uncompressedSize: number
	): Promise<WasmEncodedResult<string>>;
}

interface ClosableDatabase {
	close(): Promise<WasmEncodedResult<void>>;
}

const requireSnapshotInstaller = (database: SQLiteWasmDatabase): SnapshotCapableDatabase => {
	const candidate = database as unknown as Partial<SnapshotCapableDatabase>;
	if (typeof candidate.installSnapshot !== 'function') {
		throw new Error(
			'The installed @rainlanguage/sqlite-web version does not support snapshot installation'
		);
	}
	return candidate as SnapshotCapableDatabase;
};

const closeDatabase = async (database: SQLiteWasmDatabase): Promise<void> => {
	const candidate = database as unknown as Partial<ClosableDatabase>;
	if (typeof candidate.close !== 'function') {
		throw new Error(
			'The installed @rainlanguage/sqlite-web version does not support explicit close'
		);
	}
	const result = await candidate.close();
	if (result.error) throw new Error(result.error.readableMsg);
};

const combineErrors = (primary: unknown, cleanup: unknown): AggregateError =>
	new AggregateError([primary, cleanup], 'Snapshot runtime failed and cleanup also failed');

const disposeSnapshotResources = async (
	client: RaindexClient | undefined,
	database: SQLiteWasmDatabase | undefined,
	registry: DotrainRegistry,
	releaseDatabase: () => void
): Promise<void> => {
	let firstError: unknown;
	const attempt = async (dispose: (() => void | Promise<void>) | undefined) => {
		if (!dispose) return;
		try {
			await dispose();
		} catch (error) {
			firstError ??= error;
		}
	};

	await attempt(client ? () => client.free() : undefined);
	// close() waits for any in-flight snapshot cancellation to be acknowledged,
	// terminates the SQLite worker, and releases its OPFS handles. The wrapper may
	// only be freed, and ownership may only pass to another tab, after that
	// asynchronous shutdown contract has completed.
	await attempt(database ? () => closeDatabase(database) : undefined);
	await attempt(database ? () => database.free() : undefined);
	await attempt(() => registry.free());
	await attempt(releaseDatabase);

	if (firstError !== undefined) throw firstError;
};

interface InstalledSnapshotBootstrap {
	status: 'installed';
	elapsedMs: number;
	readyElapsedMs: number;
	bytesWritten: number;
}

interface ReusedSnapshotBootstrap {
	status: 'reused';
	elapsedMs: number;
	readyElapsedMs: number;
	bytesWritten: number;
}

interface FailedSnapshotBootstrap {
	status: 'failed';
	elapsedMs: number;
	message: string;
}

export type SnapshotBootstrap =
	| InstalledSnapshotBootstrap
	| ReusedSnapshotBootstrap
	| FailedSnapshotBootstrap;

type PendingSnapshotBootstrap =
	| Omit<InstalledSnapshotBootstrap, 'readyElapsedMs'>
	| Omit<ReusedSnapshotBootstrap, 'readyElapsedMs'>;

export interface SnapshotPocRuntime {
	localDb: SQLiteWasmDatabase;
	raindexClient: RaindexClient;
	registry: DotrainRegistry;
	snapshotBootstrap: InstalledSnapshotBootstrap | ReusedSnapshotBootstrap;
	free(): Promise<void>;
}

interface SnapshotPocDependencies {
	loadRegistry?: (registryUrl: string) => ReturnType<typeof DotrainRegistry.new>;
}

const hasInstalledSnapshot = async (database: SQLiteWasmDatabase): Promise<boolean> => {
	const tableResult = await database.query(
		"SELECT 1 AS ready FROM sqlite_master WHERE type = 'table' AND name = 'target_watermarks' LIMIT 1"
	);
	// A failed read can mean another page still owns the OPFS database. Never
	// interpret that as an empty database and overwrite a valid local snapshot.
	if (tableResult.error) throw new Error(tableResult.error.readableMsg);
	const tables = JSON.parse(tableResult.value) as Array<{ ready: number }>;
	if (tables.length === 0) return false;

	const watermarkResult = await database.query('SELECT 1 AS ready FROM target_watermarks LIMIT 1');
	if (watermarkResult.error) throw new Error(watermarkResult.error.readableMsg);
	const watermarks = JSON.parse(watermarkResult.value) as Array<{
		ready: number;
	}>;
	return watermarks.length > 0;
};

const installSnapshot = async (
	database: SQLiteWasmDatabase,
	startedAt: number,
	onPhase: (message: string) => void
): Promise<PendingSnapshotBootstrap> => {
	if (await hasInstalledSnapshot(database)) {
		return {
			status: 'reused',
			elapsedMs: performance.now() - startedAt,
			bytesWritten: SNAPSHOT_UNCOMPRESSED_SIZE
		};
	}

	onPhase('Downloading and installing database snapshot');
	const installResult = await requireSnapshotInstaller(database).installSnapshot(
		SNAPSHOT_URL,
		'gzip',
		SNAPSHOT_SHA256,
		SNAPSHOT_UNCOMPRESSED_SIZE
	);
	if (installResult.error) throw new Error(installResult.error.readableMsg);

	const metadata = JSON.parse(installResult.value) as SnapshotInstallMetadata;
	const snapshotBootstrap = {
		status: 'installed' as const,
		elapsedMs: performance.now() - startedAt,
		bytesWritten: metadata.bytesWritten
	};
	return snapshotBootstrap;
};

export async function initializeSnapshotPoc(
	registryUrl: string,
	onPhase: (message: string) => void,
	dependencies: SnapshotPocDependencies = {}
): Promise<SnapshotPocRuntime> {
	const startedAt = performance.now();
	let registry: DotrainRegistry | undefined;
	resetLocalDbStatus();

	try {
		onPhase('Loading registry');
		const registryResult = await (dependencies.loadRegistry?.(registryUrl) ??
			DotrainRegistry.new(registryUrl));
		if (registryResult.error) throw new Error(registryResult.error.readableMsg);
		const loadedRegistry = registryResult.value;
		registry = loadedRegistry;

		await init();

		const openRuntime = async (releaseDatabase: () => void): Promise<SnapshotPocRuntime> => {
			let localDb: SQLiteWasmDatabase | undefined;
			let raindexClient: RaindexClient | undefined;
			try {
				onPhase('Opening local database');
				const localDbResult = await SQLiteWasmDatabase.new(SNAPSHOT_DATABASE_NAME);
				if (localDbResult.error) throw new Error(localDbResult.error.readableMsg);
				const openedDb = localDbResult.value;
				localDb = openedDb;

				// The database lock is acquired before this read, so a waiting tab
				// rechecks the snapshot after the previous owner has released it.
				const snapshotBootstrap = await installSnapshot(openedDb, startedAt, onPhase);

				onPhase('Starting live database sync');
				const clientResult = await loadedRegistry.getRaindexClient({
					localDb: openedDb,
					// SQLite Web authenticates the complete byte stream before activation.
					// Raindex then validates schema and every configured target watermark
					// before continuing incremental RPC sync from the installed snapshot.
					localDbProvisioning: 'preinstalled-snapshot',
					statusCallback: updateStatus
				});
				if (clientResult.error) throw new Error(clientResult.error.readableMsg);

				const loadedClient = clientResult.value;
				raindexClient = loadedClient;
				let freePromise: Promise<void> | undefined;
				// The validated snapshot is already queryable. Live catch-up continues in
				// the background and must not hide the installed data.
				markLocalDbInitialSyncComplete();
				return {
					localDb: openedDb,
					raindexClient: loadedClient,
					registry: loadedRegistry,
					snapshotBootstrap: {
						...snapshotBootstrap,
						readyElapsedMs: performance.now() - startedAt
					},
					free: () =>
						(freePromise ??= disposeSnapshotResources(
							loadedClient,
							openedDb,
							loadedRegistry,
							releaseDatabase
						))
				};
			} catch (error) {
				try {
					await disposeSnapshotResources(
						raindexClient,
						localDb,
						loadedRegistry,
						releaseDatabase
					);
				} catch (cleanupError) {
					throw combineErrors(error, cleanupError);
				}
				throw error;
			}
		};

		const locks = globalThis.navigator?.locks;
		if (!locks) return openRuntime(() => {});

		let releaseDatabase = () => {};
		const databaseReleased = new Promise<void>((resolve) => {
			releaseDatabase = resolve;
		});
		let resolveRuntime!: (runtime: SnapshotPocRuntime) => void;
		let rejectRuntime!: (error: unknown) => void;
		const runtimeReady = new Promise<SnapshotPocRuntime>((resolve, reject) => {
			resolveRuntime = resolve;
			rejectRuntime = reject;
		});
		let lockAcquired = false;
		void locks
			.request(SNAPSHOT_DATABASE_LOCK, async () => {
				lockAcquired = true;
				try {
					resolveRuntime(await openRuntime(releaseDatabase));
					// Keep exclusive ownership for as long as the returned runtime can
					// access the OPFS-backed database.
					await databaseReleased;
				} catch (error) {
					rejectRuntime(error);
				}
			})
			.catch((error) => {
				let rejection = error;
				if (!lockAcquired && registry) {
					try {
						registry.free();
					} catch (cleanupError) {
						rejection = combineErrors(error, cleanupError);
					}
				}
				rejectRuntime(rejection);
			});
		return runtimeReady;
	} catch (error) {
		if (registry) {
			try {
				registry.free();
			} catch (cleanupError) {
				throw combineErrors(error, cleanupError);
			}
		}
		throw error;
	}
}
