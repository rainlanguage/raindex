import { render, waitFor, screen } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import Layout from './+layout.svelte';
import { validChainIds } from '$lib/stores/settings';
import { networkStatuses, resetLocalDbStatus } from '$lib/stores/localDbStatus';

const { mockPageStore, initialPageState, mockSignerAddressStore, mockValidChainIdsStore } =
	await vi.hoisted(() => import('$lib/__mocks__/stores'));

const mockEnv = vi.hoisted(() => ({ browser: true }));
const mockInitWallet = vi.hoisted(() => vi.fn());
const mockInitializeSnapshotPoc = vi.hoisted(() => vi.fn());

vi.mock('$lib/services/handleWalletInitialization', () => ({
	initWallet: mockInitWallet
}));

vi.mock('$lib/services/snapshotPoc', () => ({
	initializeSnapshotPoc: mockInitializeSnapshotPoc
}));

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$app/environment', () => mockEnv);

vi.mock('$lib/components/TransactionProviderWrapper.svelte', async () => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return {
		default: MockComponent
	};
});

vi.mock('$lib/components/Sidebar.svelte', async () => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockComponent };
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	const ProviderProbe = (await import('$lib/__mocks__/ProviderProbe.svelte')).default;
	return {
		...(await importOriginal()),
		cachedWritableStore: vi.fn(),
		WalletProvider: MockComponent,
		ToastProvider: MockComponent,
		FixedBottomTransaction: MockComponent,
		DotrainRegistryProvider: ProviderProbe,
		LocalDbProvider: ProviderProbe,
		RaindexClientProvider: ProviderProbe
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	signerAddress: mockSignerAddressStore
}));

vi.mock('$lib/stores/settings', () => ({
	validChainIds: mockValidChainIdsStore
}));

vi.mock('$env/static/public', () => ({
	PUBLIC_WALLETCONNECT_PROJECT_ID: 'test-project-id'
}));

vi.mock('@wagmi/connectors', async (importOriginal) => {
	return {
		...(await importOriginal()),
		injected: vi.fn().mockReturnValue('injected-connector'),
		walletConnect: vi.fn().mockReturnValue('wallet-connect-connector')
	};
});

vi.mock('@tanstack/svelte-query', async (importOriginal) => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return {
		...(await importOriginal()),
		QueryClientProvider: MockComponent,
		QueryClient: vi.fn().mockImplementation(() => ({
			name: 'test'
		}))
	};
});

