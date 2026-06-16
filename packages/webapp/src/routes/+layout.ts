import {
	DotrainRainlang,
	type RaindexClient,
	type Address,
	type Hex
} from '@rainlanguage/orderbook';
import init, { SQLiteWasmDatabase } from '@rainlanguage/sqlite-web';
import type { AppStoresInterface } from '@rainlanguage/ui-components';
import { RAINLANG_URL } from '$lib/constants';
import { updateStatus } from '$lib/stores/localDbStatus';
import { writable } from 'svelte/store';
import type { LayoutLoad } from './$types';

export interface LayoutData {
	errorMessage?: string;
	stores: AppStoresInterface | null;
	raindexClient: RaindexClient | null;
	rainlang: DotrainRainlang | null;
	localDb: SQLiteWasmDatabase | null;
}

export const load: LayoutLoad<LayoutData> = async ({ url }) => {
	let errorMessage: string | undefined;

	const rainlangParam = url.searchParams.get('rainlang');
	let rainlangUrl = RAINLANG_URL;

	if (rainlangParam) {
		rainlangUrl = rainlangParam;
		if (typeof localStorage !== 'undefined') {
			try {
				localStorage.setItem('rainlang', rainlangParam);
			} catch {
				// ignore persistence failure
			}
		}
	} else {
		if (typeof localStorage !== 'undefined') {
			try {
				rainlangUrl = localStorage.getItem('rainlang') || RAINLANG_URL;
			} catch {
				rainlangUrl = RAINLANG_URL;
			}
		}
	}

	let rainlang: DotrainRainlang | null = null;
	if (!errorMessage) {
		try {
			const rainlangResult = await DotrainRainlang.new(rainlangUrl);
			if (rainlangResult.error) {
				errorMessage = 'Failed to load rainlang. ' + rainlangResult.error.readableMsg;
			} else {
				rainlang = rainlangResult.value;
			}
		} catch (error: unknown) {
			errorMessage = 'Failed to load rainlang. ' + (error as Error).message;
		}
	}

	let localDb: SQLiteWasmDatabase | null = null;
	if (!errorMessage) {
		try {
			await init();
			const localDbRes = await SQLiteWasmDatabase.new('worker.db');
			if (localDbRes.error) {
				errorMessage = 'Error initializing local database: ' + localDbRes.error.readableMsg;
			} else {
				localDb = localDbRes.value;
			}
		} catch (error: unknown) {
			errorMessage = 'Error initializing local database: ' + (error as Error).message;
		}
	}

	let raindexClient: RaindexClient | null = null;
	try {
		if (!errorMessage && rainlang) {
			const raindexClientRes = await rainlang.getRaindexClient(
				localDb?.query?.bind(localDb),
				localDb?.wipeAndRecreate?.bind(localDb),
				updateStatus
			);
			if (raindexClientRes.error) {
				errorMessage = raindexClientRes.error.readableMsg;
			} else {
				raindexClient = raindexClientRes.value;
			}
		}
	} catch (error: unknown) {
		errorMessage = 'Error initializing RaindexClient: ' + (error as Error).message;
	}

	if (errorMessage) {
		return {
			errorMessage,
			stores: null,
			rainlang,
			localDb,
			raindexClient: null
		};
	}

	return {
		stores: {
			selectedChainIds: writable<number[]>([]),
			showInactiveOrders: writable<boolean>(false),
			// @ts-expect-error initially the value is empty
			orderHash: writable<Hex>(''),
			hideZeroBalanceVaults: writable<boolean>(false),
			hideInactiveOrdersVaults: writable<boolean>(false),
			activeTokens: writable<Address[]>([]),
			activeOrderbookAddresses: writable<Address[]>([]),
			// @ts-expect-error initially the value is empty
			ownerFilter: writable<Address>('')
		},
		rainlang,
		localDb,
		raindexClient
	};
};

export const ssr = false;

if (import.meta.vitest) {
	const { describe, it, expect, beforeEach, vi } = import.meta.vitest;

	const { mockRainlangNew, mockGetRaindexClient, mockInit, mockLocalDbNew } = vi.hoisted(() => ({
		mockRainlangNew: vi.fn(),
		mockGetRaindexClient: vi.fn(),
		mockInit: vi.fn(),
		mockLocalDbNew: vi.fn()
	}));

	vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
		const original = (await importOriginal()) as Record<string, unknown>;
		return {
			...original,
			DotrainRainlang: {
				new: mockRainlangNew
			}
		};
	});

	vi.mock('@rainlanguage/sqlite-web', () => ({
		default: mockInit,
		SQLiteWasmDatabase: {
			new: mockLocalDbNew
		}
	}));

	describe('Layout load function', () => {
		beforeEach(() => {
			vi.clearAllMocks();
			// @ts-expect-error mock storage
			global.localStorage = {
				data: {} as Record<string, string>,
				getItem(key: string) {
					return this.data[key] ?? null;
				},
				setItem(key: string, value: string) {
					this.data[key] = value;
				},
				removeItem(key: string) {
					delete this.data[key];
				}
			};
			mockInit.mockResolvedValue(undefined);
			mockLocalDbNew.mockReturnValue({
				value: { db: true, query: vi.fn(), wipeAndRecreate: vi.fn() }
			});
		});

		it('should return errorMessage if rainlang fails to load', async () => {
			mockRainlangNew.mockRejectedValueOnce(new Error('Network error'));

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ url: new URL('http://localhost:3000') } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result.errorMessage).toContain('Failed to load rainlang');
		});

		it('should return errorMessage if RaindexClient fails to initialize', async () => {
			mockGetRaindexClient.mockResolvedValue({
				error: { readableMsg: 'Malformed settings' }
			});
			const mockRainlang = { getRaindexClient: mockGetRaindexClient };
			mockRainlangNew.mockResolvedValueOnce({
				value: mockRainlang
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ url: new URL('http://localhost:3000') } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result.errorMessage).toContain('Malformed settings');
		});

		it('should return errorMessage if local database fails to initialize', async () => {
			const mockRainlang = { getRaindexClient: mockGetRaindexClient };
			mockRainlangNew.mockResolvedValueOnce({
				value: mockRainlang
			});
			mockLocalDbNew.mockReturnValue({
				error: { readableMsg: 'Database init failed' }
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ url: new URL('http://localhost:3000') } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result.errorMessage).toContain('Error initializing local database');
		});

		it('should initialize when rainlang and RaindexClient succeed', async () => {
			mockGetRaindexClient.mockResolvedValue({
				value: { client: true }
			});
			const mockRainlang = { getRaindexClient: mockGetRaindexClient };
			mockRainlangNew.mockResolvedValueOnce({
				value: mockRainlang
			});
			mockLocalDbNew.mockReturnValue({
				value: { db: true, query: vi.fn(), wipeAndRecreate: vi.fn() }
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ url: new URL('http://localhost:3000') } as any);

			expect(result.errorMessage).toBeUndefined();
			expect(result.stores).not.toBeNull();
			expect(result.rainlang).toEqual(mockRainlang);
		});
	});
}
