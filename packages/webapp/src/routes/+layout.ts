import type { AppStoresInterface } from '@rainlanguage/ui-components';
import { writable } from 'svelte/store';
import type { LayoutLoad } from './$types';
import { RaindexClient, type AccountCfg, type Address, type Hex } from '@rainlanguage/orderbook';
import type { Mock } from 'vitest';

export interface LayoutData {
	errorMessage?: string;
	stores: AppStoresInterface | null;
	raindexClient: RaindexClient | null;
}

const REMOTE_SETTINGS_URL =
	'https://raw.githubusercontent.com/rainlanguage/rain.strategies/19c00deb27fdbd61c76e0d01388c83c7801e4bd4/settings.yaml';

export const load: LayoutLoad<LayoutData> = async ({ fetch }) => {
	let errorMessage: string | undefined;
	let settingsYamlText: string;

	try {
		const response = await fetch(REMOTE_SETTINGS_URL);
		if (!response.ok) {
			throw new Error('Error status: ' + response.status.toString());
		}
		settingsYamlText = await response.text();
	} catch (error: unknown) {
		errorMessage = 'Failed to get site config settings. ' + (error as Error).message;
		return {
			errorMessage,
			stores: null,
			raindexClient: null
		};
	}

	let raindexClient: RaindexClient | null = null;
	try {
		const raindexClientRes = RaindexClient.new([settingsYamlText]);
		if (raindexClientRes.error) {
			return {
				errorMessage: raindexClientRes.error.readableMsg,
				stores: null,
				raindexClient: null
			};
		} else {
			raindexClient = raindexClientRes.value;
		}
	} catch (error: unknown) {
		return {
			errorMessage: 'Error initializing RaindexClient: ' + (error as Error).message,
			stores: null,
			raindexClient: null
		};
	}

	return {
		stores: {
			selectedChainIds: writable<number[]>([]),
			accounts: writable<Record<string, AccountCfg>>({}),
			activeAccountsItems: writable<Record<string, Address>>({}),
			// Instantiate with false to show only active orders
			showInactiveOrders: writable<boolean>(false),
			// @ts-expect-error initially the value is empty
			orderHash: writable<Hex>(''),
			hideZeroBalanceVaults: writable<boolean>(false),
			showMyItemsOnly: writable<boolean>(false),
			activeTokens: writable<Address[]>([])
		},
		raindexClient
	};
};

export const ssr = false;

if (import.meta.vitest) {
	const { describe, it, expect, beforeEach, vi } = import.meta.vitest;

	const mockFetch = vi.fn();
	vi.stubGlobal('fetch', mockFetch);

	vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
		return {
			...(await importOriginal()),
			RaindexClient: {
				new: vi.fn()
			}
		};
	});

	describe('Layout load function', () => {
		beforeEach(() => {
			vi.clearAllMocks();
			vi.resetAllMocks();
		});

		it('should return errorMessage if fetch fails with non-OK status', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 404,
				statusText: 'Not Found'
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Failed to get site config settings.');
			expect(result.errorMessage).toContain('Error status: 404');
		});

		it('should return errorMessage if response.text() fails', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.reject(new Error('Invalid YAML'))
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Failed to get site config settings.');
			expect(result.errorMessage).toContain('Invalid YAML');
		});

		it('should handle fetch failure', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Failed to get site config settings.');
			expect(result.errorMessage).toContain('Network error');
		});

		it('should handle empty or malformed settings YAML', async () => {
			(RaindexClient.new as Mock).mockReturnValue({
				value: undefined,
				error: {
					msg: 'Malformed settings',
					readableMsg: 'Malformed settings'
				}
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve('malformed')
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Malformed settings');
			expect(result.errorMessage).toContain('Malformed settings');
		});
	});
}
