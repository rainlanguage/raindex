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
const SNAPSHOT_INSTALL_LOCK = `raindex-snapshot-install:${SNAPSHOT_DATABASE_NAME}`;

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

const requireSnapshotInstaller = (database: SQLiteWasmDatabase): SnapshotCapableDatabase => {
	const candidate = database as unknown as Partial<SnapshotCapableDatabase>;
	if (typeof candidate.installSnapshot !== 'function') {
		throw new Error(
			'The installed @rainlanguage/sqlite-web version does not support snapshot installation'
		);
	}
	return candidate as SnapshotCapableDatabase;
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

const installOrReuseSnapshot = async (
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

	const locks = globalThis.navigator?.locks;
	if (!locks) return installSnapshot(database, startedAt, onPhase);

	return locks.request(SNAPSHOT_INSTALL_LOCK, () =>
		// Another tab may have completed the installation while this tab waited.
		installSnapshot(database, startedAt, onPhase)
	);
};

export async function initializeSnapshotPoc(
	registryUrl: string,
	onPhase: (message: string) => void,
	dependencies: SnapshotPocDependencies = {}
): Promise<SnapshotPocRuntime> {
	const startedAt = performance.now();
	let registry: DotrainRegistry | undefined;
	let localDb: SQLiteWasmDatabase | undefined;
	resetLocalDbStatus();

	try {
		onPhase('Loading registry');
		const registryResult = await (dependencies.loadRegistry?.(registryUrl) ??
			DotrainRegistry.new(registryUrl));
		if (registryResult.error) throw new Error(registryResult.error.readableMsg);
		registry = registryResult.value;

		onPhase('Opening local database');
		await init();
		const localDbResult = await SQLiteWasmDatabase.new(SNAPSHOT_DATABASE_NAME);
		if (localDbResult.error) throw new Error(localDbResult.error.readableMsg);
		localDb = localDbResult.value;

		const snapshotBootstrap = await installOrReuseSnapshot(localDb, startedAt, onPhase);

		onPhase('Starting live database sync');
		const clientResult = await registry.getRaindexClient({
			localDb,
			// SQLite Web authenticates the complete byte stream before activation.
			// Raindex then validates schema and every configured target watermark
			// before continuing incremental RPC sync from the installed snapshot.
			localDbProvisioning: 'preinstalled-snapshot',
			statusCallback: updateStatus
		});
		if (clientResult.error) throw new Error(clientResult.error.readableMsg);

		// The validated snapshot is already queryable. Live catch-up continues in
		// the background and must not hide the installed data.
		markLocalDbInitialSyncComplete();
		return {
			localDb,
			raindexClient: clientResult.value,
			registry,
			snapshotBootstrap: {
				...snapshotBootstrap,
				readyElapsedMs: performance.now() - startedAt
			}
		};
	} catch (error) {
		localDb?.free();
		registry?.free();
		throw error;
	}
}