describe('Layout component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		mockPageStore.reset();
		mockEnv.browser = true;
		mockInitWallet.mockResolvedValue(null);
		mockValidChainIdsStore.reset();
		resetLocalDbStatus();
	});

	it('displays an error message if wallet initialization fails', async () => {
		mockInitWallet.mockResolvedValue(
			'Failed to initialize wallet connection: Test error. Please try again or check console.'
		);
		mockPageStore.mockSetSubscribeValue(initialPageState);

		render(Layout);

		const errorMessage = await screen.findByText(
			'Failed to initialize wallet connection: Test error. Please try again or check console.'
		);
		expect(errorMessage).toBeInTheDocument();
	});

	it('renders Homepage when on root path', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			url: new URL('http://localhost/')
		});

		const { container } = render(Layout);

		await waitFor(() => {
			expect(container.querySelector('main')).not.toBeInTheDocument();
			expect(screen.getByTestId('homepage')).toBeInTheDocument();
		});
	});

	it('does not bootstrap the snapshot on the provider-free homepage', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/?snapshot-poc=1')
		});

		render(Layout);

		await waitFor(() => expect(screen.getByTestId('homepage')).toBeInTheDocument());
		expect(mockInitializeSnapshotPoc).not.toHaveBeenCalled();
	});

	it('renders main content when not on root path', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			url: new URL('http://localhost/some-page')
		});

		render(Layout);

		await waitFor(() => {
			expect(screen.getByTestId('layout-container')).toBeInTheDocument();
		});
	});

	it('renders snapshot progress immediately and then exposes the initialized providers', async () => {
		let resolveBootstrap: (runtime: unknown) => void = () => {};
		mockInitializeSnapshotPoc.mockImplementation(
			(_registryUrl: string, onPhase: (phase: string) => void) => {
				onPhase('Opening local database');
				return new Promise((resolve) => {
					resolveBootstrap = resolve;
				});
			}
		);
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				errorMessage: '',
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example',
				registry: null,
				localDb: null,
				raindexClient: null
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});

		render(Layout);

		expect(screen.getByTestId('snapshot-poc-loading-shell')).toBeInTheDocument();
		expect(screen.getByTestId('local-db-syncing-notice')).toHaveTextContent(
			'Opening local database'
		);
		expect(screen.getByRole('status')).toHaveTextContent('Opening local database');
		expect(mockInitializeSnapshotPoc).toHaveBeenCalledWith(
			'https://registry.example',
			expect.any(Function)
		);

		const raindexClient = {
			free: vi.fn(),
			getUniqueChainIds: () => ({ value: [], error: undefined }),
			getLocalDbSyncSnapshot: () =>
				Promise.resolve({
					value: undefined,
					error: { readableMsg: 'not configured' }
				})
		};
		const registry = { id: 'registry', free: vi.fn() };
		const localDb = { id: 'database', free: vi.fn() };
		const free = vi.fn().mockResolvedValue(undefined);
		resolveBootstrap({
			registry,
			localDb,
			raindexClient,
			free,
			snapshotBootstrap: {
				status: 'reused',
				elapsedMs: 4,
				readyElapsedMs: 7,
				bytesWritten: 1_051_512_832
			}
		});

		await waitFor(() => {
			expect(screen.getByTestId('layout-container')).toBeInTheDocument();
			expect(screen.getByText('SQLite snapshot POC: reused')).toBeInTheDocument();
			expect(screen.getByTestId('snapshot-poc-result')).toHaveAttribute('role', 'status');
		});

		mockPageStore.mockSetSubscribeValue({
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/vaults')
		});

		await waitFor(() => expect(screen.getByTestId('layout-container')).toBeInTheDocument());
		expect(mockInitializeSnapshotPoc).toHaveBeenCalledTimes(1);
		expect(raindexClient.free).not.toHaveBeenCalled();
		expect(localDb.free).not.toHaveBeenCalled();
		expect(registry.free).not.toHaveBeenCalled();
		expect(free).not.toHaveBeenCalled();
	});

	it('shows the homepage immediately when navigating away during snapshot bootstrap', async () => {
		mockInitializeSnapshotPoc.mockImplementation(() => new Promise(() => {}));
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});

		render(Layout);
		expect(screen.getByTestId('snapshot-poc-loading-shell')).toBeInTheDocument();

		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/')
		});

		await waitFor(() => expect(screen.getByTestId('homepage')).toBeInTheDocument());
		expect(screen.queryByTestId('snapshot-poc-loading-shell')).not.toBeInTheDocument();
	});

	it('frees a snapshot runtime that finishes after the layout is destroyed', async () => {
		const clearIntervalSpy = vi.spyOn(window, 'clearInterval');
		let resolveBootstrap: (runtime: unknown) => void = () => {};
		mockInitializeSnapshotPoc.mockImplementation(
			() =>
				new Promise((resolve) => {
					resolveBootstrap = resolve;
				})
		);
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});
		const free = vi.fn().mockResolvedValue(undefined);

		const { unmount } = render(Layout);
		unmount();
		expect(clearIntervalSpy).toHaveBeenCalled();
		resolveBootstrap({
			registry: {},
			localDb: {},
			raindexClient: {},
			free
		});

		await waitFor(() => {
			expect(free).toHaveBeenCalledTimes(1);
		});
		clearIntervalSpy.mockRestore();
	});

	it.each([
		['string', 'Snapshot SHA-256 mismatch', 'Snapshot SHA-256 mismatch'],
		['message object', { message: 'Snapshot request failed' }, 'Snapshot request failed'],
		['readable message object', { readableMsg: 'Snapshot is invalid' }, 'Snapshot is invalid'],
		['plain object', { status: 503 }, '{"status":503}'],
		['null', null, 'Unknown error'],
		[
			'circular object',
			(() => {
				const circular: Record<string, unknown> = {};
				circular.self = circular;
				return circular;
			})(),
			'Unknown error'
		]
	])('shows useful %s snapshot initialization failures', async (_label, rejection, message) => {
		mockInitializeSnapshotPoc.mockRejectedValue(rejection);
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});

		render(Layout);

		await waitFor(() => {
			expect(screen.getByTestId('error-page')).toBeInTheDocument();
			expect(screen.queryByTestId('snapshot-poc-result')).not.toBeInTheDocument();
			expect(screen.getByTestId('error-page')).toHaveAttribute('role', 'alert');
			expect(screen.getByText(`Error initializing local database: ${message}`)).toBeInTheDocument();
		});
	});

	it('remounts one-shot provider contexts with page-owned resources after navigation', async () => {
		const createClient = (id: string) => ({
			id,
			getUniqueChainIds: () => ({ value: [], error: undefined }),
			getLocalDbSyncSnapshot: () =>
				Promise.resolve({
					value: undefined,
					error: { readableMsg: 'not configured' }
				})
		});
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				registry: { id: 'first-registry' } as never,
				localDb: { id: 'first-database' } as never,
				raindexClient: createClient('first-client') as never
			},
			url: new URL('http://localhost/orders')
		});
		render(Layout);

		const initialLocalDbProvider = screen.getByTestId('local-db-provider');
		const initialClientProvider = screen.getByTestId('raindex-client-provider');
		expect(initialLocalDbProvider).toHaveAttribute('data-resource-id', 'first-database');
		expect(initialClientProvider).toHaveAttribute('data-resource-id', 'first-client');

		mockPageStore.mockSetSubscribeValue({
			data: {
				...initialPageState.data,
				registry: { id: 'next-registry' } as never,
				localDb: { id: 'next-database' } as never,
				raindexClient: createClient('next-client') as never
			},
			url: new URL('http://localhost/vaults')
		});

		await waitFor(() => {
			const nextLocalDbProvider = screen.getByTestId('local-db-provider');
			const nextClientProvider = screen.getByTestId('raindex-client-provider');
			expect(nextLocalDbProvider).not.toBe(initialLocalDbProvider);
			expect(nextClientProvider).not.toBe(initialClientProvider);
			expect(nextLocalDbProvider).toHaveAttribute('data-resource-id', 'next-database');
			expect(nextClientProvider).toHaveAttribute('data-resource-id', 'next-client');
		});

		mockPageStore.mockSetSubscribeValue({
			data: {
				...initialPageState.data,
				errorMessage: 'Navigation failed'
			}
		});

		await waitFor(() => {
			expect(screen.getByTestId('error-page')).toHaveTextContent('Navigation failed');
		});
	});

	it('ignores a stale sync snapshot after navigating to a different client', async () => {
		let resolveFirstSnapshot: (result: unknown) => void = () => {};
		let resolveSecondSnapshot: (result: unknown) => void = () => {};
		const firstClient = {
			id: 'first-client',
			getUniqueChainIds: () => ({ value: [1], error: undefined }),
			getLocalDbSyncSnapshot: () =>
				new Promise((resolve) => {
					resolveFirstSnapshot = resolve;
				})
		};
		const secondClient = {
			id: 'second-client',
			getUniqueChainIds: () => ({ value: [137], error: undefined }),
			getLocalDbSyncSnapshot: () =>
				new Promise((resolve) => {
					resolveSecondSnapshot = resolve;
				})
		};
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				registry: { id: 'first-registry' } as never,
				localDb: { id: 'first-database' } as never,
				raindexClient: firstClient as never
			},
			url: new URL('http://localhost/orders')
		});
		render(Layout);

		await waitFor(() => expect(get(validChainIds)).toEqual([1]));
		mockPageStore.mockSetSubscribeValue({
			data: {
				...initialPageState.data,
				registry: { id: 'second-registry' } as never,
				localDb: { id: 'second-database' } as never,
				raindexClient: secondClient as never
			},
			url: new URL('http://localhost/vaults')
		});
		await waitFor(() => expect(get(validChainIds)).toEqual([137]));

		resolveSecondSnapshot({
			value: {
				configured: true,
				networks: [{ chainId: 137, status: 'active' }],
				raindexes: []
			},
			error: undefined
		});
		await waitFor(() => expect(Array.from(get(networkStatuses).keys())).toEqual([137]));

		resolveFirstSnapshot({
			value: {
				configured: true,
				networks: [{ chainId: 1, status: 'active' }],
				raindexes: []
			},
			error: undefined
		});
		await Promise.resolve();
		await Promise.resolve();

		expect(get(validChainIds)).toEqual([137]);
		expect(Array.from(get(networkStatuses).keys())).toEqual([137]);
	});

	it('starts snapshot bootstrap when snapshot mode is entered by client navigation', async () => {
		mockInitializeSnapshotPoc.mockImplementation(() => new Promise(() => {}));
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			url: new URL('http://localhost/orders')
		});
		render(Layout);
		expect(mockInitializeSnapshotPoc).not.toHaveBeenCalled();

		mockPageStore.mockSetSubscribeValue({
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/vaults?snapshot-poc=1')
		});

		await waitFor(() => {
			expect(screen.getByTestId('snapshot-poc-loading-shell')).toBeInTheDocument();
			expect(mockInitializeSnapshotPoc).toHaveBeenCalledWith(
				'https://registry.example',
				expect.any(Function)
			);
		});
	});

	it('preserves a snapshot bootstrap failure across client navigation', async () => {
		mockInitializeSnapshotPoc.mockRejectedValue({
			readableMsg: 'Snapshot is corrupt'
		});
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});
		render(Layout);

		await screen.findByText('Error initializing local database: Snapshot is corrupt');
		mockPageStore.mockSetSubscribeValue({
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/vaults')
		});

		await waitFor(() => {
			expect(
				screen.getByText('Error initializing local database: Snapshot is corrupt')
			).toBeInTheDocument();
			expect(screen.queryByTestId('layout-container')).not.toBeInTheDocument();
		});
		expect(mockInitializeSnapshotPoc).toHaveBeenCalledTimes(1);
	});

	it('frees an owned runtime and cancels its elapsed timer on teardown', async () => {
		const clearIntervalSpy = vi.spyOn(window, 'clearInterval');
		const free = vi.fn().mockResolvedValue(undefined);
		mockInitializeSnapshotPoc.mockResolvedValue({
			registry: {},
			localDb: {},
			raindexClient: {
				getUniqueChainIds: () => ({ value: [], error: undefined }),
				getLocalDbSyncSnapshot: () =>
					Promise.resolve({
						value: undefined,
						error: { readableMsg: 'not configured' }
					})
			},
			free,
			snapshotBootstrap: {
				status: 'reused',
				elapsedMs: 4,
				readyElapsedMs: 7,
				bytesWritten: 100
			}
		});
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				snapshotPocEnabled: true,
				registryUrl: 'https://registry.example'
			},
			url: new URL('http://localhost/orders?snapshot-poc=1')
		});

		const { unmount } = render(Layout);
		await screen.findByTestId('layout-container');
		unmount();

		expect(free).toHaveBeenCalledTimes(1);
		expect(clearIntervalSpy).toHaveBeenCalled();
		clearIntervalSpy.mockRestore();
	});

	it('does not initialize wallet when not in browser environment', async () => {
		const originalNavigator = global.navigator;
		mockEnv.browser = false;
		render(Layout);
		expect(mockInitWallet).not.toHaveBeenCalled();
		Object.defineProperty(global, 'navigator', {
			value: originalNavigator,
			writable: true
		});
	});

	it('displays an error page if page.error is set', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				errorMessage: 'Test error'
			}
		});
		render(Layout);

		await waitFor(() => {
			expect(screen.getByText('Test error')).toBeInTheDocument();
			expect(screen.getByTestId('error-page')).toBeInTheDocument();
		});
	});
});
